// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Cargo quality analysis tool for Rust code.
//!
//! This binary provides a command-line interface for analyzing and improving
//! Rust code quality according to project standards. It detects common issues
//! and suggests or applies fixes automatically.
//!
//! # Available Commands
//!
//! - `cargo qual check` - Analyze code without modifications
//! - `cargo qual fix` - Apply automatic fixes
//! - `cargo qual format` - Format code according to quality rules
//!
//! # Examples
//!
//! ```bash
//! cargo qual check src/
//! cargo qual fix --dry-run src/
//! cargo qual format .
//! ```

use std::{fs, path::PathBuf};

use masterror::AppResult;

use crate::{
    analyzer::{AnalysisResult, Fix, Issue},
    analyzers::get_analyzers,
    cli::{Command, QualityArgs, Shell},
    differ::{DiffResult, generate_diff, show_full, show_interactive, show_summary},
    error::{IoError, ParseError},
    file_utils::collect_rust_files,
    mod_rs::{ModRsResult, find_mod_rs_issues, fix_all_mod_rs},
    report::{GlobalReport, Report}
};

mod analyzer;
mod analyzers;
mod cli;
mod differ;
mod error;
mod file_utils;
mod formatter;
mod help;
mod mod_rs;
mod report;

fn main() -> AppResult<()> {
    let args = QualityArgs::parse_args();

    match args.command {
        Command::Check {
            path,
            verbose,
            analyzer,
            color
        } => check_quality(&path, verbose, analyzer.as_deref(), color)?,
        Command::Fix {
            path,
            dry_run,
            analyzer
        } => fix_quality(&path, dry_run, analyzer.as_deref())?,
        Command::Format {
            path
        } => format_quality(&path)?,
        Command::Fmt {
            path: _
        } => formatter::format_code()?,
        Command::Diff {
            path,
            summary,
            interactive,
            analyzer,
            color
        } => run_diff(&path, summary, interactive, analyzer.as_deref(), color)?,
        Command::Help => {
            help::display_help();
            return Ok(());
        }
        Command::Completions {
            shell
        } => {
            generate_completions(shell);
            return Ok(());
        }
        Command::Setup => {
            setup_completions()?;
            return Ok(());
        }
        Command::ModRs {
            path,
            fix
        } => run_mod_rs(&path, fix)?
    }

    Ok(())
}

/// Generate shell completions.
///
/// Outputs completion script for the specified shell to stdout.
///
/// # Arguments
///
/// * `shell` - Target shell for completion generation
fn generate_completions(shell: Shell) {
    use clap::CommandFactory;
    use clap_complete::{Shell as CompShell, generate};

    let mut cmd = QualityArgs::command();
    let bin_name = "cargo-qual";

    let comp_shell = match shell {
        Shell::Bash => CompShell::Bash,
        Shell::Fish => CompShell::Fish,
        Shell::Zsh => CompShell::Zsh,
        Shell::PowerShell => CompShell::PowerShell,
        Shell::Elvish => CompShell::Elvish
    };

    generate(comp_shell, &mut cmd, bin_name, &mut std::io::stdout());
}

/// Setup shell completions automatically.
///
/// Detects current shell and installs completions to standard location.
///
/// # Returns
///
/// `AppResult<()>` - Ok if setup succeeds
fn setup_completions() -> AppResult<()> {
    let shell_name = detect_shell();

    let Some((shell, comp_dir, file_name)) = get_completion_config(&shell_name) else {
        println!("❌ Unsupported shell: {}", shell_name);
        println!("Supported shells: bash, fish, zsh");
        println!("\nManual installation:");
        println!("  cargo qual completions <shell> > <completion-file>");
        return Ok(());
    };

    fs::create_dir_all(&comp_dir).map_err(IoError::from)?;
    let comp_file = comp_dir.join(file_name);

    if shell_name == "fish" {
        install_fish_completions(&comp_file)?;
    } else {
        install_generated_completions(shell, &comp_file)?;
    }

    println!(
        "✓ {} completions installed to: {}",
        shell_name,
        comp_file.display()
    );
    println!(
        "\nCompletions will be available in new {} sessions",
        shell_name
    );
    println!("Or run: source {}", comp_file.display());

    Ok(())
}

