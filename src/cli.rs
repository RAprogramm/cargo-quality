// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

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
#[command(disable_help_flag = true, disable_help_subcommand = true)]
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
    },

    /// Format code using cargo +nightly fmt with project configuration
    Fmt {
        /// Path to format (default: current directory)
        #[arg(default_value = ".")]
        path: String
    },

    /// Show diff of proposed changes before applying
    Diff {
        /// Path to analyze (default: current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Show brief summary only
        #[arg(short, long)]
        summary: bool,

        /// Interactive mode - select changes to apply
        #[arg(short, long)]
        interactive: bool
    },

    /// Display beautiful help with examples and usage
    Help
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

    /// Parse from iterator (for testing).
    ///
    /// # Arguments
    ///
    /// * `iter` - Iterator over argument strings
    ///
    /// # Returns
    ///
    /// Parsed `QualityArgs` with selected subcommand
    #[cfg(test)]
    pub fn parse_from_iter<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone
    {
        let CargoCli::Quality(args) = CargoCli::parse_from(iter);
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_check() {
        let args = CargoCli::parse_from(["cargo", "quality", "check", "src"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Check {
                path,
                verbose
            } => {
                assert_eq!(path, "src");
                assert!(!verbose);
            }
            _ => panic!("Expected Check command")
        }
    }

    #[test]
    fn test_cli_parsing_fix() {
        let args = CargoCli::parse_from(["cargo", "quality", "fix", "--dry-run"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Fix {
                path,
                dry_run
            } => {
                assert_eq!(path, ".");
                assert!(dry_run);
            }
            _ => panic!("Expected Fix command")
        }
    }

    #[test]
    fn test_cli_parsing_format() {
        let args = CargoCli::parse_from(["cargo", "quality", "format"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Format {
                path
            } => {
                assert_eq!(path, ".");
            }
            _ => panic!("Expected Format command")
        }
    }

    #[test]
    fn test_cli_parsing_check_verbose() {
        let args = CargoCli::parse_from(["cargo", "quality", "check", "--verbose"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Check {
                path,
                verbose
            } => {
                assert_eq!(path, ".");
                assert!(verbose);
            }
            _ => panic!("Expected Check command")
        }
    }

    #[test]
    fn test_cli_parsing_fix_no_dry_run() {
        let args = CargoCli::parse_from(["cargo", "quality", "fix"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Fix {
                path,
                dry_run
            } => {
                assert_eq!(path, ".");
                assert!(!dry_run);
            }
            _ => panic!("Expected Fix command")
        }
    }

    #[test]
    fn test_cli_parsing_format_with_path() {
        let args = CargoCli::parse_from(["cargo", "quality", "format", "src/"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Format {
                path
            } => {
                assert_eq!(path, "src/");
            }
            _ => panic!("Expected Format command")
        }
    }

    #[test]
    fn test_cli_parsing_fmt() {
        let args = CargoCli::parse_from(["cargo", "quality", "fmt"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Fmt {
                path
            } => {
                assert_eq!(path, ".");
            }
            _ => panic!("Expected Fmt command")
        }
    }

    #[test]
    fn test_cli_parsing_fmt_with_path() {
        let args = CargoCli::parse_from(["cargo", "quality", "fmt", "src/"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Fmt {
                path
            } => {
                assert_eq!(path, "src/");
            }
            _ => panic!("Expected Fmt command")
        }
    }

    #[test]
    fn test_cli_parsing_help() {
        let args = CargoCli::parse_from(["cargo", "quality", "help"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Help => {}
            _ => panic!("Expected Help command")
        }
    }

    #[test]
    fn test_cli_parsing_diff() {
        let args = CargoCli::parse_from(["cargo", "quality", "diff"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive
            } => {
                assert_eq!(path, ".");
                assert!(!summary);
                assert!(!interactive);
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_cli_parsing_diff_summary() {
        let args = CargoCli::parse_from(["cargo", "quality", "diff", "--summary"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive
            } => {
                assert_eq!(path, ".");
                assert!(summary);
                assert!(!interactive);
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_cli_parsing_diff_interactive() {
        let args = CargoCli::parse_from(["cargo", "quality", "diff", "--interactive"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive
            } => {
                assert_eq!(path, ".");
                assert!(!summary);
                assert!(interactive);
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_cli_parsing_diff_with_path() {
        let args = CargoCli::parse_from(["cargo", "quality", "diff", "src/"]);
        let CargoCli::Quality(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive
            } => {
                assert_eq!(path, "src/");
                assert!(!summary);
                assert!(!interactive);
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_quality_args_parse_from_iter() {
        let args = QualityArgs::parse_from_iter(["cargo", "quality", "check", "--verbose"]);
        match args.command {
            Command::Check {
                path,
                verbose
            } => {
                assert_eq!(path, ".");
                assert!(verbose);
            }
            _ => panic!("Expected Check command")
        }
    }
}
