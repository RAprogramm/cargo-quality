// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Command-line interface definitions using clap.
//!
//! Defines the CLI structure for cargo-quality with support for check, fix,
//! and format subcommands. Uses clap derive macros for argument parsing.

use clap::{Parser, Subcommand};

/// Cargo subcommand for Rust code quality analysis.
///
/// Top-level command that wraps the qual subcommand.
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    /// Quality analysis subcommand
    Qual(QualityArgs)
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
        verbose: bool,

        /// Run specific analyzer only (e.g., inline_comments, empty_lines)
        #[arg(short, long)]
        analyzer: Option<String>,

        /// Enable colored output
        #[arg(short, long)]
        color: bool
    },

    /// Automatically fix quality issues
    Fix {
        /// Path to analyze (default: current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Dry run - show changes without applying
        #[arg(short, long)]
        dry_run: bool,

        /// Run specific analyzer only (e.g., inline_comments, empty_lines)
        #[arg(short, long)]
        analyzer: Option<String>
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
        interactive: bool,

        /// Run specific analyzer only (e.g., inline_comments, empty_lines)
        #[arg(short, long)]
        analyzer: Option<String>
    },

    /// Display beautiful help with examples and usage
    Help,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell
    },

    /// Setup shell completions automatically
    Setup
}

/// Supported shells for completion generation
#[derive(Debug, Clone, clap::ValueEnum)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
    PowerShell,
    Elvish
}

impl QualityArgs {
    /// Parse command-line arguments.
    ///
    /// Extracts qual subcommand arguments from cargo invocation.
    ///
    /// # Returns
    ///
    /// Parsed `QualityArgs` with selected subcommand
    pub fn parse_args() -> Self {
        let CargoCli::Qual(args) = CargoCli::parse();
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
        let CargoCli::Qual(args) = CargoCli::parse_from(iter);
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_check() {
        let args = CargoCli::parse_from(["cargo", "qual", "check", "src"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Check {
                path,
                verbose,
                analyzer,
                color
            } => {
                assert_eq!(path, "src");
                assert!(!verbose);
                assert!(analyzer.is_none());
                assert!(!color);
            }
            _ => panic!("Expected Check command")
        }
    }

    #[test]
    fn test_cli_parsing_fix() {
        let args = CargoCli::parse_from(["cargo", "qual", "fix", "--dry-run"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Fix {
                path,
                dry_run,
                analyzer
            } => {
                assert_eq!(path, ".");
                assert!(dry_run);
                assert!(analyzer.is_none());
            }
            _ => panic!("Expected Fix command")
        }
    }

    #[test]
    fn test_cli_parsing_format() {
        let args = CargoCli::parse_from(["cargo", "qual", "format"]);
        let CargoCli::Qual(quality) = args;
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
        let args = CargoCli::parse_from(["cargo", "qual", "check", "--verbose"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Check {
                path,
                verbose,
                analyzer,
                color
            } => {
                assert_eq!(path, ".");
                assert!(verbose);
                assert!(analyzer.is_none());
                assert!(!color);
            }
            _ => panic!("Expected Check command")
        }
    }

    #[test]
    fn test_cli_parsing_fix_no_dry_run() {
        let args = CargoCli::parse_from(["cargo", "qual", "fix"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Fix {
                path,
                dry_run,
                analyzer
            } => {
                assert_eq!(path, ".");
                assert!(!dry_run);
                assert!(analyzer.is_none());
            }
            _ => panic!("Expected Fix command")
        }
    }

    #[test]
    fn test_cli_parsing_format_with_path() {
        let args = CargoCli::parse_from(["cargo", "qual", "format", "src/"]);
        let CargoCli::Qual(quality) = args;
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
        let args = CargoCli::parse_from(["cargo", "qual", "fmt"]);
        let CargoCli::Qual(quality) = args;
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
        let args = CargoCli::parse_from(["cargo", "qual", "fmt", "src/"]);
        let CargoCli::Qual(quality) = args;
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
        let args = CargoCli::parse_from(["cargo", "qual", "help"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Help => {}
            _ => panic!("Expected Help command")
        }
    }

    #[test]
    fn test_cli_parsing_diff() {
        let args = CargoCli::parse_from(["cargo", "qual", "diff"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive,
                analyzer
            } => {
                assert_eq!(path, ".");
                assert!(!summary);
                assert!(!interactive);
                assert!(analyzer.is_none());
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_cli_parsing_diff_summary() {
        let args = CargoCli::parse_from(["cargo", "qual", "diff", "--summary"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive,
                analyzer
            } => {
                assert_eq!(path, ".");
                assert!(summary);
                assert!(!interactive);
                assert!(analyzer.is_none());
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_cli_parsing_diff_interactive() {
        let args = CargoCli::parse_from(["cargo", "qual", "diff", "--interactive"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive,
                analyzer
            } => {
                assert_eq!(path, ".");
                assert!(!summary);
                assert!(interactive);
                assert!(analyzer.is_none());
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_cli_parsing_diff_with_path() {
        let args = CargoCli::parse_from(["cargo", "qual", "diff", "src/"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Diff {
                path,
                summary,
                interactive,
                analyzer
            } => {
                assert_eq!(path, "src/");
                assert!(!summary);
                assert!(!interactive);
                assert!(analyzer.is_none());
            }
            _ => panic!("Expected Diff command")
        }
    }

    #[test]
    fn test_quality_args_parse_from_iter() {
        let args = QualityArgs::parse_from_iter(["cargo", "qual", "check", "--verbose"]);
        match args.command {
            Command::Check {
                path,
                verbose,
                analyzer,
                color
            } => {
                assert_eq!(path, ".");
                assert!(verbose);
                assert!(analyzer.is_none());
                assert!(!color);
            }
            _ => panic!("Expected Check command")
        }
    }

    #[test]
    fn test_cli_parsing_completions() {
        let args = CargoCli::parse_from(["cargo", "qual", "completions", "fish"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Completions {
                shell
            } => match shell {
                Shell::Fish => {}
                _ => panic!("Expected Fish shell")
            },
            _ => panic!("Expected Completions command")
        }
    }

    #[test]
    fn test_cli_parsing_check_with_analyzer() {
        let args =
            CargoCli::parse_from(["cargo", "qual", "check", "--analyzer", "inline_comments"]);
        let CargoCli::Qual(quality) = args;
        match quality.command {
            Command::Check {
                path,
                verbose,
                analyzer,
                color
            } => {
                assert_eq!(path, ".");
                assert!(!verbose);
                assert_eq!(analyzer, Some("inline_comments".to_string()));
                assert!(!color);
            }
            _ => panic!("Expected Check command")
        }
    }
}
