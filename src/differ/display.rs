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
    io::{self, Write}
};

use masterror::AppResult;
use owo_colors::OwoColorize;
use terminal_size::{Width, terminal_size};

pub use self::{
    grid::{calculate_columns, render_grid},
    render::render_file_block
};
use super::types::{DiffEntry, DiffResult};
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
    if color {
        println!("\n{}\n", "DIFF SUMMARY".bold());
    } else {
        println!("\nDIFF SUMMARY\n");
    }

    for file in &result.files {
        if color {
            println!("{}:", file.path.cyan().bold());
        } else {
            println!("{}:", file.path);
        }

        let mut analyzer_counts = HashMap::new();
        for entry in &file.entries {
            *analyzer_counts.entry(&entry.analyzer).or_insert(0) += 1;
        }

        for (analyzer, count) in analyzer_counts {
            if color {
                println!(
                    "  {}: {} {}",
                    analyzer.green(),
                    count,
                    if count == 1 { "issue" } else { "issues" }
                );
            } else {
                println!(
                    "  {}: {} {}",
                    analyzer,
                    count,
                    if count == 1 { "issue" } else { "issues" }
                );
            }
        }
        println!();
    }

    let summary = format!(
        "Total: {} changes in {} files",
        result.total_changes(),
        result.total_files()
    );

    if color {
        println!("{}", summary.yellow().bold());
    } else {
        println!("{}", summary);
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
    if color {
        println!("\n{}\n", "DIFF OUTPUT".bold());
    } else {
        println!("\nDIFF OUTPUT\n");
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
            println!("{}\n", layout_info.dimmed());
        } else {
            println!("{}\n", layout_info);
        }
    }

    render_grid(&rendered, columns);

    let summary = format!(
        "Total: {} changes in {} files",
        result.total_changes(),
        result.total_files()
    );

    if color {
        println!("{}", summary.yellow().bold());
    } else {
        println!("{}", summary);
    }
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
/// `AppResult<Vec<DiffEntry>>` - Selected entries for application, or error
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
/// println!("Selected {} changes", selected.len());
/// ```
pub fn show_interactive(result: &DiffResult, color: bool) -> AppResult<Vec<DiffEntry>> {
    let mut selected = Vec::with_capacity(result.total_changes());
    let mut apply_all = false;

    if color {
        println!("\n{}\n", "INTERACTIVE DIFF".bold());
        println!("{}", "Commands: y=yes, n=no, a=all, q=quit\n".dimmed());
    } else {
        println!("\nINTERACTIVE DIFF\n");
        println!("Commands: y=yes, n=no, a=all, q=quit\n");
    }

    for file in &result.files {
        if color {
            println!("{}", format!("File: {}", file.path).cyan().bold());
        } else {
            println!("File: {}", file.path);
        }
        println!();

        for (idx, entry) in file.entries.iter().enumerate() {
            if color {
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
            } else {
                println!("[{}/{}] {}", idx + 1, file.entries.len(), entry.analyzer);
                println!("Line {}:", entry.line);
                println!("- {}", entry.original);

                if let Some(import) = &entry.import {
                    println!("+ {}", import);
                }

                println!("+ {}", entry.modified);
            }
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
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        });

        result.add_file(file);
        show_summary(&result, false);
    }

    #[test]
    fn test_show_full_with_data() {
        let mut result = DiffResult::new();
        let mut file = FileDiff::new("test.rs".to_string());

        file.add_entry(DiffEntry {
            line:        10,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        });

        result.add_file(file);
        show_full(&result, false);
    }
}
