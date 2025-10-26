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
//! - `cargo quality check` - Analyze code without modifications
//! - `cargo quality fix` - Apply automatic fixes
//! - `cargo quality format` - Format code according to quality rules
//!
//! # Examples
//!
//! ```bash
//! cargo quality check src/
//! cargo quality fix --dry-run src/
//! cargo quality format .
//! ```

use std::{fs, path::PathBuf};

use masterror::AppResult;
use walkdir::WalkDir;

use crate::{
    analyzers::get_analyzers,
    cli::{Command, QualityArgs, Shell},
    differ::{
        DiffResult, collect_files, generate_diff, show_full, show_interactive, show_summary
    },
    error::{IoError, ParseError},
    report::Report
};

mod analyzer;
mod analyzers;
mod cli;
mod differ;
mod error;
mod formatter;
mod help;
mod report;

fn main() -> AppResult<()> {
    let args = QualityArgs::parse_args();

    match args.command {
        Command::Check {
            path,
            verbose
        } => check_quality(&path, verbose)?,
        Command::Fix {
            path,
            dry_run
        } => fix_quality(&path, dry_run)?,
        Command::Format {
            path
        } => format_quality(&path)?,
        Command::Fmt {
            path: _
        } => formatter::format_code()?,
        Command::Diff {
            path,
            summary,
            interactive
        } => run_diff(&path, summary, interactive)?,
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

    let mut cmd = crate::cli::CargoCli::command();
    let bin_name = "cargo-quality";

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
    use std::env;

    let shell_path = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
    let shell_path_buf = PathBuf::from(&shell_path);
    let shell_name = shell_path_buf
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("sh");

    let (shell, comp_dir, file_name) = match shell_name {
        "fish" => {
            let dir = env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    PathBuf::from(env::var("HOME").unwrap_or_else(|_| String::from("~")))
                        .join(".config")
                })
                .join("fish")
                .join("completions");
            (Shell::Fish, dir, "cargo.fish")
        }
        "bash" => {
            let dir = env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    PathBuf::from(env::var("HOME").unwrap_or_else(|_| String::from("~")))
                        .join(".local")
                        .join("share")
                })
                .join("bash-completion")
                .join("completions");
            (Shell::Bash, dir, "cargo-quality")
        }
        "zsh" => {
            let dir = env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    PathBuf::from(env::var("HOME").unwrap_or_else(|_| String::from("~")))
                        .join(".local")
                        .join("share")
                })
                .join("zsh")
                .join("site-functions");
            (Shell::Zsh, dir, "_cargo-quality")
        }
        _ => {
            println!("❌ Unsupported shell: {}", shell_name);
            println!("Supported shells: bash, fish, zsh");
            println!("\nManual installation:");
            println!("  cargo quality completions <shell> > <completion-file>");
            return Ok(());
        }
    };

    fs::create_dir_all(&comp_dir).map_err(IoError::from)?;

    let comp_file = comp_dir.join(file_name);

    if shell_name == "fish" {
        let fish_completions = r#"# Completion for cargo quality subcommand
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
        fs::write(&comp_file, fish_completions).map_err(IoError::from)?;
    } else {
        use clap::CommandFactory;
        use clap_complete::{Shell as CompShell, generate};

        let mut cmd = crate::cli::CargoCli::command();
        let comp_shell = match shell {
            Shell::Bash => CompShell::Bash,
            Shell::Zsh => CompShell::Zsh,
            _ => unreachable!()
        };

        let mut file = fs::File::create(&comp_file).map_err(IoError::from)?;
        generate(comp_shell, &mut cmd, "cargo-quality", &mut file);
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

/// Check code quality without modifying files.
///
/// Analyzes all Rust files in the specified path and reports issues found
/// by each analyzer. Prints detailed reports for files with issues.
///
/// # Arguments
///
/// * `path` - File or directory path to analyze
/// * `verbose` - Print confirmation for files without issues
///
/// # Returns
///
/// `AppResult<()>` - Ok if analysis completes, error on IO or parse failures
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::check_quality;
/// check_quality("src/", true).unwrap();
/// ```
fn check_quality(path: &str, verbose: bool) -> AppResult<()> {
    let files = collect_rust_files(path)?;
    let analyzers = get_analyzers();

    for file_path in files {
        let content = fs::read_to_string(&file_path).map_err(IoError::from)?;
        let ast = syn::parse_file(&content).map_err(ParseError::from)?;

        let mut report = Report::new(file_path.display().to_string());

        for analyzer in &analyzers {
            let result = analyzer.analyze(&ast)?;
            report.add_result(analyzer.name().to_string(), result);
        }

        if report.total_issues() > 0 {
            println!("{}", report);
        } else if verbose {
            println!("OK {}", file_path.display());
        }
    }

    Ok(())
}

/// Fix quality issues automatically.
///
/// Applies automatic fixes from all analyzers to Rust files in the specified
/// path. Can run in dry-run mode to preview changes without modifying files.
///
/// # Arguments
///
/// * `path` - File or directory path to fix
/// * `dry_run` - If true, report fixes but do not modify files
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
/// fix_quality("src/", true).unwrap();
/// fix_quality("src/", false).unwrap();
/// ```
fn fix_quality(path: &str, dry_run: bool) -> AppResult<()> {
    let files = collect_rust_files(path)?;
    let analyzers = get_analyzers();

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
    fix_quality(path, false)
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
///
/// # Returns
///
/// `AppResult<()>` - Ok if diff generated successfully, error otherwise
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::run_diff;
/// run_diff("src/", false, false).unwrap();
/// ```
fn run_diff(path: &str, summary: bool, interactive: bool) -> AppResult<()> {
    let files = collect_files(path)?;
    let analyzers = get_analyzers();

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
        show_summary(&result);
    } else if interactive {
        let _selected = show_interactive(&result)?;
    } else {
        show_full(&result);
    }

    Ok(())
}

/// Collect all Rust source files from a path.
///
/// Recursively walks directories to find all `.rs` files. Follows symbolic
/// links.
///
/// # Arguments
///
/// * `path` - File or directory path to search
///
/// # Returns
///
/// `AppResult<Vec<PathBuf>>` - List of Rust file paths found
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::collect_rust_files;
/// let files = collect_rust_files("src/").unwrap();
/// assert!(files.len() > 0);
/// ```
fn collect_rust_files(path: &str) -> AppResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path_buf = PathBuf::from(path);

    if path_buf.is_file() && path_buf.extension().is_some_and(|e| e == "rs") {
        files.push(path_buf);
    } else if path_buf.is_dir() {
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && let Some(ext) = entry.path().extension()
                && ext == "rs"
            {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_collect_rust_files_empty_dir() {
        let temp_dir = std::env::temp_dir();
        let result = collect_rust_files(temp_dir.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_collect_rust_files_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = collect_rust_files(file_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], file_path);
    }

    #[test]
    fn test_collect_rust_files_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.rs");
        let file2 = temp_dir.path().join("file2.rs");
        let file3 = temp_dir.path().join("file.txt");

        fs::write(&file1, "fn test1() {}").unwrap();
        fs::write(&file2, "fn test2() {}").unwrap();
        fs::write(&file3, "not rust").unwrap();

        let result = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_check_quality() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let result = check_quality(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_quality_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("clean.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = check_quality(temp_dir.path().to_str().unwrap(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fix_quality_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = fix_quality(temp_dir.path().to_str().unwrap(), true);
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
    fn test_collect_rust_files_non_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "not rust").unwrap();

        let result = collect_rust_files(file_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_check_quality_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad.rs");
        fs::write(&file_path, "fn main() { invalid rust syntax +++").unwrap();

        let result = check_quality(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_quality_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad.rs");
        fs::write(&file_path, "fn main() { invalid rust +++").unwrap();

        let result = fix_quality(temp_dir.path().to_str().unwrap(), false);
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

        let result = fix_quality(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collect_rust_files_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("src").join("nested");
        fs::create_dir_all(&nested_dir).unwrap();

        let file1 = temp_dir.path().join("test.rs");
        let file2 = nested_dir.join("nested.rs");

        fs::write(&file1, "fn test1() {}").unwrap();
        fs::write(&file2, "fn test2() {}").unwrap();

        let result = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_check_quality_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let result = check_quality(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fix_quality_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let result = fix_quality(temp_dir.path().to_str().unwrap(), true);
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

        let result = run_diff(temp_dir.path().to_str().unwrap(), false, false);
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

        let result = run_diff(temp_dir.path().to_str().unwrap(), true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diff_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let result = run_diff(temp_dir.path().to_str().unwrap(), false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_diff_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() { invalid +++").unwrap();

        let result = run_diff(temp_dir.path().to_str().unwrap(), false, false);
        assert!(result.is_err());
    }
}