/// Detects current shell from SHELL environment variable.
///
/// # Returns
///
/// Shell name (e.g., "bash", "fish", "zsh")
#[inline]
fn detect_shell() -> String {
    use std::env;

    let shell_path = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
    let shell_path_buf = PathBuf::from(&shell_path);
    shell_path_buf
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("sh")
        .to_string()
}

/// Gets HOME directory path.
///
/// # Returns
///
/// Home directory path or "~" if not found
#[inline]
fn get_home_dir() -> String {
    use std::env;

    env::var("HOME").unwrap_or_else(|_| String::from("~"))
}

/// Gets XDG_CONFIG_HOME directory.
///
/// Falls back to ~/.config if not set.
///
/// # Returns
///
/// Config directory path
#[inline]
fn get_xdg_config_home() -> PathBuf {
    use std::env;

    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(get_home_dir()).join(".config"))
}

/// Gets XDG_DATA_HOME directory.
///
/// Falls back to ~/.local/share if not set.
///
/// # Returns
///
/// Data directory path
#[inline]
fn get_xdg_data_home() -> PathBuf {
    use std::env;

    env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(get_home_dir()).join(".local").join("share"))
}

/// Gets completion configuration for a shell.
///
/// Returns None for unsupported shells.
///
/// # Arguments
///
/// * `shell_name` - Shell name (e.g., "bash", "fish", "zsh")
///
/// # Returns
///
/// Option<(Shell, PathBuf, &'static str)> - Shell type, directory, filename
fn get_completion_config(shell_name: &str) -> Option<(Shell, PathBuf, &'static str)> {
    match shell_name {
        "fish" => {
            let dir = get_xdg_config_home().join("fish").join("completions");
            Some((Shell::Fish, dir, "cargo.fish"))
        }
        "bash" => {
            let dir = get_xdg_data_home()
                .join("bash-completion")
                .join("completions");
            Some((Shell::Bash, dir, "cargo-quality"))
        }
        "zsh" => {
            let dir = get_xdg_data_home().join("zsh").join("site-functions");
            Some((Shell::Zsh, dir, "_cargo-quality"))
        }
        _ => None
    }
}

/// Installs fish shell completions.
///
/// Uses hardcoded fish completion script.
///
/// # Arguments
///
/// * `comp_file` - Completion file path
///
/// # Returns
///
/// `AppResult<()>` - Ok if installation succeeds
fn install_fish_completions(comp_file: &std::path::Path) -> AppResult<()> {
    let fish_completions = r#"# Completion for cargo qual subcommand
complete -c cargo -n "__fish_seen_subcommand_from quality" -s h -l help -d 'Print help'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "check" -d 'Check code quality'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "fix" -d 'Fix quality issues'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "format" -d 'Format code'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "fmt" -d 'Run cargo +nightly fmt'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "diff" -d 'Show proposed changes'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "help" -d 'Display help'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "completions" -d 'Generate completions'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "setup" -d 'Setup completions'

# Diff options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from diff" -s s -l summary -d 'Brief summary'
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from diff" -s i -l interactive -d 'Interactive mode'

# Check options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from check" -s v -l verbose -d 'Detailed output'

# Fix options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from fix" -s d -l dry-run -d 'Dry run'

# Completions options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from completions" -f -a "bash fish zsh powershell elvish"
"#;
    fs::write(comp_file, fish_completions).map_err(IoError::from)?;
    Ok(())
}

