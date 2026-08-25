// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Professional responsive diff display with grid layout.
//!
//! This module provides a sophisticated diff visualization system that adapts
//! to terminal width, offering newspaper-style column layouts for optimal
//! screen space utilization. Features include intelligent import grouping,
//! ANSI-aware text measurement, and zero-allocation rendering paths.
//!
//! # Architecture
//!
//! The display system is organized into specialized modules:
//!
//! - `types` - Core data structures for rendered output
//! - `formatting` - Text padding and width calculation
//! - `grouping` - Import deduplication and intelligent grouping
//! - `grid` - Responsive column layout calculations
//! - `render` - File diff block rendering
//!
//! # Performance
//!
//! - Pre-allocated vectors with estimated capacities
//! - Single-pass width calculations
//! - ANSI-aware measurements using `console` crate
//! - Minimal string allocations
//! - Zero-cost abstractions for layout logic
//!
//! # Examples
//!
//! ```no_run
//! use cargo_quality::differ::{DiffResult, display::show_full};
//!
//! let result = DiffResult::new();
//! show_full(&result, false);
//! ```

pub mod formatting;
pub mod grid;
pub mod grouping;
pub mod render;
pub mod types;

// Re-export key types and functions for public API
use std::{
    collections::HashMap,
    io::{self, BufWriter, Write}
};

use masterror::AppResult;
use owo_colors::OwoColorize;
use terminal_size::{Width, terminal_size};

pub use self::{
    grid::{calculate_columns, render_grid},
    render::render_file_block
};
use super::types::{DiffEntry, DiffResult, FileDiff};
use crate::error::IoError;

/// Displays diff in summary mode with brief statistics.
///
/// Shows a compact overview of changes grouped by file and analyzer,
/// providing quick insight into the scope of modifications without
/// showing detailed line-by-line changes.
///
/// # Output Format
///
/// ```text
/// DIFF SUMMARY
///
/// file1.rs:
///   analyzer1: 3 issues
///   analyzer2: 1 issue
///
/// file2.rs:
///   analyzer1: 2 issues
///
/// Total: 6 changes in 2 files
/// ```
///
/// # Arguments
///
/// * `result` - Diff results to display
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::differ::{DiffResult, display::show_summary};
///
/// let result = DiffResult::new();
/// show_summary(&result, false);
/// ```
pub fn show_summary(result: &DiffResult, color: bool) {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    write_summary(&mut out, result, color).ok();
    out.flush().ok();
}

/// Writes the diff summary into the given writer.
///
/// # Arguments
///
/// * `out` - Destination writer
/// * `result` - Diff results to render
/// * `color` - Enable colored output
///
/// # Returns
///
/// `io::Result<()>` - Ok when every line was written
fn write_summary(out: &mut impl Write, result: &DiffResult, color: bool) -> io::Result<()> {
    if color {
        writeln!(out, "\n{}\n", "DIFF SUMMARY".bold())?;
    } else {
        out.write_all(b"\nDIFF SUMMARY\n\n")?;
    }

    for file in &result.files {
        if color {
            writeln!(out, "{}:", file.path.cyan().bold())?;
        } else {
            writeln!(out, "{}:", file.path)?;
        }

        let mut analyzer_counts = HashMap::new();
        for entry in &file.entries {
            *analyzer_counts.entry(&entry.analyzer).or_insert(0) += 1;
        }

        for (analyzer, count) in analyzer_counts {
            let noun = if count == 1 { "issue" } else { "issues" };
            if color {
                writeln!(out, "  {}: {} {}", analyzer.green(), count, noun)?;
            } else {
                writeln!(out, "  {}: {} {}", analyzer, count, noun)?;
            }
        }
        out.write_all(b"\n")?;
    }

    write_totals(out, result, color)
}

/// Writes the closing totals line shared by the summary and full views.
///
/// # Arguments
///
/// * `out` - Destination writer
/// * `result` - Diff results to summarize
/// * `color` - Enable colored output
///
/// # Returns
///
/// `io::Result<()>` - Ok when the line was written
fn write_totals(out: &mut impl Write, result: &DiffResult, color: bool) -> io::Result<()> {
    let summary = format!(
        "Total: {} changes in {} files",
        result.total_changes(),
        result.total_files()
    );

    if color {
        writeln!(out, "{}", summary.yellow().bold())
    } else {
        writeln!(out, "{}", summary)
    }
}

