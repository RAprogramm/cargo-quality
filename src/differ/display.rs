// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{self, Write}
};

use console::measure_text_width;
use masterror::AppResult;
use owo_colors::OwoColorize;
use terminal_size::{Width, terminal_size};

use super::types::{DiffEntry, DiffResult, FileDiff};
use crate::error::IoError;

const COLUMN_GAP: usize = 4;
const MIN_FILE_WIDTH: usize = 40;

/// Pre-rendered file diff block.
///
/// Contains all lines of output for a single file.
#[derive(Debug, Clone)]
struct RenderedFile {
    lines: Vec<String>,
    width: usize
}

/// Groups imports by root module and formats them.
///
/// # Arguments
///
/// * `imports` - List of import statements
///
/// # Returns
///
/// Formatted and grouped import statements
fn group_imports(imports: &[&str]) -> Vec<String> {
    let unique: HashSet<&str> = imports.iter().copied().collect();

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for import in unique {
        let import_str = import
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .trim();

        if let Some(double_colon_pos) = import_str.find("::") {
            let root = import_str[..double_colon_pos].to_string();
            let path = import_str[double_colon_pos + 2..].to_string();
            grouped.entry(root).or_default().push(path);
        } else {
            grouped
                .entry(import_str.to_string())
                .or_default()
                .push(String::new());
        }
    }

    let mut result = Vec::new();
    for (root, mut paths) in grouped {
        if paths.len() == 1 && !paths[0].is_empty() {
            result.push(format!("use {}::{};", root, paths[0]));
        } else if paths.len() == 1 && paths[0].is_empty() {
            result.push(format!("use {};", root));
        } else {
            paths.sort();

            let common_prefix = find_common_prefix(&paths);
            if !common_prefix.is_empty() {
                let prefix_with_sep = if paths[0].starts_with(&format!("{}::", common_prefix)) {
                    format!("{}::", common_prefix)
                } else {
                    common_prefix.clone()
                };

                let suffixes: Vec<String> = paths
                    .iter()
                    .map(|p| {
                        p.strip_prefix(&prefix_with_sep)
                            .unwrap_or(p.as_str())
                            .to_string()
                    })
                    .collect();

                if prefix_with_sep.ends_with("::") {
                    result.push(format!(
                        "use {}::{}::{{{}}};",
                        root,
                        common_prefix,
                        suffixes.join(", ")
                    ));
                } else {
                    result.push(format!("use {}::{{{}}};", root, suffixes.join(", ")));
                }
            } else {
                result.push(format!("use {}::{{{}}};", root, paths.join(", ")));
            }
        }
    }

    result
}

/// Finds common prefix among paths.
///
/// # Arguments
///
/// * `paths` - List of paths
///
/// # Returns
///
/// Common prefix string
fn find_common_prefix(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }

    let parts: Vec<Vec<&str>> = paths.iter().map(|p| p.split("::").collect()).collect();

    let min_len = parts.iter().map(|p| p.len()).min().unwrap_or(0);

    let mut common = Vec::new();
    for i in 0..min_len {
        let first = parts[0][i];
        if parts.iter().all(|p| p[i] == first) {
            common.push(first);
        } else {
            break;
        }
    }

    if !common.is_empty() {
        common.join("::")
    } else {
        String::new()
    }
}

