// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use console::measure_text_width;
use owo_colors::OwoColorize;

use super::{grid::MIN_FILE_WIDTH, grouping::group_imports, types::RenderedFile};
use crate::differ::types::FileDiff;

/// Estimated lines per file diff for pre-allocation.
///
/// Based on typical diff structure:
/// - 2 lines for header
/// - 1-5 lines for imports
/// - 3-5 lines per issue (analyzer header + line + old + new + blank)
const ESTIMATED_LINES_PER_FILE: usize = 20;

/// Renders a single file diff into formatted output lines.
///
/// Transforms file diff data into visual representation with colors,
/// separators, and grouped imports. Pre-calculates visual width for grid layout
/// optimization.
///
/// # Structure
///
/// ```text
/// File: path/to/file.rs           <- Header (cyan + bold)
/// ────────────────────────────    <- Separator
/// Imports (file top)              <- Import section header
/// +    use std::fs::write;        <- Grouped imports (green)
///
/// analyzer_name (N issues)        <- Analyzer section (green + bold)
///
/// Line 42                         <- Line number (cyan)
/// -    old code                   <- Removal (red)
/// +    new code                   <- Addition (green)
///
/// ════════════════════════════    <- End separator
/// ```
///
/// # Arguments
///
/// * `file` - File diff containing all changes
///
/// # Returns
///
/// `RenderedFile` with formatted lines and calculated width
///
/// # Performance
///
/// - Pre-allocates Vec with estimated capacity
/// - Groups and deduplicates imports once
/// - Calculates width incrementally (single pass)
/// - Minimizes string allocations
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::differ::{display::render::render_file_block, types::FileDiff};
///
/// let file_diff = FileDiff::new("test.rs".to_string());
/// let rendered = render_file_block(&file_diff, false);
///
/// assert!(!rendered.lines.is_empty());
/// assert!(rendered.width >= 40);
/// ```
pub fn render_file_block(file: &FileDiff, color: bool) -> RenderedFile {
    let estimated_capacity = ESTIMATED_LINES_PER_FILE + file.entries.len() * 5;

    let mut lines = Vec::with_capacity(estimated_capacity);
    let mut max_width = 0;

    render_header(&mut lines, &mut max_width, &file.path, color);

    render_imports(&mut lines, &mut max_width, file, color);

    render_issues(&mut lines, &mut max_width, file, color);

    render_empty_lines_note(&mut lines, max_width, file, color);

    render_footer(&mut lines, &mut max_width, color);

    RenderedFile {
        lines,
        width: max_width.max(MIN_FILE_WIDTH)
    }
}

/// Renders file header with path.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
/// * `path` - File path string
#[inline]
fn render_header(lines: &mut Vec<String>, max_width: &mut usize, path: &str, color: bool) {
    let header = format!("File: {}", path);
    *max_width = (*max_width).max(measure_text_width(&header));

    if color {
        lines.push(header.cyan().bold().to_string());
    } else {
        lines.push(header);
    }

    let separator = "─".repeat(40);
    *max_width = (*max_width).max(measure_text_width(&separator));

    if color {
        lines.push(separator.dimmed().to_string());
    } else {
        lines.push(separator);
    }
}

/// Renders grouped import section if present.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
/// * `file` - File diff data
#[inline]
fn render_imports(lines: &mut Vec<String>, max_width: &mut usize, file: &FileDiff, color: bool) {
    let imports: Vec<&str> = file
        .entries
        .iter()
        .filter_map(|e| e.import.as_deref())
        .collect();

    if imports.is_empty() {
        return;
    }

    let import_header = "Imports (file top)";
    *max_width = (*max_width).max(measure_text_width(import_header));

    if color {
        lines.push(import_header.dimmed().to_string());
    } else {
        lines.push(import_header.to_string());
    }

    let grouped = group_imports(&imports);
    for import in grouped {
        let import_line = format!("+    {}", import);
        *max_width = (*max_width).max(measure_text_width(&import_line));

        if color {
            lines.push(import_line.green().to_string());
        } else {
            lines.push(import_line);
        }
    }

    lines.push(String::new());
}

/// Renders all issues grouped by analyzer.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
/// * `file` - File diff data
#[inline]
fn render_issues(lines: &mut Vec<String>, max_width: &mut usize, file: &FileDiff, color: bool) {
    let mut last_analyzer = "";

    for entry in &file.entries {
        if entry.analyzer == "empty_lines" {
            continue;
        }

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

            *max_width = (*max_width).max(measure_text_width(&analyzer_line));

            if color {
                lines.push(analyzer_line.green().bold().to_string());
            } else {
                lines.push(analyzer_line);
            }

            lines.push(String::new());

            last_analyzer = &entry.analyzer;
        }

        render_issue_entry(lines, max_width, entry, color);
    }
}

