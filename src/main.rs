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
    cli::{Command, QualityArgs},
    error::{IoError, ParseError},
    report::Report
};

mod analyzer;
mod analyzers;
mod cli;
mod error;
mod report;

fn main() -> AppResult<()> {
    let args = QualityArgs::parse_args();

    match args.command {
        Command::Check { path, verbose } => check_quality(&path, verbose)?,
        Command::Fix { path, dry_run } => fix_quality(&path, dry_run)?,
        Command::Format { path } => format_quality(&path)?
    }

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
            println!("✓ {}", file_path.display());
        }
    }

    Ok(())
}

/// Fix quality issues automatically.
///
/// Applies automatic fixes from all analyzers to Rust files in the specified path.
/// Can run in dry-run mode to preview changes without modifying files.
///
/// # Arguments
///
/// * `path` - File or directory path to fix
/// * `dry_run` - If true, report fixes but do not modify files
///
/// # Returns
///
/// `AppResult<()>` - Ok if fixes applied successfully, error on IO or parse failures
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

/// Collect all Rust source files from a path.
///
/// Recursively walks directories to find all `.rs` files. Follows symbolic links.
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

    if path_buf.is_file() && path_buf.extension().map_or(false, |e| e == "rs") {
        files.push(path_buf);
    } else if path_buf.is_dir() {
        for entry in WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_rust_files_empty_dir() {
        let temp_dir = std::env::temp_dir();
        let result = collect_rust_files(temp_dir.to_str().unwrap());
        assert!(result.is_ok());
    }
}
