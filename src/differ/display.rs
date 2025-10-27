// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    io::{self, Write}
};

use masterror::AppResult;
use owo_colors::OwoColorize;
use terminal_size::{Width, terminal_size};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::types::{DiffEntry, DiffResult};
use crate::error::IoError;

const NARROW_THRESHOLD: usize = 100;
const WIDE_THRESHOLD: usize = 150;

/// Terminal layout modes based on width.
///
/// Determines how diff output is formatted based on available space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Narrow,
    Medium,
    Wide
}

/// Professional responsive diff formatter.
///
/// Adapts output format based on terminal width for optimal readability.
struct DiffFormatter {
    mode:        LayoutMode,
    term_width:  usize,
    left_width:  usize,
    right_width: usize
}

impl DiffFormatter {
    /// Creates a new formatter with automatic layout detection.
    ///
    /// # Returns
    ///
    /// Configured `DiffFormatter` instance
    fn new() -> Self {
        let term_width = terminal_size()
            .map(|(Width(w), _)| w as usize)
            .unwrap_or(80);

        let mode = match term_width {
            w if w < NARROW_THRESHOLD => LayoutMode::Narrow,
            w if w < WIDE_THRESHOLD => LayoutMode::Medium,
            _ => LayoutMode::Wide
        };

        let (left_width, right_width) = match mode {
            LayoutMode::Narrow => (0, 0),
            LayoutMode::Medium => {
                let usable = term_width.saturating_sub(7);
                let half = usable / 2;
                (half, half)
            }
            LayoutMode::Wide => {
                let usable = term_width.saturating_sub(7);
                let half = usable / 2;
                (half, half)
            }
        };

        Self {
            mode,
            term_width,
            left_width,
            right_width
        }
    }

    /// Truncates text to fit width with ellipsis.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to truncate
    /// * `max_width` - Maximum width in characters
    ///
    /// # Returns
    ///
    /// Truncated string
    fn truncate(&self, text: &str, max_width: usize) -> String {
        let width = text.width();
        if width <= max_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let target = max_width.saturating_sub(ellipsis.len());

        let mut result = String::with_capacity(max_width);
        let mut current_width = 0;

        for ch in text.chars() {
            let ch_width = ch.width().unwrap_or(0);
            if current_width + ch_width > target {
                break;
            }
            result.push(ch);
            current_width += ch_width;
        }

        result.push_str(ellipsis);
        result
    }

    /// Pads text to exact width.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to pad
    /// * `width` - Target width
    ///
    /// # Returns
    ///
    /// Padded string
    fn pad(&self, text: &str, width: usize) -> String {
        let current = text.width();
        if current >= width {
            return text.to_string();
        }

        let padding = width - current;
        format!("{}{}", text, " ".repeat(padding))
    }

    /// Formats diff entry based on layout mode.
    ///
    /// # Arguments
    ///
    /// * `entry` - Diff entry to format
    fn format_entry(&self, entry: &DiffEntry) {
        match self.mode {
            LayoutMode::Narrow => self.format_vertical(entry),
            LayoutMode::Medium | LayoutMode::Wide => self.format_side_by_side(entry)
        }
    }

    /// Formats entry in vertical mode (traditional).
    ///
    /// # Arguments
    ///
    /// * `entry` - Diff entry to format
    fn format_vertical(&self, entry: &DiffEntry) {
        println!("{}", format!("Line {}", entry.line).cyan());
        println!("{}", format!("-    {}", entry.original).red());
        if let Some(import) = &entry.import {
            println!("{}", format!("+    {}", import).green());
        }
        println!("{}", format!("+    {}", entry.modified).green());
        println!();
    }

    /// Formats entry in side-by-side mode.
    ///
    /// # Arguments
    ///
    /// * `entry` - Diff entry to format
    fn format_side_by_side(&self, entry: &DiffEntry) {
        println!("{}", format!("Line {}", entry.line).cyan());

        let left = self.truncate(entry.original.trim(), self.left_width);
        let right_content = if let Some(import) = &entry.import {
            format!("{}\n    {}", import, entry.modified)
        } else {
            entry.modified.clone()
        };
        let right = self.truncate(right_content.trim(), self.right_width);

        let left_padded = self.pad(&left, self.left_width);

        println!(
            "{} {} {}",
            format!("-  {}", left_padded).red(),
            "|".dimmed(),
            format!("+  {}", right).green()
        );
        println!();
    }
}