/// Installs generated completions for bash/zsh.
///
/// Uses clap_complete to generate shell-specific completions.
///
/// # Arguments
///
/// * `shell` - Shell type
/// * `comp_file` - Completion file path
///
/// # Returns
///
/// `AppResult<()>` - Ok if installation succeeds
fn install_generated_completions(shell: Shell, comp_file: &std::path::Path) -> AppResult<()> {
    use clap::CommandFactory;
    use clap_complete::{Shell as CompShell, generate};

    let mut cmd = QualityArgs::command();
    let comp_shell = match shell {
        Shell::Bash => CompShell::Bash,
        Shell::Zsh => CompShell::Zsh,
        _ => unreachable!()
    };

    let mut file = fs::File::create(comp_file).map_err(IoError::from)?;
    generate(comp_shell, &mut cmd, "cargo-qual", &mut file);
    Ok(())
}

/// Run mod.rs analyzer.
///
/// Finds mod.rs files and optionally fixes them.
///
/// # Arguments
///
/// * `path` - Path to analyze
/// * `fix` - Apply fixes automatically
fn run_mod_rs(path: &str, fix: bool) -> AppResult<()> {
    let result = find_mod_rs_issues(path)?;

    if result.is_empty() {
        println!("No mod.rs files found");
        return Ok(());
    }

    if fix {
        let fixed = fix_all_mod_rs(path)?;
        println!("Fixed {} mod.rs files", fixed);
    } else {
        println!("Found {} mod.rs files:", result.len());
        for issue in &result.issues {
            println!(
                "  {} -> {}",
                issue.path.display(),
                issue.suggested.display()
            );
        }
        println!("\nRun with --fix to apply changes");
    }

    Ok(())
}

/// Check code quality without modifying files.
///
/// Analyzes all Rust files in the specified path and reports issues found
/// by each analyzer or a specific analyzer if provided. Prints detailed
/// reports for files with issues.
///
/// # Arguments
///
/// * `path` - File or directory path to analyze
/// * `verbose` - Print confirmation for files without issues
/// * `analyzer_name` - Optional analyzer name to run (e.g., "inline_comments")
///
/// # Returns
///
/// `AppResult<()>` - Ok if analysis completes, error on IO or parse failures
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::check_quality;
/// check_quality("src/", true, None, false).unwrap();
/// check_quality("src/", false, Some("inline_comments"), true).unwrap();
/// ```
fn check_quality(
    path: &str,
    verbose: bool,
    analyzer_name: Option<&str>,
    color: bool
) -> AppResult<()> {
    let files = collect_rust_files(path)?;
    let all_analyzers = get_analyzers();

    let analyzers: Vec<_> = if let Some(name) = analyzer_name {
        all_analyzers
            .into_iter()
            .filter(|a| a.name() == name)
            .collect()
    } else {
        all_analyzers
    };

    if let Some(name) = analyzer_name
        && analyzers.is_empty()
        && name != "mod_rs"
    {
        eprintln!("Unknown analyzer: {}. Available analyzers:", name);
        for analyzer in get_analyzers() {
            eprintln!("  - {}", analyzer.name());
        }
        eprintln!("  - mod_rs");
        return Ok(());
    }

    let mut global_report = GlobalReport::new();

    let should_check_mod_rs = analyzer_name.is_none() || analyzer_name == Some("mod_rs");
    if should_check_mod_rs {
        let mod_rs_result = find_mod_rs_issues(path)?;
        if !mod_rs_result.is_empty() {
            add_mod_rs_to_report(&mod_rs_result, &mut global_report);
        }
    }

    if analyzer_name != Some("mod_rs") {
        for file_path in files {
            let content = fs::read_to_string(&file_path).map_err(IoError::from)?;
            let ast = syn::parse_file(&content).map_err(ParseError::from)?;

            let mut report = Report::new(file_path.display().to_string());

            for analyzer in &analyzers {
                let result = analyzer.analyze(&ast, &content)?;
                report.add_result(analyzer.name().to_string(), result);
            }

            if report.total_issues() > 0 || verbose {
                global_report.add_report(report);
            }
        }
    }

    if global_report.total_issues() > 0 {
        if let Some(analyzer) = analyzer_name {
            print!("{}", global_report.display_analyzer(analyzer, color));
        } else if verbose {
            print!("{}", global_report.display_verbose(color));
        } else {
            print!("{}", global_report.display_compact(color));
        }
    } else {
        print!("{}", global_report.display_compact(color));
    }

    Ok(())
}

