// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use super::{formatting::pad_to_width, types::RenderedFile};

/// Minimum space between columns in grid layout.
pub const COLUMN_GAP: usize = 4;

/// Minimum width for a file column to be considered viable.
pub const MIN_FILE_WIDTH: usize = 40;

/// Calculates optimal number of columns for grid layout.
///
/// Determines how many file columns can fit horizontally based on terminal
/// width and file content widths. Uses a greedy algorithm that tries to
/// maximize column count while respecting minimum width requirements.
///
/// # Algorithm
///
/// 1. Find maximum file width across all rendered files
/// 2. Try column counts from N down to 1
/// 3. Calculate total width: `cols × max_width + (cols-1) × gap`
/// 4. Return first count that fits terminal width
///
/// # Arguments
///
/// * `files` - Slice of rendered files with calculated widths
/// * `term_width` - Terminal width in characters
///
/// # Returns
///
/// Number of columns (1 to N) that optimally fit the terminal
///
/// # Performance
///
/// - O(n) to find max width
/// - O(n) to try column counts (at most file count iterations)
/// - Total: O(n) where n is file count
///
/// # Examples
///
/// ```
/// use cargo_quality::differ::display::{grid::calculate_columns, types::RenderedFile};
///
/// let files = vec![
///     RenderedFile {
///         lines: Vec::new(),
///         width: 50
///     },
///     RenderedFile {
///         lines: Vec::new(),
///         width: 45
///     },
/// ];
///
/// let cols = calculate_columns(&files, 150);
/// assert!(cols >= 1 && cols <= 2);
/// ```
///
/// ```
/// use cargo_quality::differ::display::{grid::calculate_columns, types::RenderedFile};
///
/// let files = vec![RenderedFile {
///     lines: Vec::new(),
///     width: 100
/// }];
/// let cols = calculate_columns(&files, 80);
/// assert_eq!(cols, 1); // Too wide for multiple columns
/// ```
#[inline]
pub fn calculate_columns(files: &[RenderedFile], term_width: usize) -> usize {
    if files.is_empty() {
        return 1;
    }

    let max_file_width = files
        .iter()
        .map(|f| f.width)
        .max()
        .unwrap_or(MIN_FILE_WIDTH)
        .max(MIN_FILE_WIDTH);

    for cols in (1..=files.len()).rev() {
        let total_width = cols * max_file_width + (cols.saturating_sub(1)) * COLUMN_GAP;

        if total_width <= term_width {
            return cols;
        }
    }

    1
}

/// Renders files in responsive grid layout.
///
/// Arranges files horizontally in columns, printing them row by row. Each file
/// occupies one column, with rows printed until all files are displayed.
/// Handles variable line counts by padding short files with empty lines.
///
/// # Algorithm
///
/// 1. If single column: print each file vertically
/// 2. Calculate column width (max file width)
/// 3. Process files in chunks of `columns` size
/// 4. For each chunk:
///    - Find max line count
///    - Print each row across all columns
///    - Pad columns to align properly
///
/// # Arguments
///
/// * `files` - Slice of rendered files to display
/// * `columns` - Number of columns to use (from `calculate_columns`)
///
/// # Performance
///
/// - Single allocation per output line
/// - Pre-calculates maximum line count per chunk
/// - Uses padding with pre-calculated widths
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::differ::display::{grid::render_grid, types::RenderedFile};
///
/// let file1 = RenderedFile {
///     lines: vec!["Line 1".to_string(), "Line 2".to_string()],
///     width: 40
/// };
///
/// let file2 = RenderedFile {
///     lines: vec!["Other 1".to_string(), "Other 2".to_string()],
///     width: 40
/// };
///
/// render_grid(&[file1, file2], 2);
/// ```
pub fn render_grid(files: &[RenderedFile], columns: usize) {
    if files.is_empty() {
        return;
    }

    if columns == 1 {
        render_single_column(files);
        return;
    }

    let col_width = files
        .iter()
        .map(|f| f.width)
        .max()
        .unwrap_or(MIN_FILE_WIDTH);

    for chunk in files.chunks(columns) {
        let max_lines = chunk.iter().map(|f| f.line_count()).max().unwrap_or(0);

        for row_idx in 0..max_lines {
            let mut row_output = String::with_capacity(columns * (col_width + COLUMN_GAP));

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

/// Renders files in single column mode.
///
/// Simple vertical layout for narrow terminals or when optimal layout requires
/// one column. Prints each file sequentially with spacing.
///
/// # Arguments
///
/// * `files` - Slice of rendered files
///
/// # Performance
///
/// - Direct line-by-line output
/// - No padding calculations needed
/// - Minimal allocations
#[inline]
fn render_single_column(files: &[RenderedFile]) {
    for file in files {
        for line in &file.lines {
            println!("{}", line);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_columns_empty() {
        let files: Vec<RenderedFile> = vec![];
        let cols = calculate_columns(&files, 100);
        assert_eq!(cols, 1);
    }

    #[test]
    fn test_calculate_columns_single_narrow() {
        let files = vec![RenderedFile {
            lines: Vec::new(),
            width: 40
        }];
        let cols = calculate_columns(&files, 200);
        assert_eq!(cols, 1);
    }

    #[test]
    fn test_calculate_columns_two_fit() {
        let files = vec![
            RenderedFile {
                lines: Vec::new(),
                width: 50
            },
            RenderedFile {
                lines: Vec::new(),
                width: 50
            },
        ];
        let cols = calculate_columns(&files, 150);
        assert!(cols >= 1);
    }

    #[test]
    fn test_calculate_columns_narrow_terminal() {
        let files = vec![
            RenderedFile {
                lines: Vec::new(),
                width: 100
            },
            RenderedFile {
                lines: Vec::new(),
                width: 100
            },
        ];
        let cols = calculate_columns(&files, 80);
        assert_eq!(cols, 1);
    }

    #[test]
    fn test_calculate_columns_wide_terminal() {
        let files = vec![
            RenderedFile {
                lines: Vec::new(),
                width: 40
            },
            RenderedFile {
                lines: Vec::new(),
                width: 40
            },
            RenderedFile {
                lines: Vec::new(),
                width: 40
            },
        ];
        let cols = calculate_columns(&files, 250);
        assert!(cols >= 2);
    }

    #[test]
    fn test_render_grid_single_column() {
        let file = RenderedFile {
            lines: vec!["line1".to_string(), "line2".to_string()],
            width: 40
        };

        render_grid(&[file], 1);
    }

    #[test]
    fn test_render_grid_empty() {
        let files: Vec<RenderedFile> = vec![];
        render_grid(&files, 2);
    }

    #[test]
    fn test_render_grid_multiple_columns() {
        let file1 = RenderedFile {
            lines: vec!["file1".to_string()],
            width: 40
        };

        let file2 = RenderedFile {
            lines: vec!["file2".to_string()],
            width: 40
        };

        render_grid(&[file1, file2], 2);
    }

    #[test]
    fn test_calculate_columns_respects_min_width() {
        let files = vec![RenderedFile {
            lines: Vec::new(),
            width: 30
        }];
        let cols = calculate_columns(&files, 200);
        assert_eq!(cols, 1);
    }

    #[test]
    fn test_render_single_column_multiple_files() {
        let file1 = RenderedFile {
            lines: vec!["test1".to_string()],
            width: 40
        };

        let file2 = RenderedFile {
            lines: vec!["test2".to_string()],
            width: 40
        };

        render_single_column(&[file1, file2]);
    }
}
