use clap::{Parser, Subcommand};

/// Cargo subcommand for Rust code quality analysis
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub enum CargoCli {
    Quality(QualityArgs)
}

/// Quality analysis and fixes for Rust code
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct QualityArgs {
    #[command(subcommand)]
    pub command: Command
}

/// Available commands
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
    /// Parse CLI arguments
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
