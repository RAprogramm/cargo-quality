// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::io::{self, BufWriter, Write};

use owo_colors::{OwoColorize, colors::*};

pub fn display_help() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    write_help(&mut out).ok();
    out.flush().ok();
}

/// Writes the full help text into the given writer.
///
/// # Arguments
///
/// * `out` - Destination writer
///
/// # Returns
///
/// `io::Result<()>` - Ok when every line was written
fn write_help(out: &mut impl Write) -> io::Result<()> {
    writeln!(
        out,
        "\n{}",
        "╔══════════════════════════════════════════════════════════════════╗".fg::<Cyan>()
    )?;
    writeln!(
        out,
        "{}",
        "║                    CARGO QUALITY TOOLKIT                         ║"
            .fg::<Cyan>()
            .bold()
    )?;
    writeln!(
        out,
        "{}",
        "║           Professional Rust Code Quality Analysis               ║".fg::<Cyan>()
    )?;
    writeln!(
        out,
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".fg::<Cyan>()
    )?;

    writeln!(out, "\n{}", "COMMANDS".fg::<Yellow>().bold())?;
    writeln!(
        out,
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "check".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} Analyze code quality without modifying files",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXIT:".fg::<Blue>().dimmed(),
        "1 if any issues are found, 0 otherwise (usable as a CI gate)".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "OPTIONS:".fg::<Blue>().dimmed(),
        "--verbose, -v | --analyzer, -a <NAME> | --color, -c".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual check src/".fg::<Cyan>().italic()
    )?;
    writeln!(
        out,
        "             {}",
        "cargo qual check -a inline_comments".fg::<Cyan>().italic()
    )?;
    writeln!(
        out,
        "             {}",
        "cargo qual check --color src/".fg::<Cyan>().italic()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "fix".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} Apply automatic quality fixes",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "OPTIONS:".fg::<Blue>().dimmed(),
        "--dry-run, -d | --analyzer, -a <NAME>".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual fix --dry-run".fg::<Cyan>().italic()
    )?;
    writeln!(
        out,
        "             {}",
        "cargo qual fix -a path_import".fg::<Cyan>().italic()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "format".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} Format code according to quality rules",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual format .".fg::<Cyan>().italic()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "fmt".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} Run cargo +nightly fmt with project config",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Uses hardcoded .rustfmt.toml configuration",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Preserves existing config automatically",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual fmt".fg::<Cyan>().italic()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "diff".fg::<Green>().bold(),
        "[PATH]".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} Show proposed changes before applying",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "OPTIONS:".fg::<Blue>().dimmed(),
        "--summary, -s | --interactive, -i | --analyzer, -a <NAME> | --color, -c".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "MODES:".fg::<Blue>().dimmed(),
        "full (default), summary, interactive".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual diff --summary".fg::<Cyan>().italic()
    )?;
    writeln!(
        out,
        "             {}",
        "cargo qual diff -a path_import".fg::<Cyan>().italic()
    )?;
    writeln!(
        out,
        "             {}",
        "cargo qual diff --color --summary".fg::<Cyan>().italic()
    )?;

    writeln!(out, "\n  {}", "help".fg::<Green>().bold())?;
    writeln!(
        out,
        "    {} Display this beautiful help message",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual help".fg::<Cyan>().italic()
    )?;

    writeln!(out, "\n  {}", "setup".fg::<Green>().bold())?;
    writeln!(
        out,
        "    {} Automatically install shell completions",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "NOTE:".fg::<Blue>().dimmed(),
        "Detects your shell and installs to standard location".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual setup".fg::<Cyan>().italic()
    )?;

    writeln!(out, "\n  {}", "completions".fg::<Green>().bold())?;
    writeln!(
        out,
        "    {} Generate shell completion scripts (manual)",
        "→".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "SHELLS:".fg::<Blue>().dimmed(),
        "bash, fish, zsh, powershell, elvish".fg::<Magenta>()
    )?;
    writeln!(
        out,
        "    {} {}",
        "EXAMPLE:".fg::<Blue>().dimmed(),
        "cargo qual completions fish > ~/.config/fish/completions/cargo.fish"
            .fg::<Cyan>()
            .italic()
    )?;

    writeln!(out, "\n{}", "ANALYZERS".fg::<Yellow>().bold())?;
    writeln!(
        out,
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Path Import Analyzer".fg::<Cyan>().bold()
    )?;
    writeln!(
        out,
        "    {} Detects direct module path usage (e.g., std::fs::read)",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Suggests importing functions instead",
        "•".fg::<Blue>()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Format Args Analyzer".fg::<Cyan>().bold()
    )?;
    writeln!(
        out,
        "    {} Detects positional arguments in format! macros",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Suggests using named arguments for clarity",
        "•".fg::<Blue>()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Empty Lines Analyzer".fg::<Cyan>().bold()
    )?;
    writeln!(
        out,
        "    {} Detects empty lines inside function bodies",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Indicates untamed complexity (code smell)",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Shown as summary note in diff output",
        "•".fg::<Blue>()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "✓".fg::<Green>(),
        "Inline Comments Analyzer".fg::<Cyan>().bold()
    )?;
    writeln!(
        out,
        "    {} Detects inline comments (//) inside function bodies",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Suggests moving to doc block # Notes section with code",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Format: /// - Comment text - `code`",
        "•".fg::<Blue>()
    )?;
    writeln!(
        out,
        "    {} Use: cargo qual check -a inline_comments",
        "•".fg::<Blue>()
    )?;

    writeln!(out, "\n{}", "WORKFLOW".fg::<Yellow>().bold())?;
    writeln!(
        out,
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "1.".fg::<Magenta>().bold(),
        "Check your code".fg::<Green>()
    )?;
    writeln!(
        out,
        "     {}",
        "cargo qual check src/".fg::<Cyan>().italic()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "2.".fg::<Magenta>().bold(),
        "Preview fixes".fg::<Green>()
    )?;
    writeln!(
        out,
        "     {}",
        "cargo qual fix --dry-run".fg::<Cyan>().italic()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "3.".fg::<Magenta>().bold(),
        "Apply fixes".fg::<Green>()
    )?;
    writeln!(out, "     {}", "cargo qual fix".fg::<Cyan>().italic())?;

    writeln!(
        out,
        "\n  {} {}",
        "4.".fg::<Magenta>().bold(),
        "Format code".fg::<Green>()
    )?;
    writeln!(out, "     {}", "cargo qual fmt".fg::<Cyan>().italic())?;

    writeln!(out, "\n{}", "PROJECT INFO".fg::<Yellow>().bold())?;
    writeln!(
        out,
        "{}",
        "────────────────────────────────────────────────────────────────────"
            .fg::<Yellow>()
            .dimmed()
    )?;

    writeln!(
        out,
        "\n  {} {}",
        "Version:".fg::<Blue>(),
        env!("CARGO_PKG_VERSION").fg::<Green>()
    )?;
    writeln!(
        out,
        "  {} {}",
        "Repository:".fg::<Blue>(),
        "https://github.com/RAprogramm/cargo-quality"
            .fg::<Cyan>()
            .underline()
    )?;
    writeln!(out, "  {} {}", "License:".fg::<Blue>(), "MIT".fg::<Green>())?;
    writeln!(
        out,
        "  {} {}",
        "Author:".fg::<Blue>(),
        "RAprogramm".fg::<Magenta>()
    )?;

    writeln!(
        out,
        "\n{}",
        "═══════════════════════════════════════════════════════════════════".fg::<Cyan>()
    )?;
    writeln!(
        out,
        "{}",
        "              Professional Rust Quality Tooling                    "
            .fg::<Cyan>()
            .italic()
    )?;
    writeln!(
        out,
        "{}",
        "═══════════════════════════════════════════════════════════════════\n".fg::<Cyan>()
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_help_no_panic() {
        display_help();
    }
}
