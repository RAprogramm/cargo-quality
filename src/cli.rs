//! Command-line interface definitions using clap.
//!
//! Defines the CLI structure for cargo-quality with support for check, fix,
//! and format subcommands. Uses clap derive macros for argument parsing.

use clap::{Parser, Subcommand};

/// Cargo subcommand for Rust code quality analysis.
///
/// Top-level command that wraps the quality subcommand.
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    /// Quality analysis subcommand
    Quality(QualityArgs)
}

/// Quality analysis and fixes for Rust code.
///
/// Main argument structure containing the subcommand to execute.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct QualityArgs {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Command
}

/// Available quality analysis commands.
///
/// Each variant represents a different operation mode:
/// - Check: Report issues without modifications
/// - Fix: Apply automatic fixes
/// - Format: Format code according to rules
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check code quality without modifying files
    Check {
        /// Path to analyze (default: current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Show detailed output
        #[arg(short, long)]
        verbose: bool
    },

    /// Automatically fix quality issues
    Fix {
        /// Path to analyze (default: current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Dry run - show changes without applying
        #[arg(short, long)]
        dry_run: bool
    },

    /// Format code according to quality rules
    Format {
        /// Path to analyze (default: current directory)
        #[arg(default_value = ".")]
        path: String
    }
}

impl QualityArgs {
    /// Parse command-line arguments.
    ///
    /// Extracts quality subcommand arguments from cargo invocation.
    ///
    /// # Returns
    ///
    /// Parsed `QualityArgs` with selected subcommand
    pub fn parse_args() -> Self {
        let CargoCli::Quality(args) = CargoCli::parse();
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_check() {
        let args = CargoCli::parse_from(["cargo", "quality", "check", "src"]);
        if let CargoCli::Quality(quality) = args {
            match quality.command {
                Command::Check { path, verbose } => {
                    assert_eq!(path, "src");
                    assert!(!verbose);
                }
                _ => panic!("Expected Check command")
            }
        }
    }

    #[test]
    fn test_cli_parsing_fix() {
        let args = CargoCli::parse_from(["cargo", "quality", "fix", "--dry-run"]);
        if let CargoCli::Quality(quality) = args {
            match quality.command {
                Command::Fix { path, dry_run } => {
                    assert_eq!(path, ".");
                    assert!(dry_run);
                }
                _ => panic!("Expected Fix command")
            }
        }
    }

    #[test]
    fn test_cli_parsing_format() {
        let args = CargoCli::parse_from(["cargo", "quality", "format"]);
        if let CargoCli::Quality(quality) = args {
            match quality.command {
                Command::Format { path } => {
                    assert_eq!(path, ".");
                }
                _ => panic!("Expected Format command")
            }
        }
    }
}