/// Renders a single file diff block into lines.
///
/// # Arguments
///
/// * `file` - File diff to render
///
/// # Returns
///
/// Rendered file with lines and calculated width
fn render_file_block(file: &FileDiff) -> RenderedFile {
    let mut lines = Vec::new();
    let mut max_width = 0;

    let header = format!("File: {}", file.path);
    max_width = max_width.max(measure_text_width(&header));
    lines.push(header.cyan().bold().to_string());

    let separator = "─".repeat(40);
    max_width = max_width.max(measure_text_width(&separator));
    lines.push(separator.dimmed().to_string());

    let imports: Vec<&str> = file
        .entries
        .iter()
        .filter_map(|e| e.import.as_deref())
        .collect();

    if !imports.is_empty() {
        let import_header = "Imports (file top)";
        max_width = max_width.max(measure_text_width(import_header));
        lines.push(import_header.dimmed().to_string());

        let grouped = group_imports(&imports);
        for import in grouped {
            let import_line = format!("+    {}", import);
            max_width = max_width.max(measure_text_width(&import_line));
            lines.push(import_line.green().to_string());
        }
        lines.push(String::new());
    }

    let mut last_analyzer = "";
    for entry in &file.entries {
        if entry.analyzer != last_analyzer {
            if !last_analyzer.is_empty() {
                lines.push(String::new());
            }
            let analyzer_line = format!(
                "{} ({} issues)",
                entry.analyzer,
                file.entries
                    .iter()
                    .filter(|e| e.analyzer == entry.analyzer)
                    .count()
            );
            max_width = max_width.max(measure_text_width(&analyzer_line));
            lines.push(analyzer_line.green().bold().to_string());
            lines.push(String::new());
            last_analyzer = &entry.analyzer;
        }

        let line_header = format!("Line {}", entry.line);
        max_width = max_width.max(measure_text_width(&line_header));
        lines.push(line_header.cyan().to_string());

        let old_line = format!("-    {}", entry.original);
        max_width = max_width.max(measure_text_width(&old_line));
        lines.push(old_line.red().to_string());

        let new_line = format!("+    {}", entry.modified);
        max_width = max_width.max(measure_text_width(&new_line));
        lines.push(new_line.green().to_string());

        lines.push(String::new());
    }

    let end_separator = "═".repeat(40);
    max_width = max_width.max(measure_text_width(&end_separator));
    lines.push(end_separator.dimmed().to_string());

    RenderedFile {
        lines,
        width: max_width.max(MIN_FILE_WIDTH)
    }
}

/// Calculates how many columns fit in terminal width.
///
/// # Arguments
///
/// * `files` - Rendered files
/// * `term_width` - Terminal width
///
/// # Returns
///
/// Number of columns that fit
fn calculate_columns(files: &[RenderedFile], term_width: usize) -> usize {
    if files.is_empty() {
        return 1;
    }

    let max_file_width = files
        .iter()
        .map(|f| f.width)
        .max()
        .unwrap_or(MIN_FILE_WIDTH);

    for cols in (1..=files.len()).rev() {
        let total_width = cols * max_file_width + (cols - 1) * COLUMN_GAP;
        if total_width <= term_width {
            return cols;
        }
    }

    1
}

/// Pads string to exact width.
///
/// # Arguments
///
/// * `text` - Text to pad (may contain ANSI escape codes)
/// * `width` - Target width
///
/// # Returns
///
/// Padded string
fn pad_to_width(text: &str, width: usize) -> String {
    let current = measure_text_width(text);
    if current >= width {
        return text.to_string();
    }

    let padding = width - current;
    format!("{}{}", text, " ".repeat(padding))
}

