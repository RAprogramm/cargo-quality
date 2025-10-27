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
/// let rendered = render_file_block(&file_diff);
///
/// assert!(!rendered.lines.is_empty());
/// assert!(rendered.width >= 40);
/// ```
pub fn render_file_block(file: &FileDiff) -> RenderedFile {
    let estimated_capacity = ESTIMATED_LINES_PER_FILE + file.entries.len() * 5;

    let mut lines = Vec::with_capacity(estimated_capacity);
    let mut max_width = 0;

    render_header(&mut lines, &mut max_width, &file.path);

    render_imports(&mut lines, &mut max_width, file);

    render_issues(&mut lines, &mut max_width, file);

    render_footer(&mut lines, &mut max_width);

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
fn render_header(lines: &mut Vec<String>, max_width: &mut usize, path: &str) {
    let header = format!("File: {}", path);
    *max_width = (*max_width).max(measure_text_width(&header));
    lines.push(header.cyan().bold().to_string());

    let separator = "─".repeat(40);
    *max_width = (*max_width).max(measure_text_width(&separator));
    lines.push(separator.dimmed().to_string());
}

/// Renders grouped import section if present.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
/// * `file` - File diff data
#[inline]
fn render_imports(lines: &mut Vec<String>, max_width: &mut usize, file: &FileDiff) {
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
    lines.push(import_header.dimmed().to_string());

    let grouped = group_imports(&imports);
    for import in grouped {
        let import_line = format!("+    {}", import);
        *max_width = (*max_width).max(measure_text_width(&import_line));
        lines.push(import_line.green().to_string());
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
fn render_issues(lines: &mut Vec<String>, max_width: &mut usize, file: &FileDiff) {
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

            *max_width = (*max_width).max(measure_text_width(&analyzer_line));
            lines.push(analyzer_line.green().bold().to_string());
            lines.push(String::new());

            last_analyzer = &entry.analyzer;
        }

        render_issue_entry(lines, max_width, entry);
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
    entry: &crate::differ::types::DiffEntry
) {
    let line_header = format!("Line {}", entry.line);
    *max_width = (*max_width).max(measure_text_width(&line_header));
    lines.push(line_header.cyan().to_string());

    let old_line = format!("-    {}", entry.original);
    *max_width = (*max_width).max(measure_text_width(&old_line));
    lines.push(old_line.red().to_string());

    let new_line = format!("+    {}", entry.modified);
    *max_width = (*max_width).max(measure_text_width(&new_line));
    lines.push(new_line.green().to_string());

    lines.push(String::new());
}

/// Renders footer separator.
///
/// # Arguments
///
/// * `lines` - Output buffer
/// * `max_width` - Running maximum width tracker
#[inline]
fn render_footer(lines: &mut Vec<String>, max_width: &mut usize) {
    let end_separator = "═".repeat(40);
    *max_width = (*max_width).max(measure_text_width(&end_separator));
    lines.push(end_separator.dimmed().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::differ::types::{DiffEntry, FileDiff};

    #[test]
    fn test_render_file_block_empty() {
        let file = FileDiff::new("test.rs".to_string());
        let rendered = render_file_block(&file);

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

        let rendered = render_file_block(&file);
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

        let rendered = render_file_block(&file);
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

        let rendered = render_file_block(&file);
        assert!(rendered.lines.iter().any(|l| l.contains("analyzer1")));
        assert!(rendered.lines.iter().any(|l| l.contains("analyzer2")));
    }

    #[test]
    fn test_render_respects_capacity() {
        let file = FileDiff::new("test.rs".to_string());
        let rendered = render_file_block(&file);

        assert!(rendered.lines.capacity() >= ESTIMATED_LINES_PER_FILE);
    }
}
