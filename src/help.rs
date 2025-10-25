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
        "--verbose, -v".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo quality check src/".fg::<Cyan>().italic()
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
        "--dry-run, -d".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo quality fix --dry-run".fg::<Cyan>().italic()
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
        "cargo quality format .".fg::<Cyan>().italic()
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
        "cargo quality fmt".fg::<Cyan>().italic()
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
        "--summary, -s | --interactive, -i".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "MODES:".fg::<Blue>().dimmed(),
        "full (default), summary, interactive".fg::<Magenta>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo quality diff --summary".fg::<Cyan>().italic()
    );

    println!("\n  {}", "help".fg::<Green>().bold());
    println!(
        "    {} Display this beautiful help message",
        "→".fg::<Blue>()
    );
    println!(
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo quality help".fg::<Cyan>().italic()
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
    println!("     {}", "cargo quality check src/".fg::<Cyan>().italic());

    println!(
        "\n  {} {}",
        "2.".fg::<Magenta>().bold(),
        "Preview fixes".fg::<Green>()
    );
    println!(
        "     {}",
        "cargo quality fix --dry-run".fg::<Cyan>().italic()
    );

    println!(
        "\n  {} {}",
        "3.".fg::<Magenta>().bold(),
        "Apply fixes".fg::<Green>()
    );
    println!("     {}", "cargo quality fix".fg::<Cyan>().italic());

    println!(
        "\n  {} {}",
        "4.".fg::<Magenta>().bold(),
        "Format code".fg::<Green>()
    );
    println!("     {}", "cargo quality fmt".fg::<Cyan>().italic());

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