/// Displays full responsive diff output with adaptive grid layout.
///
/// Automatically arranges file diffs in newspaper-style columns based on
/// terminal width. On narrow terminals, displays one file per row. On wider
/// terminals, arranges multiple files side-by-side for efficient space usage.
///
/// # Layout Modes
///
/// - **Narrow** (< 100 chars): Single column, vertical stacking
/// - **Medium** (100-200 chars): 2 columns side-by-side
/// - **Wide** (> 200 chars): 3+ columns based on content width
///
/// # Arguments
///
/// * `result` - Diff results to display
///
/// # Performance
///
/// - Pre-renders all files once
/// - Calculates optimal column count based on terminal width
/// - Uses ANSI-aware padding for perfect alignment
/// - Minimal allocations during grid rendering
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::differ::{DiffResult, display::show_full};
///
/// let result = DiffResult::new();
/// show_full(&result, false);
/// ```
pub fn show_full(result: &DiffResult, color: bool) {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if color {
        writeln!(out, "\n{}\n", "DIFF OUTPUT".bold()).ok();
    } else {
        out.write_all(b"\nDIFF OUTPUT\n\n").ok();
    }

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let rendered: Vec<_> = result
        .files
        .iter()
        .map(|f| render_file_block(f, color))
        .collect();

    let columns = calculate_columns(&rendered, term_width);

    if columns > 1 {
        let layout_info = format!(
            "Layout: {} columns (terminal width: {})",
            columns, term_width
        );

        if color {
            writeln!(out, "{}\n", layout_info.dimmed()).ok();
        } else {
            writeln!(out, "{}\n", layout_info).ok();
        }
    }

    out.flush().ok();
    render_grid(&rendered, columns);

    write_totals(&mut out, result, color).ok();
    out.flush().ok();
}

/// Displays interactive diff with user prompts for selective application.
///
/// Presents each change individually and asks for user confirmation before
/// applying. Supports batch operations (apply all, quit) for efficiency.
///
/// # Commands
///
/// - `y` / `yes` - Apply this change
/// - `n` / `no` - Skip this change
/// - `a` / `all` - Apply all remaining changes
/// - `q` / `quit` - Exit without processing remaining changes
///
/// # Arguments
///
/// * `result` - Diff results to display
///
/// # Returns
///
/// `AppResult<DiffResult>` - Selected entries grouped by file, or error
///
/// # Errors
///
/// Returns error if I/O operations fail during user input reading.
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::differ::{DiffResult, display::show_interactive};
///
/// let result = DiffResult::new();
/// let selected = show_interactive(&result, false).unwrap();
/// println!("Selected {} changes", selected.total_changes());
/// ```
pub fn show_interactive(result: &DiffResult, color: bool) -> AppResult<DiffResult> {
    let mut selected = DiffResult::new();
    let mut apply_all = false;

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if color {
        writeln!(out, "\n{}\n", "INTERACTIVE DIFF".bold()).map_err(IoError::from)?;
        writeln!(out, "{}", "Commands: y=yes, n=no, a=all, q=quit\n".dimmed())
            .map_err(IoError::from)?;
    } else {
        out.write_all(b"\nINTERACTIVE DIFF\n\nCommands: y=yes, n=no, a=all, q=quit\n\n")
            .map_err(IoError::from)?;
    }

    for file in &result.files {
        if color {
            writeln!(out, "{}", format!("File: {}", file.path).cyan().bold())
                .map_err(IoError::from)?;
        } else {
            writeln!(out, "File: {}", file.path).map_err(IoError::from)?;
        }
        out.write_all(b"\n").map_err(IoError::from)?;

        let mut file_selected = FileDiff::new(file.path.clone());

        for (idx, entry) in file.entries.iter().enumerate() {
            write_entry(&mut out, idx, file.entries.len(), entry, color).map_err(IoError::from)?;

            if apply_all {
                file_selected.add_entry(entry.clone());
                continue;
            }

            write!(out, "{}", "Apply this fix? [y/n/a/q]: ".bold()).map_err(IoError::from)?;
            out.flush().map_err(IoError::from)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(IoError::from)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    file_selected.add_entry(entry.clone());
                    writeln!(out, "{}", "Applied".green()).map_err(IoError::from)?;
                }
                "n" | "no" => {
                    writeln!(out, "{}", "Skipped".yellow()).map_err(IoError::from)?;
                }
                "a" | "all" => {
                    apply_all = true;
                    file_selected.add_entry(entry.clone());
                    writeln!(out, "{}", "Applying all remaining changes".green().bold())
                        .map_err(IoError::from)?;
                }
                "q" | "quit" => {
                    writeln!(out, "{}", "Quit".red()).map_err(IoError::from)?;
                    selected.add_file(file_selected);
                    write_selected_total(&mut out, &selected).map_err(IoError::from)?;
                    out.flush().map_err(IoError::from)?;
                    return Ok(selected);
                }
                _ => {
                    writeln!(out, "{}", "Invalid input, skipping".red()).map_err(IoError::from)?;
                }
            }
            out.write_all(b"\n").map_err(IoError::from)?;
        }

        selected.add_file(file_selected);
    }

    write_selected_total(&mut out, &selected).map_err(IoError::from)?;
    out.flush().map_err(IoError::from)?;

    Ok(selected)
}

