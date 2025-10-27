// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::fs;

use masterror::AppResult;

use super::types::{DiffEntry, FileDiff};
use crate::{
    analyzer::Analyzer,
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
        let result = analyzer.analyze(&ast, &content)?;

        for issue in result.issues {
            if issue.line == 0 || !issue.fix.is_available() {
                continue;
            }

            let original_content = content
                .lines()
                .nth(issue.line.saturating_sub(1))
                .unwrap_or("");

            let (modified_line, import) =
                if let Some((import, pattern, replacement)) = issue.fix.as_import() {
                    let modified = original_content.replace(pattern, replacement);
                    (modified, Some(import.to_string()))
                } else if let Some(simple) = issue.fix.as_simple() {
                    (simple.to_string(), None)
                } else {
                    continue;
                };

            let entry = DiffEntry {
                line: issue.line,
                analyzer: analyzer.name().to_string(),
                original: original_content.to_string(),
                modified: modified_line,
                description: issue.message,
                import
            };

            file_diff.add_entry(entry);
        }
    }

    Ok(file_diff)
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
