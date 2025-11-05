// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use owo_colors::{OwoColorize, colors::*};

pub fn display_help() {
    println!(
        "\n{}",
        "╔══════════════════════════════════════════════════════════════════╗".fg::<Cyan>()
    );
    println!(
        "{}",
        "║                    CARGO QUALITY TOOLKIT                         ║"
            .fg::<Cyan>()
            .bold()
    );
    println!(
        "{}",
        "║           Professional Rust Code Quality Analysis               ║".fg::<Cyan>()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".fg::<Cyan>()
    );

    println!("\n{}", "COMMANDS".fg::<Yellow>().bold());
    println!(
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    );

    println!(
        "\n  {} {}",
        "check".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    );
    println!(
        "    {} Analyze code quality without modifying files",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "OPTIONS:".fg::<Blue>().dimmed(),
        "--verbose, -v | --analyzer, -a <NAME> | --color, -c".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual check src/".fg::<Cyan>().italic()
    );
    println!(
        "             {}",
        "cargo qual check -a inline_comments".fg::<Cyan>().italic()
    );
    println!(
        "             {}",
        "cargo qual check --color src/".fg::<Cyan>().italic()
    );

    println!(
        "\n  {} {}",
        "fix".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    );
    println!("    {} Apply automatic quality fixes", "→".fg::<Blue>());
    println!(
        "    {} {}",
        "OPTIONS:".fg::<Blue>().dimmed(),
        "--dry-run, -d | --analyzer, -a <NAME>".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual fix --dry-run".fg::<Cyan>().italic()
    );
    println!(
        "             {}",
        "cargo qual fix -a path_import".fg::<Cyan>().italic()
    );

    println!(
        "\n  {} {}",
        "format".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    );
    println!(
        "    {} Format code according to quality rules",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual format .".fg::<Cyan>().italic()
    );

    println!(
        "\n  {} {}",
        "fmt".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    );
    println!(
        "    {} Run cargo +nightly fmt with project config",
        "→".fg::<Blue>()
    );
    println!(
        "    {} Uses hardcoded .rustfmt.toml configuration",
        "→".fg::<Blue>()
    );
    println!(
        "    {} Preserves existing config automatically",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual fmt".fg::<Cyan>().italic()
    );

    println!(
        "\n  {} {}",
        "diff".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    );
    println!(
        "    {} Show proposed changes before applying",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "OPTIONS:".fg::<Blue>().dimmed(),
        "--summary, -s | --interactive, -i | --analyzer, -a <NAME> | --color, -c"
            .fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "MODES:".fg::<Blue>().dimmed(),
        "full (default), summary, interactive".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual diff --summary".fg::<Cyan>().italic()
    );
    println!(
        "             {}",
        "cargo qual diff -a path_import".fg::<Cyan>().italic()
    );
    println!(
        "             {}",
        "cargo qual diff --color --summary".fg::<Cyan>().italic()
    );

    println!("\n  {}", "help".fg::<Green>().bold());
    println!(
        "    {} Display this beautiful help message",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual help".fg::<Cyan>().italic()
    );

    println!("\n  {}", "setup".fg::<Green>().bold());
    println!(
        "    {} Automatically install shell completions",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "NOTE:".fg::<Blue>().dimmed(),
        "Detects your shell and installs to standard location".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual setup".fg::<Cyan>().italic()
    );

    println!("\n  {}", "completions".fg::<Green>().bold());
    println!(
        "    {} Generate shell completion scripts (manual)",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "SHELLS:".fg::<Blue>().dimmed(),
        "bash, fish, zsh, powershell, elvish".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual completions fish > ~/.config/fish/completions/cargo.fish"
            .fg::<Cyan>()
            .italic()
    );

    println!("\n{}", "ANALYZERS".fg::<Yellow>().bold());
    println!(
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    );

    println!(
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Path Import Analyzer".fg::<Cyan>().bold()
    );
    println!(
        "    {} Detects direct module path usage (e.g., std::fs::read)",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Suggests importing functions instead",
        "•".fg::<Blue>()
    );

    println!(
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Format Args Analyzer".fg::<Cyan>().bold()
    );
    println!(
        "    {} Detects positional arguments in format! macros",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Suggests using named arguments for clarity",
        "•".fg::<Blue>()
    );

    println!(
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Empty Lines Analyzer".fg::<Cyan>().bold()
    );
    println!(
        "    {} Detects empty lines inside function bodies",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Indicates untamed complexity (code smell)",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Shown as summary note in diff output",
        "•".fg::<Blue>()
    );

    println!(
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Inline Comments Analyzer".fg::<Cyan>().bold()
    );
    println!(
        "    {} Detects inline comments (//) inside function bodies",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Suggests moving to doc block # Notes section with code",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Format: /// - Comment text - `code`",
        "•".fg::<Blue>()
    );
    println!(
        "    {} Use: cargo qual check -a inline_comments",
        "•".fg::<Blue>()
    );

    println!("\n{}", "WORKFLOW".fg::<Yellow>().bold());
    println!(
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    );

    println!(
        "\n  {} {}",
        "1.".fg::<Magenta>().bold(),
        "Check your code".fg::<Green>()
    );
    println!("     {}", "cargo qual check src/".fg::<Cyan>().italic());

    println!(
        "\n  {} {}",
        "2.".fg::<Magenta>().bold(),
        "Preview fixes".fg::<Green>()
    );
    println!("     {}", "cargo qual fix --dry-run".fg::<Cyan>().italic());

    println!(
        "\n  {} {}",
        "3.".fg::<Magenta>().bold(),
        "Apply fixes".fg::<Green>()
    );
    println!("     {}", "cargo qual fix".fg::<Cyan>().italic());

    println!(
        "\n  {} {}",
        "4.".fg::<Magenta>().bold(),
        "Format code".fg::<Green>()
    );
    println!("     {}", "cargo qual fmt".fg::<Cyan>().italic());

    println!("\n{}", "PROJECT INFO".fg::<Yellow>().bold());
    println!(
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    );

    println!(
        "\n  {} {}",
        "Version:".fg::<Blue>(),
        env!("CARGO_PKG_VERSION").fg::<Green>()
    );
    println!(
        "  {} {}",
        "Repository:".fg::<Blue>(),
        "https://github.com/RAprogramm/cargo-quality"
            .fg::<Cyan>()
            .underline()
    );
    println!("  {} {}", "License:".fg::<Blue>(), "MIT".fg::<Green>());
    println!(
        "  {} {}",
        "Author:".fg::<Blue>(),
        "RAprogramm".fg::<Magenta>()
    );

    println!(
        "\n{}",
        "═══════════════════════════════════════════════════════════════════".fg::<Cyan>()
    );
    println!(
        "{}",
        "              Professional Rust Quality Tooling                    "
            .fg::<Cyan>()
            .italic()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════\n".fg::<Cyan>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_help_no_panic() {
        display_help();
    }
}
