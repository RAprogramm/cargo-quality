// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application of byte-range text edits to source code.
//!
//! Fixes are expressed as [`crate::analyzer::TextEdit`]s against the original
//! source and applied here. Because only the edited byte ranges change,
//! comments, blank lines, and the author's formatting are preserved — unlike
//! reprinting the AST, which drops comments and reformats the whole file.

use std::collections::HashSet;

use crate::analyzer::{Suggestion, TextEdit};

/// Applies fix suggestions to the source, deduplicating their imports.
///
/// Collects each suggestion's rewrite edit, inserts every distinct required
/// import once at the top of the file, and applies them via [`apply_edits`].
/// Comments, blank lines, and formatting outside the edits are preserved.
///
/// # Arguments
///
/// * `source` - Original source code
/// * `suggestions` - Suggestions to apply
///
/// # Returns
///
/// The edited source
pub fn apply_suggestions(source: &str, suggestions: &[Suggestion]) -> String {
    let mut edits: Vec<TextEdit> = suggestions.iter().map(|s| s.edit.clone()).collect();

    let mut seen = HashSet::new();
    let mut imports = Vec::new();
    for suggestion in suggestions {
        if let Some(import) = &suggestion.import
            && seen.insert(import.clone())
        {
            imports.push(import.clone());
        }
    }

    if !imports.is_empty() {
        let offset = import_insertion_offset(source);
        let mut block = imports.join("\n");
        block.push('\n');
        edits.push(TextEdit {
            range:       offset..offset,
            replacement: block
        });
    }

    apply_edits(source, edits)
}

/// Applies non-overlapping text edits to the source.
///
/// Edits are applied from the highest start offset to the lowest so that
/// earlier byte offsets stay valid while later ones are rewritten.
///
/// # Arguments
///
/// * `source` - Original source code
/// * `edits` - Non-overlapping byte-range edits
///
/// # Returns
///
/// The edited source
///
/// # Examples
///
/// ```
/// use cargo_quality::{analyzer::TextEdit, fixer::apply_edits};
///
/// let src = "let x = std::fs::read(\"f\");";
/// let edits = vec![TextEdit {
///     range:       8..17,
///     replacement: String::new()
/// }];
/// assert_eq!(apply_edits(src, edits), "let x = read(\"f\");");
/// ```
pub fn apply_edits(source: &str, mut edits: Vec<TextEdit>) -> String {
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));

    let mut output = source.to_string();
    for edit in edits {
        output.replace_range(edit.range, &edit.replacement);
    }

    output
}

/// Computes the byte offset at which to insert `use` statements.
///
/// Skips the leading run of blank lines, non-doc `//` comments, module docs
/// (`//!`), and inner attributes (`#!`), so inserted imports stay valid Rust
/// and land above the first item.
///
/// # Arguments
///
/// * `source` - Original source code
///
/// # Returns
///
/// Byte offset for import insertion
pub fn import_insertion_offset(source: &str) -> usize {
    let mut offset = 0;

    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.is_empty()
            || trimmed.starts_with("//!")
            || trimmed.starts_with("#!")
            || (trimmed.starts_with("//") && !trimmed.starts_with("///"))
        {
            offset += line.len();
        } else {
            break;
        }
    }

    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_single_edit() {
        let src = "abcdef";
        let edits = vec![TextEdit {
            range:       2..4,
            replacement: "XY".to_string()
        }];
        assert_eq!(apply_edits(src, edits), "abXYef");
    }

    #[test]
    fn test_apply_multiple_non_overlapping_edits() {
        let src = "one two three";
        let edits = vec![
            TextEdit {
                range:       0..3,
                replacement: "1".to_string()
            },
            TextEdit {
                range:       8..13,
                replacement: "3".to_string()
            },
        ];
        assert_eq!(apply_edits(src, edits), "1 two 3");
    }

    #[test]
    fn test_apply_deletion() {
        let src = "let x = std::fs::read(\"f\");";
        let edits = vec![TextEdit {
            range:       8..17,
            replacement: String::new()
        }];
        assert_eq!(apply_edits(src, edits), "let x = read(\"f\");");
    }

    #[test]
    fn test_apply_no_edits_is_identity() {
        let src = "unchanged";
        assert_eq!(apply_edits(src, Vec::new()), "unchanged");
    }

    #[test]
    fn test_insertion_offset_skips_module_docs() {
        let src = "// SPDX header\n//! module doc\n\nuse std::fmt;\nfn main() {}\n";
        let offset = import_insertion_offset(src);
        assert_eq!(&src[offset..offset + 3], "use");
    }

    #[test]
    fn test_insertion_offset_stops_at_outer_doc() {
        let src = "/// item doc\nfn main() {}\n";
        assert_eq!(import_insertion_offset(src), 0);
    }

    #[test]
    fn test_insertion_offset_empty_source() {
        assert_eq!(import_insertion_offset(""), 0);
    }
}