/// Displays diff in summary mode.
///
/// Shows brief statistics for each file.
///
/// # Arguments
///
/// * `result` - Diff results to display
pub fn show_summary(result: &DiffResult) {
    println!("\n{}\n", "DIFF SUMMARY".bold());

    for file in &result.files {
        println!("{}:", file.path.cyan().bold());

        let mut analyzer_counts = HashMap::new();
        for entry in &file.entries {
            *analyzer_counts.entry(&entry.analyzer).or_insert(0) += 1;
        }

        for (analyzer, count) in analyzer_counts {
            println!(
                "  {}: {} {}",
                analyzer.green(),
                count,
                if count == 1 { "issue" } else { "issues" }
            );
        }
        println!();
    }

    println!(
        "{}",
        format!(
            "Total: {} changes in {} files",
            result.total_changes(),
            result.total_files()
        )
        .yellow()
        .bold()
    );
}

/// Displays full responsive diff output.
///
/// Automatically adapts layout based on terminal width.
///
/// # Arguments
///
/// * `result` - Diff results to display
pub fn show_full(result: &DiffResult) {
    let formatter = DiffFormatter::new();

    println!("\n{}\n", "DIFF OUTPUT".bold());

    if formatter.mode != LayoutMode::Narrow {
        let mode_label = match formatter.mode {
            LayoutMode::Medium => "Medium",
            LayoutMode::Wide => "Wide",
            LayoutMode::Narrow => unreachable!()
        };
        println!(
            "{}\n",
            format!(
                "Layout: {} ({}×{} columns, terminal width: {})",
                mode_label, formatter.left_width, formatter.right_width, formatter.term_width
            )
            .dimmed()
        );
    }

    for file in &result.files {
        println!("{}", format!("File: {}", file.path).cyan().bold());
        println!("{}", "────────────────────────────────────────".dimmed());

        let mut last_analyzer = "";
        for entry in &file.entries {
            if entry.analyzer != last_analyzer {
                if !last_analyzer.is_empty() {
                    println!();
                }
                println!(
                    "{} ({} issues)",
                    entry.analyzer.green().bold(),
                    file.entries
                        .iter()
                        .filter(|e| e.analyzer == entry.analyzer)
                        .count()
                );
                println!();
                last_analyzer = &entry.analyzer;
            }

            formatter.format_entry(entry);
        }

        println!("{}", "════════════════════════════════════════".dimmed());
    }

    println!(
        "\n{}",
        format!(
            "Total: {} changes in {} files",
            result.total_changes(),
            result.total_files()
        )
        .yellow()
        .bold()
    );
}

