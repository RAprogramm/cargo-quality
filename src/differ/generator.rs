// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fs;

use masterror::AppResult;

use super::types::{DiffEntry, FileDiff};
use crate::{
    analyzer::{Analyzer, Suggestion},
    error::{IoError, ParseError}
};

/// Generates diff showing proposed changes.
///
/// Analyzes files and compares current state with proposed fixes.
///
/// # Arguments
///
/// * `file_path` - Path to analyze
/// * `analyzers` - List of analyzers to apply
///
/// # Returns
///
/// `AppResult<FileDiff>` - Diff results or error
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::{analyzers::get_analyzers, differ::generate_diff};
/// let diff = generate_diff("src/main.rs", &get_analyzers()).unwrap();
/// ```
pub fn generate_diff(file_path: &str, analyzers: &[Box<dyn Analyzer>]) -> AppResult<FileDiff> {
    let content = fs::read_to_string(file_path).map_err(IoError::from)?;
    let ast = syn::parse_file(&content).map_err(ParseError::from)?;

    let mut file_diff = FileDiff::new(file_path.to_string());

    for analyzer in analyzers {
        for suggestion in analyzer.suggestions(&ast, &content)? {
            file_diff.add_entry(entry_from_suggestion(analyzer.name(), &content, suggestion));
        }
    }

    Ok(file_diff)
}

/// Builds a displayable diff entry from a fix suggestion.
///
/// Derives the affected line number and its before/after text from the
/// suggestion's byte-range edit, and keeps the edit for application.
///
/// # Arguments
///
/// * `analyzer` - Name of the analyzer that produced the suggestion
/// * `content` - Original source code
/// * `suggestion` - Suggestion to render
///
/// # Returns
///
/// A `DiffEntry` for display and application
fn entry_from_suggestion(analyzer: &str, content: &str, suggestion: Suggestion) -> DiffEntry {
    let start = suggestion.edit.range.start;
    let end = suggestion.edit.range.end;

    let line = content[..start]
        .bytes()
        .filter(|&byte| byte == b'\n')
        .count()
        + 1;
    let line_start = content[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[start..]
        .find('\n')
        .map_or(content.len(), |index| start + index);

    let original = content[line_start..line_end].to_string();
    let rel_start = start - line_start;
    let rel_end = (end - line_start).min(original.len());
    let modified = format!(
        "{}{}{}",
        &original[..rel_start],
        suggestion.edit.replacement,
        &original[rel_end..]
    );

    DiffEntry {
        line,
        analyzer: analyzer.to_string(),
        original,
        modified,
        description: format!("{} fix", analyzer),
        import: suggestion.import,
        edit: suggestion.edit
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::analyzers::get_analyzers;

    #[test]
    fn test_generate_diff_integration() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_diff_no_issues() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_diff_invalid_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "fn main() { invalid syntax +++").unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers);

        assert!(result.is_err());
    }

    #[test]
    fn test_path_import_included_in_diff() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers).unwrap();

        assert!(
            result.entries.iter().any(|e| e.analyzer == "path_import"),
            "path_import should be included in diff with suggestions"
        );
    }

    #[test]
    fn test_format_args_excluded_from_diff_without_suggestion() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            "fn main() { println!(\"Hello {}\", \"world\"); }"
        )
        .unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers).unwrap();

        for entry in &result.entries {
            assert_ne!(entry.analyzer, "format_args");
        }
    }
}
