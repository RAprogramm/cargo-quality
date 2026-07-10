// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Application of selected diff entries to files on disk.
//!
//! Used by interactive mode to write only the changes the user accepted.
//! Line replacements are keyed by line number and guarded by an equality check
//! against the recorded original, so a change is skipped if the file no longer
//! matches what the diff was generated from.

use std::fs;

use masterror::AppResult;

use super::types::{DiffResult, FileDiff};
use crate::error::IoError;

/// Applies selected diff entries to their files.
///
/// For each file, replaces recorded lines with their modified form and inserts
/// deduplicated `use` imports after leading module docs and inner attributes.
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
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let mut imports: Vec<String> = Vec::new();
    let mut applied = 0;

    for entry in &file.entries {
        let idx = entry.line.saturating_sub(1);

        let Some(line) = lines.get_mut(idx) else {
            continue;
        };

        if *line != entry.original {
            continue;
        }

        *line = entry.modified.clone();

        if let Some(import) = &entry.import
            && !imports.contains(import)
        {
            imports.push(import.clone());
        }

        applied += 1;
    }

    if applied == 0 {
        return Ok(0);
    }

    insert_imports(&mut lines, imports);

    let mut text = lines.join("\n");
    if content.ends_with('\n') {
        text.push('\n');
    }

    fs::write(&file.path, text).map_err(IoError::from)?;

    Ok(applied)
}

/// Inserts import lines after leading module docs and inner attributes.
///
/// Skips a leading run of blank lines, non-doc `//` comments, module docs
/// (`//!`), and inner attributes (`#!`) so the imports remain valid Rust.
///
/// # Arguments
///
/// * `lines` - File lines to modify in place
/// * `imports` - Deduplicated import statements to insert
fn insert_imports(lines: &mut Vec<String>, imports: Vec<String>) {
    if imports.is_empty() {
        return;
    }

    let mut insert_at = 0;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty()
            || trimmed.starts_with("//!")
            || trimmed.starts_with("#!")
            || (trimmed.starts_with("//") && !trimmed.starts_with("///"))
        {
            insert_at = index + 1;
        } else {
            break;
        }
    }

    for (offset, import) in imports.into_iter().enumerate() {
        lines.insert(insert_at + offset, import);
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::differ::types::DiffEntry;

    fn entry(line: usize, original: &str, modified: &str, import: Option<&str>) -> DiffEntry {
        DiffEntry {
            line,
            analyzer: "path_import".to_string(),
            original: original.to_string(),
            modified: modified.to_string(),
            description: "desc".to_string(),
            import: import.map(str::to_string)
        }
    }

    #[test]
    fn test_apply_rewrites_line_and_inserts_import() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("a.rs");
        fs::write(
            &path,
            "//! Module doc\n\nfn main() {\n    let x = std::fs::read_to_string(\"f\");\n}\n"
        )
        .unwrap();

        let mut result = DiffResult::new();
        let mut file = FileDiff::new(path.to_str().unwrap().to_string());
        file.add_entry(entry(
            4,
            "    let x = std::fs::read_to_string(\"f\");",
            "    let x = read_to_string(\"f\");",
            Some("use std::fs::read_to_string;")
        ));
        result.add_file(file);

        let applied = apply_diff(&result).unwrap();
        assert_eq!(applied, 1);

        let output = fs::read_to_string(&path).unwrap();
        assert!(output.contains("use std::fs::read_to_string;"));
        assert!(output.contains("let x = read_to_string(\"f\");"));
        assert!(!output.contains("std::fs::read_to_string("));
        assert!(output.starts_with("//! Module doc"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_apply_skips_mismatched_original() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("b.rs");
        fs::write(&path, "fn main() {\n    let x = 1;\n}\n").unwrap();

        let mut result = DiffResult::new();
        let mut file = FileDiff::new(path.to_str().unwrap().to_string());
        file.add_entry(entry(2, "    let x = 2;", "    let x = changed;", None));
        result.add_file(file);

        let applied = apply_diff(&result).unwrap();
        assert_eq!(applied, 0);

        let output = fs::read_to_string(&path).unwrap();
        assert!(output.contains("let x = 1;"));
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

        let mut result = DiffResult::new();
        let mut file = FileDiff::new(path.to_str().unwrap().to_string());
        file.add_entry(entry(
            2,
            "    let a = std::fs::read_to_string(\"a\");",
            "    let a = read_to_string(\"a\");",
            Some("use std::fs::read_to_string;")
        ));
        file.add_entry(entry(
            3,
            "    let b = std::fs::read_to_string(\"b\");",
            "    let b = read_to_string(\"b\");",
            Some("use std::fs::read_to_string;")
        ));
        result.add_file(file);

        let applied = apply_diff(&result).unwrap();
        assert_eq!(applied, 2);

        let output = fs::read_to_string(&path).unwrap();
        assert_eq!(output.matches("use std::fs::read_to_string;").count(), 1);
    }

    #[test]
    fn test_apply_empty_result() {
        let result = DiffResult::new();
        let applied = apply_diff(&result).unwrap();
        assert_eq!(applied, 0);
    }
}
