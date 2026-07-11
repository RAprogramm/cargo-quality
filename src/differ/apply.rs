// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application of selected diff entries to files on disk.
//!
//! Used by interactive mode to write only the changes the user accepted. Each
//! entry carries the underlying [`crate::analyzer::TextEdit`], so changes are
//! applied through the same [`crate::fixer::apply_suggestions`] engine as the
//! `fix` command — collision-safe and comment-preserving. An entry is skipped
//! if the file no longer matches the line the diff was generated from.

use std::fs;

use masterror::AppResult;

use super::types::{DiffResult, FileDiff};
use crate::{analyzer::Suggestion, error::IoError, fixer::apply_suggestions};

/// Applies selected diff entries to their files.
///
/// Each file's accepted entries are turned back into suggestions and applied
/// through [`apply_suggestions`], so the result is collision-safe and preserves
/// comments and formatting — identical to the `fix` command.
///
/// # Arguments
///
/// * `result` - Selected diff entries grouped by file
///
/// # Returns
///
/// `AppResult<usize>` - Number of line changes written across all files
///
/// # Errors
///
/// Returns an error if reading or writing a file fails.
pub fn apply_diff(result: &DiffResult) -> AppResult<usize> {
    let mut applied = 0;

    for file in &result.files {
        applied += apply_file(file)?;
    }

    Ok(applied)
}

/// Applies the entries of a single file diff.
///
/// # Arguments
///
/// * `file` - File diff with the entries to apply
///
/// # Returns
///
/// `AppResult<usize>` - Number of line changes written for this file
fn apply_file(file: &FileDiff) -> AppResult<usize> {
    if file.entries.is_empty() {
        return Ok(0);
    }

    let content = fs::read_to_string(&file.path).map_err(IoError::from)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut suggestions = Vec::new();
    for entry in &file.entries {
        let idx = entry.line.saturating_sub(1);
        if lines.get(idx).is_some_and(|line| *line == entry.original) {
            suggestions.push(Suggestion {
                edit:   entry.edit.clone(),
                import: entry.import.clone()
            });
        }
    }

    if suggestions.is_empty() {
        return Ok(0);
    }

    let updated = apply_suggestions(&content, &suggestions);
    fs::write(&file.path, updated).map_err(IoError::from)?;

    Ok(suggestions.len())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;
    use crate::analyzers::get_analyzers;

    fn diff_for(path: &Path) -> DiffResult {
        let file = super::super::generate_diff(path.to_str().unwrap(), &get_analyzers()).unwrap();
        let mut result = DiffResult::new();
        result.add_file(file);
        result
    }

    #[test]
    fn test_apply_rewrites_and_preserves_comments() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("a.rs");
        fs::write(
            &path,
            "//! Module doc\n\nfn main() {\n    // note\n    let x = std::fs::read_to_string(\"f\");\n}\n"
        )
        .unwrap();

        let applied = apply_diff(&diff_for(&path)).unwrap();
        assert_eq!(applied, 1);

        let output = fs::read_to_string(&path).unwrap();
        assert!(output.contains("use std::fs::read_to_string;"));
        assert!(output.contains("let x = read_to_string(\"f\");"));
        assert!(!output.contains("std::fs::read_to_string("));
        assert!(output.contains("// note"), "comment preserved");
        assert!(output.starts_with("//! Module doc"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_apply_dedups_imports() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("c.rs");
        fs::write(
            &path,
            "fn main() {\n    let a = std::fs::read_to_string(\"a\");\n    let b = std::fs::read_to_string(\"b\");\n}\n"
        )
        .unwrap();

        let applied = apply_diff(&diff_for(&path)).unwrap();
        assert_eq!(applied, 2);

        let output = fs::read_to_string(&path).unwrap();
        assert_eq!(output.matches("use std::fs::read_to_string;").count(), 1);
    }

    #[test]
    fn test_apply_is_collision_safe() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("d.rs");
        fs::write(
            &path,
            "fn main() {\n    let a = std::fs::read(\"x\");\n    let b = other::helpers::read(\"y\");\n}\n"
        )
        .unwrap();

        let result = diff_for(&path);
        assert_eq!(
            result.total_changes(),
            0,
            "colliding reads produce no changes"
        );

        let applied = apply_diff(&result).unwrap();
        assert_eq!(applied, 0);
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("std::fs::read(\"x\")")
        );
    }

    #[test]
    fn test_apply_skips_when_file_changed() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("e.rs");
        fs::write(
            &path,
            "fn main() {\n    let x = std::fs::read_to_string(\"f\");\n}\n"
        )
        .unwrap();

        let result = diff_for(&path);
        fs::write(&path, "fn main() {\n    let y = 1;\n}\n").unwrap();

        let applied = apply_diff(&result).unwrap();
        assert_eq!(applied, 0, "stale entry is skipped");
        assert!(fs::read_to_string(&path).unwrap().contains("let y = 1;"));
    }

    #[test]
    fn test_apply_empty_result() {
        let applied = apply_diff(&DiffResult::new()).unwrap();
        assert_eq!(applied, 0);
    }
}