/// Writes one interactive diff entry block.
///
/// # Arguments
///
/// * `out` - Destination writer
/// * `idx` - Zero-based entry index within the file
/// * `total` - Total entries in the file
/// * `entry` - Entry to render
/// * `color` - Enable colored output
///
/// # Returns
///
/// `io::Result<()>` - Ok when every line was written
fn write_entry(
    out: &mut impl Write,
    idx: usize,
    total: usize,
    entry: &DiffEntry,
    color: bool
) -> io::Result<()> {
    if color {
        writeln!(
            out,
            "{} {}",
            format!("[{}/{}]", idx + 1, total).yellow(),
            entry.analyzer.green()
        )?;
        writeln!(out, "{}", format!("Line {}:", entry.line).dimmed())?;
        writeln!(out, "{}", format!("- {}", entry.preview.original).red())?;

        if let Some(import) = &entry.suggestion.import {
            writeln!(out, "{}", format!("+ {}", import.statement).green())?;
        }

        writeln!(out, "{}", format!("+ {}", entry.preview.modified).green())?;
    } else {
        writeln!(out, "[{}/{}] {}", idx + 1, total, entry.analyzer)?;
        writeln!(out, "Line {}:", entry.line)?;
        writeln!(out, "- {}", entry.preview.original)?;

        if let Some(import) = &entry.suggestion.import {
            writeln!(out, "+ {}", import.statement)?;
        }

        writeln!(out, "+ {}", entry.preview.modified)?;
    }
    out.write_all(b"\n")
}

/// Writes the closing "selected changes" line of the interactive flow.
///
/// # Arguments
///
/// * `out` - Destination writer
/// * `selected` - Accumulated selection to report
///
/// # Returns
///
/// `io::Result<()>` - Ok when the line was written
fn write_selected_total(out: &mut impl Write, selected: &DiffResult) -> io::Result<()> {
    writeln!(
        out,
        "\n{}",
        format!(
            "Selected {} changes for application",
            selected.total_changes()
        )
        .yellow()
        .bold()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analyzer::{Suggestion, TextEdit},
        differ::types::{ChangePreview, DiffEntry}
    };

    #[test]
    fn test_show_summary_empty() {
        let result = DiffResult::new();
        show_summary(&result, false);
    }

    #[test]
    fn test_show_full_empty() {
        let result = DiffResult::new();
        show_full(&result, false);
    }

    #[test]
    fn test_show_summary_with_data() {
        let mut result = DiffResult::new();
        let mut file = FileDiff::new("test.rs".to_string());

        file.add_entry(DiffEntry {
            line:       1,
            analyzer:   "test".to_string(),
            preview:    ChangePreview {
                original:    "old".to_string(),
                modified:    "new".to_string(),
                description: "desc".to_string()
            },
            suggestion: Suggestion {
                edit:   TextEdit::default(),
                import: None
            }
        });

        result.add_file(file);
        show_summary(&result, false);
    }

    #[test]
    fn test_show_full_with_data() {
        let mut result = DiffResult::new();
        let mut file = FileDiff::new("test.rs".to_string());

        file.add_entry(DiffEntry {
            line:       10,
            analyzer:   "test".to_string(),
            preview:    ChangePreview {
                original:    "old".to_string(),
                modified:    "new".to_string(),
                description: "desc".to_string()
            },
            suggestion: Suggestion {
                edit:   TextEdit::default(),
                import: None
            }
        });

        result.add_file(file);
        show_full(&result, false);
    }
}