/// Adds mod.rs issues to the global report.
///
/// Converts ModRsResult into Report format for unified display.
///
/// # Arguments
///
/// * `mod_rs_result` - Result from mod_rs analysis
/// * `global_report` - Global report to add issues to
fn add_mod_rs_to_report(mod_rs_result: &ModRsResult, global_report: &mut GlobalReport) {
    for issue in &mod_rs_result.issues {
        let mut report = Report::new(issue.path.display().to_string());

        let analysis_result = AnalysisResult {
            issues:        vec![Issue {
                line:    issue.line,
                column:  issue.column,
                message: issue.message.clone(),
                fix:     Fix::Simple(issue.suggested.display().to_string())
            }],
            fixable_count: 1
        };

        report.add_result("mod_rs".to_string(), analysis_result);
        global_report.add_report(report);
    }
}

/// Fix quality issues automatically.
///
/// Applies automatic fixes from all analyzers or a specific analyzer to Rust
/// files in the specified path. Can run in dry-run mode to preview changes
/// without modifying files.
///
/// # Arguments
///
/// * `path` - File or directory path to fix
/// * `dry_run` - If true, report fixes but do not modify files
/// * `analyzer_name` - Optional analyzer name to run (e.g., "path_import")
///
/// # Returns
///
/// `AppResult<()>` - Ok if fixes applied successfully, error on IO or parse
/// failures
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::fix_quality;
/// fix_quality("src/", true, None).unwrap();
/// fix_quality("src/", false, Some("path_import")).unwrap();
/// ```
fn fix_quality(path: &str, dry_run: bool, analyzer_name: Option<&str>) -> AppResult<()> {
    let all_analyzers = get_analyzers();

    let analyzers: Vec<_> = if let Some(name) = analyzer_name {
        all_analyzers
            .into_iter()
            .filter(|a| a.name() == name)
            .collect()
    } else {
        all_analyzers
    };

    if let Some(name) = analyzer_name
        && analyzers.is_empty()
        && name != "mod_rs"
    {
        eprintln!("Unknown analyzer: {}. Available analyzers:", name);
        for analyzer in get_analyzers() {
            eprintln!("  - {}", analyzer.name());
        }
        eprintln!("  - mod_rs");
        return Ok(());
    }

    let should_fix_mod_rs = analyzer_name.is_none() || analyzer_name == Some("mod_rs");
    if should_fix_mod_rs {
        let mod_rs_result = find_mod_rs_issues(path)?;
        if !mod_rs_result.is_empty() {
            if dry_run {
                for issue in &mod_rs_result.issues {
                    println!(
                        "Would fix: {} -> {}",
                        issue.path.display(),
                        issue.suggested.display()
                    );
                }
            } else {
                let fixed = fix_all_mod_rs(path)?;
                if fixed > 0 {
                    println!("Fixed {} mod.rs files", fixed);
                }
            }
        }
    }

    if analyzer_name != Some("mod_rs") {
        let files = collect_rust_files(path)?;
        for file_path in files {
            let content = fs::read_to_string(&file_path).map_err(IoError::from)?;
            let mut ast = syn::parse_file(&content).map_err(ParseError::from)?;

            let mut total_fixed = 0;

            for analyzer in &analyzers {
                let fixed = analyzer.fix(&mut ast)?;
                total_fixed += fixed;
            }

            if total_fixed > 0 {
                println!("Fixed {} issues in {}", total_fixed, file_path.display());

                if !dry_run {
                    let formatted = prettyplease::unparse(&ast);
                    fs::write(&file_path, formatted).map_err(IoError::from)?;
                }
            }
        }
    }

    Ok(())
}