/// Displays interactive diff with user prompts.
///
/// Shows each change and asks for confirmation.
///
/// # Arguments
///
/// * `result` - Diff results to display
///
/// # Returns
///
/// `AppResult<Vec<DiffEntry>>` - Selected entries or error
pub fn show_interactive(result: &DiffResult) -> AppResult<Vec<DiffEntry>> {
    let mut selected = Vec::with_capacity(result.total_changes());
    let mut apply_all = false;

    println!("\n{}\n", "INTERACTIVE DIFF".bold());
    println!("{}", "Commands: y=yes, n=no, a=all, q=quit\n".dimmed());

    for file in &result.files {
        println!("{}", format!("File: {}", file.path).cyan().bold());
        println!();

        for (idx, entry) in file.entries.iter().enumerate() {
            println!(
                "{} {}",
                format!("[{}/{}]", idx + 1, file.entries.len()).yellow(),
                entry.analyzer.green()
            );
            println!("{}", format!("Line {}:", entry.line).dimmed());
            println!("{}", format!("- {}", entry.original).red());
            if let Some(import) = &entry.import {
                println!("{}", format!("+ {}", import).green());
            }
            println!("{}", format!("+ {}", entry.modified).green());
            println!();

            if apply_all {
                selected.push(entry.clone());
                continue;
            }

            print!("{}", "Apply this fix? [y/n/a/q]: ".bold());
            io::stdout().flush().map_err(IoError::from)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(IoError::from)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    selected.push(entry.clone());
                    println!("{}", "Applied".green());
                }
                "n" | "no" => {
                    println!("{}", "Skipped".yellow());
                }
                "a" | "all" => {
                    apply_all = true;
                    selected.push(entry.clone());
                    println!("{}", "Applying all remaining changes".green().bold());
                }
                "q" | "quit" => {
                    println!("{}", "Quit".red());
                    break;
                }
                _ => {
                    println!("{}", "Invalid input, skipping".red());
                }
            }
            println!();
        }
    }

    println!(
        "\n{}",
        format!("Selected {} changes for application", selected.len())
            .yellow()
            .bold()
    );

    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::differ::types::FileDiff;

    #[test]
    fn test_diff_formatter_new() {
        let formatter = DiffFormatter::new();
        assert!(formatter.term_width > 0);
    }

    #[test]
    fn test_truncate_short_text() {
        let formatter = DiffFormatter::new();
        let text = "short";
        let result = formatter.truncate(text, 10);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_long_text() {
        let formatter = DiffFormatter::new();
        let text = "this is a very long text that should be truncated";
        let result = formatter.truncate(text, 20);
        assert!(result.ends_with("..."));
        assert!(result.width() <= 20);
    }

    #[test]
    fn test_pad_text() {
        let formatter = DiffFormatter::new();
        let text = "hello";
        let result = formatter.pad(text, 10);
        assert_eq!(result.len(), 10);
        assert!(result.starts_with("hello"));
    }

    #[test]
    fn test_show_summary_no_panic() {
        let result = DiffResult::new();
        show_summary(&result);
    }

    #[test]
    fn test_show_full_no_panic() {
        let result = DiffResult::new();
        show_full(&result);
    }

    #[test]
    fn test_show_summary_with_data() {
        let mut result = DiffResult::new();
        let mut file_diff = FileDiff::new("test.rs".to_string());

        let entry1 = DiffEntry {
            line:        1,
            analyzer:    "test_analyzer".to_string(),
            original:    "old line".to_string(),
            modified:    "new line".to_string(),
            description: "test change".to_string(),
            import:      None
        };

        let entry2 = DiffEntry {
            line:        2,
            analyzer:    "test_analyzer".to_string(),
            original:    "old line 2".to_string(),
            modified:    "new line 2".to_string(),
            description: "test change 2".to_string(),
            import:      None
        };

        file_diff.add_entry(entry1);
        file_diff.add_entry(entry2);
        result.add_file(file_diff);

        show_summary(&result);
    }

    #[test]
    fn test_show_full_with_data() {
        let mut result = DiffResult::new();
        let mut file_diff = FileDiff::new("test.rs".to_string());

        let entry = DiffEntry {
            line:        10,
            analyzer:    "format_args".to_string(),
            original:    "println!(\"Hello {}\", name)".to_string(),
            modified:    "println!(\"Hello {name}\")".to_string(),
            description: "Use named arguments".to_string(),
            import:      None
        };

        file_diff.add_entry(entry);
        result.add_file(file_diff);

        show_full(&result);
    }

    #[test]
    fn test_layout_mode_detection() {
        let formatter = DiffFormatter::new();
        assert!(matches!(
            formatter.mode,
            LayoutMode::Narrow | LayoutMode::Medium | LayoutMode::Wide
        ));
    }

    #[test]
    fn test_format_entry_no_panic() {
        let formatter = DiffFormatter::new();
        let entry = DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        };

        formatter.format_entry(&entry);
    }

    #[test]
    fn test_format_entry_with_import() {
        let formatter = DiffFormatter::new();
        let entry = DiffEntry {
            line:        1,
            analyzer:    "path_import".to_string(),
            original:    "std::fs::read_to_string(\"file\")".to_string(),
            modified:    "read_to_string(\"file\")".to_string(),
            description: "Use import".to_string(),
            import:      Some("use std::fs::read_to_string;".to_string())
        };

        formatter.format_entry(&entry);
    }
}