/// Renders single issue entry with line changes.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
/// * `entry` - Diff entry data
#[inline]
fn render_issue_entry(
    lines: &mut Vec<String>,
    max_width: &mut usize,
    entry: &crate::differ::types::DiffEntry,
    color: bool
) {
    let line_header = format!("Line {}", entry.line);
    *max_width = (*max_width).max(measure_text_width(&line_header));

    if color {
        lines.push(line_header.cyan().to_string());
    } else {
        lines.push(line_header);
    }

    let old_line = format!("-    {}", entry.original);
    *max_width = (*max_width).max(measure_text_width(&old_line));

    if color {
        lines.push(old_line.red().to_string());
    } else {
        lines.push(old_line);
    }

    let new_line = format!("+    {}", entry.modified);
    *max_width = (*max_width).max(measure_text_width(&new_line));

    if color {
        lines.push(new_line.green().to_string());
    } else {
        lines.push(new_line);
    }

    lines.push(String::new());
}

/// Renders empty lines removal note if present.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Maximum width from other content (not updated)
/// * `file` - File diff data
#[inline]
fn render_empty_lines_note(
    lines: &mut Vec<String>,
    max_width: usize,
    file: &FileDiff,
    color: bool
) {
    let empty_entries: Vec<_> = file
        .entries
        .iter()
        .filter(|e| e.analyzer == "empty_lines")
        .collect();

    if empty_entries.is_empty() {
        return;
    }

    let line_numbers: Vec<String> = empty_entries.iter().map(|e| e.line.to_string()).collect();

    let prefix = format!(
        "Note: {} empty {} will be removed from lines: ",
        empty_entries.len(),
        if empty_entries.len() == 1 {
            "line"
        } else {
            "lines"
        }
    );

    let mut current_line = prefix.clone();

    for (i, num) in line_numbers.iter().enumerate() {
        let separator = if i == 0 { "" } else { ", " };
        let addition = format!("{}{}", separator, num);

        if current_line.len() + addition.len() > max_width && i > 0 {
            if color {
                lines.push(current_line.dimmed().italic().to_string());
            } else {
                lines.push(current_line);
            }
            current_line = format!(" {}", num);
        } else {
            current_line.push_str(&addition);
        }
    }

    if !current_line.is_empty() {
        if color {
            lines.push(current_line.dimmed().italic().to_string());
        } else {
            lines.push(current_line);
        }
    }

    lines.push(String::new());
}

/// Renders footer separator.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
#[inline]
fn render_footer(lines: &mut Vec<String>, max_width: &mut usize, color: bool) {
    let end_separator = "═".repeat(40);
    *max_width = (*max_width).max(measure_text_width(&end_separator));

    if color {
        lines.push(end_separator.dimmed().to_string());
    } else {
        lines.push(end_separator);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::differ::types::{DiffEntry, FileDiff};

    #[test]
    fn test_render_file_block_empty() {
        let file = FileDiff::new("test.rs".to_string());
        let rendered = render_file_block(&file, false);

        assert!(!rendered.lines.is_empty());
        assert!(rendered.width >= MIN_FILE_WIDTH);
    }

    #[test]
    fn test_render_file_block_with_entry() {
        let mut file = FileDiff::new("test.rs".to_string());
        file.add_entry(DiffEntry {
            line:        10,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        });

        let rendered = render_file_block(&file, false);
        assert!(rendered.line_count() > 5);
    }

    #[test]
    fn test_render_file_block_with_import() {
        let mut file = FileDiff::new("test.rs".to_string());
        file.add_entry(DiffEntry {
            line:        10,
            analyzer:    "path_import".to_string(),
            original:    "std::fs::read()".to_string(),
            modified:    "read()".to_string(),
            description: "Use import".to_string(),
            import:      Some("use std::fs::read;".to_string())
        });

        let rendered = render_file_block(&file, false);
        assert!(rendered.lines.iter().any(|l| l.contains("Imports")));
    }

    #[test]
    fn test_render_file_block_multiple_analyzers() {
        let mut file = FileDiff::new("test.rs".to_string());

        file.add_entry(DiffEntry {
            line:        10,
            analyzer:    "analyzer1".to_string(),
            original:    "old1".to_string(),
            modified:    "new1".to_string(),
            description: "desc1".to_string(),
            import:      None
        });

        file.add_entry(DiffEntry {
            line:        20,
            analyzer:    "analyzer2".to_string(),
            original:    "old2".to_string(),
            modified:    "new2".to_string(),
            description: "desc2".to_string(),
            import:      None
        });

        let rendered = render_file_block(&file, false);
        assert!(rendered.lines.iter().any(|l| l.contains("analyzer1")));
        assert!(rendered.lines.iter().any(|l| l.contains("analyzer2")));
    }

    #[test]
    fn test_render_respects_capacity() {
        let file = FileDiff::new("test.rs".to_string());
        let rendered = render_file_block(&file, false);

        assert!(rendered.lines.capacity() >= ESTIMATED_LINES_PER_FILE);
    }
}