/// Format code according to quality rules.
///
/// Wrapper around `fix_quality` that applies all fixes without dry-run mode.
///
/// # Arguments
///
/// * `path` - File or directory path to format
///
/// # Returns
///
/// `AppResult<()>` - Ok if formatting succeeds, error otherwise
fn format_quality(path: &str) -> AppResult<()> {
    fix_quality(path, false, None)
}

/// Show diff of proposed quality fixes.
///
/// Displays changes that would be made by quality analyzers. Supports three
/// modes:
/// - Full: Complete unified diff output
/// - Summary: Brief statistics by file and analyzer
/// - Interactive: User selects which changes to apply
///
/// # Arguments
///
/// * `path` - File or directory path to analyze
/// * `summary` - Show brief summary instead of full diff
/// * `interactive` - Enable interactive mode for selecting changes
/// * `analyzer_name` - Optional analyzer name to run (e.g., "path_import")
///
/// # Returns
///
/// `AppResult<()>` - Ok if diff generated successfully, error otherwise
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::run_diff;
/// run_diff("src/", false, false, None, false).unwrap();
/// run_diff("src/", true, false, Some("path_import"), false).unwrap();
/// ```
fn run_diff(
    path: &str,
    summary: bool,
    interactive: bool,
    analyzer_name: Option<&str>,
    color: bool
) -> AppResult<()> {
    let files = collect_rust_files(path)?;
    let all_analyzers = get_analyzers();

    let analyzers: Vec<_> = if let Some(name) = analyzer_name {
        all_analyzers
            .into_iter()
            .filter(|a| a.name() == name)
            .collect()
    } else {
        all_analyzers
    };

    if let Some(name) = analyzer_name
        && analyzers.is_empty()
    {
        eprintln!("Unknown analyzer: {}. Available analyzers:", name);
        for analyzer in get_analyzers() {
            eprintln!("  - {}", analyzer.name());
        }
        return Ok(());
    }

    let mut result = DiffResult::new();

    for file_path in files {
        let file_diff = generate_diff(file_path.to_str().unwrap_or(""), &analyzers)?;
        result.add_file(file_diff);
    }

    if result.total_changes() == 0 {
        println!("No changes proposed");
        return Ok(());
    }

    if summary {
        show_summary(&result, color);
    } else if interactive {
        let _selected = show_interactive(&result, color)?;
    } else {
        show_full(&result, color);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_check_quality() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let result = check_quality(temp_dir.path().to_str().unwrap(), false, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_quality_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("clean.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = check_quality(temp_dir.path().to_str().unwrap(), true, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fix_quality_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = fix_quality(temp_dir.path().to_str().unwrap(), true, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_quality() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = format_quality(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_quality_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad.rs");
        fs::write(&file_path, "fn main() { invalid rust syntax +++").unwrap();

        let result = check_quality(temp_dir.path().to_str().unwrap(), false, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_quality_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad.rs");
        fs::write(&file_path, "fn main() { invalid rust +++").unwrap();

        let result = fix_quality(temp_dir.path().to_str().unwrap(), false, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_quality_with_fixes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let result = fix_quality(temp_dir.path().to_str().unwrap(), false, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_quality_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let result = check_quality(temp_dir.path().to_str().unwrap(), false, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fix_quality_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let result = fix_quality(temp_dir.path().to_str().unwrap(), true, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_quality_with_changes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let result = format_quality(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diff_full() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let result = run_diff(temp_dir.path().to_str().unwrap(), false, false, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diff_summary() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let result = run_diff(temp_dir.path().to_str().unwrap(), true, false, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diff_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = run_diff(temp_dir.path().to_str().unwrap(), false, false, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diff_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() { invalid +++").unwrap();

        let result = run_diff(temp_dir.path().to_str().unwrap(), false, false, None, false);
        assert!(result.is_err());
    }
}