/// Renders files in grid layout.
///
/// # Arguments
///
/// * `files` - Rendered files
/// * `columns` - Number of columns
fn render_grid(files: &[RenderedFile], columns: usize) {
    if columns == 1 {
        for file in files {
            for line in &file.lines {
                println!("{}", line);
            }
            println!();
        }
        return;
    }

    let col_width = files
        .iter()
        .map(|f| f.width)
        .max()
        .unwrap_or(MIN_FILE_WIDTH);

    for chunk in files.chunks(columns) {
        let max_lines = chunk.iter().map(|f| f.lines.len()).max().unwrap_or(0);

        for row_idx in 0..max_lines {
            let mut row_output = String::new();

            for (col_idx, file) in chunk.iter().enumerate() {
                let line = file.lines.get(row_idx).map(String::as_str).unwrap_or("");

                let padded = pad_to_width(line, col_width);
                row_output.push_str(&padded);

                if col_idx < chunk.len() - 1 {
                    row_output.push_str(&" ".repeat(COLUMN_GAP));
                }
            }

            println!("{}", row_output);
        }

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

/// Displays full responsive diff output with grid layout.
///
/// Automatically arranges files in columns based on terminal width.
///
/// # Arguments
///
/// * `result` - Diff results to display
pub fn show_full(result: &DiffResult) {
    println!("\n{}\n", "DIFF OUTPUT".bold());

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let rendered: Vec<RenderedFile> = result.files.iter().map(render_file_block).collect();

    let columns = calculate_columns(&rendered, term_width);

    if columns > 1 {
        println!(
            "{}\n",
            format!(
                "Layout: {} columns (terminal width: {})",
                columns, term_width
            )
            .dimmed()
        );
    }

    render_grid(&rendered, columns);

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
    fn test_render_file_block() {
        let mut file_diff = FileDiff::new("test.rs".to_string());
        file_diff.add_entry(DiffEntry {
            line:        10,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        });

        let rendered = render_file_block(&file_diff);
        assert!(!rendered.lines.is_empty());
        assert!(rendered.width >= MIN_FILE_WIDTH);
    }

    #[test]
    fn test_calculate_columns() {
        let file1 = RenderedFile {
            lines: vec![String::from("test")],
            width: 50
        };
        let file2 = RenderedFile {
            lines: vec![String::from("test")],
            width: 50
        };

        let files = vec![file1, file2];
        let cols = calculate_columns(&files, 200);
        assert!(cols >= 1);
    }

    #[test]
    fn test_calculate_columns_narrow() {
        let file = RenderedFile {
            lines: vec![String::from("test")],
            width: 100
        };

        let files = vec![file];
        let cols = calculate_columns(&files, 80);
        assert_eq!(cols, 1);
    }

    #[test]
    fn test_pad_to_width() {
        let result = pad_to_width("hello", 10);
        assert_eq!(result.len(), 10);
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

        file_diff.add_entry(DiffEntry {
            line:        1,
            analyzer:    "test_analyzer".to_string(),
            original:    "old line".to_string(),
            modified:    "new line".to_string(),
            description: "test change".to_string(),
            import:      None
        });

        result.add_file(file_diff);
        show_summary(&result);
    }

    #[test]
    fn test_show_full_with_data() {
        let mut result = DiffResult::new();
        let mut file_diff = FileDiff::new("test.rs".to_string());

        file_diff.add_entry(DiffEntry {
            line:        10,
            analyzer:    "format_args".to_string(),
            original:    "println!(\"Hello {}\", name)".to_string(),
            modified:    "println!(\"Hello {name}\")".to_string(),
            description: "Use named arguments".to_string(),
            import:      None
        });

        result.add_file(file_diff);
        show_full(&result);
    }

    #[test]
    fn test_render_grid_single_column() {
        let file = RenderedFile {
            lines: vec![String::from("line1"), String::from("line2")],
            width: 50
        };

        render_grid(&[file], 1);
    }

    #[test]
    fn test_render_grid_multiple_columns() {
        let file1 = RenderedFile {
            lines: vec![String::from("file1 line1")],
            width: 50
        };
        let file2 = RenderedFile {
            lines: vec![String::from("file2 line1")],
            width: 50
        };

        render_grid(&[file1, file2], 2);
    }

    #[test]
    fn test_render_file_with_import() {
        let mut file_diff = FileDiff::new("test.rs".to_string());
        file_diff.add_entry(DiffEntry {
            line:        10,
            analyzer:    "path_import".to_string(),
            original:    "std::fs::read(...)".to_string(),
            modified:    "read(...)".to_string(),
            description: "Use import".to_string(),
            import:      Some("use std::fs::read;".to_string())
        });

        let rendered = render_file_block(&file_diff);
        assert!(rendered.lines.iter().any(|l| l.contains("Imports")));
    }
}
