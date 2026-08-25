// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Core analyzer trait and types for code quality analysis.
//!
//! This module defines the fundamental abstractions for building code
//! analyzers:
//! - `Analyzer` trait that all analyzers must implement
//! - `Issue` struct representing detected problems
//! - `AnalysisResult` struct containing analysis outcomes

use std::ops::Range;

use masterror::AppResult;
use syn::File;

/// A single text replacement over the original source.
///
/// Fixes are expressed as byte-range edits against the untouched source text so
/// that everything outside the edited range — comments, blank lines, and the
/// author's formatting — is preserved. This mirrors how `rustfmt` and
/// `rust-analyzer` apply changes, rather than reprinting the AST (which loses
/// comments and reformats the whole file).
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzer::TextEdit;
///
/// let edit = TextEdit {
///     range:       0..9,
///     replacement: String::new()
/// };
/// assert_eq!(edit.range.len(), 9);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextEdit {
    /// Byte range in the original source to replace
    pub range:       Range<usize>,
    /// Text to substitute for the range (empty to delete)
    pub replacement: String
}

/// A `use` statement insertion anchored to a specific byte offset.
///
/// The offset addresses the module that must receive the import — the top of
/// the file for top-level rewrites, or the first item of an inline module for
/// rewrites inside it — so the inserted name is always in scope at the rewrite
/// site.
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzer::ImportEdit;
///
/// let import = ImportEdit {
///     offset:    0,
///     statement: "use std::fs::read;".to_string()
/// };
/// assert!(import.statement.starts_with("use "));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEdit {
    /// Byte offset in the original source at which to insert the statement
    pub offset:    usize,
    /// The `use` statement to insert, without a trailing newline
    pub statement: String
}

/// A single fixable change: one source edit plus any import it requires.
///
/// Both the `fix` command and the diff/interactive flow are built from
/// suggestions, so applying a change is identical everywhere: the [`edit`] is
/// spliced into the source and the [`import`], if any, is inserted once per
/// target offset (imports are deduplicated across the applied suggestions).
///
/// [`edit`]: Suggestion::edit
/// [`import`]: Suggestion::import
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The byte-range edit that performs the rewrite
    pub edit:   TextEdit,
    /// A `use` statement the rewrite depends on, if any
    pub import: Option<ImportEdit>
}

/// Type of fix that can be applied to resolve an issue.
///
/// Represents different kinds of automatic fixes that analyzers can provide.
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzer::Fix;
///
/// let simple_fix = Fix::Simple("let x = 42;".to_string());
/// assert!(simple_fix.is_available());
/// assert_eq!(simple_fix.as_simple(), Some("let x = 42;"));
///
/// let import_fix = Fix::WithImport {
///     import:      "use std::fs::read;".to_string(),
///     pattern:     "std::fs::read".to_string(),
///     replacement: "read".to_string()
/// };
/// assert!(import_fix.is_available());
/// assert_eq!(
///     import_fix.as_import(),
///     Some(("use std::fs::read;", "std::fs::read", "read"))
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Fix {
    /// No automatic fix available
    None,

    /// Simple line replacement
    ///
    /// Replace the entire line with the provided string.
    ///
    /// Note: Reserved for future analyzers that need simple line replacements.
    #[allow(dead_code)]
    Simple(String),

    /// Fix requiring import addition
    ///
    /// Adds an import statement and replaces the line.
    WithImport {
        /// Import statement to add (e.g., "use std::fs::read_to_string;")
        import:      String,
        /// Pattern to find in original line (e.g., "std::fs::read_to_string")
        pattern:     String,
        /// Replacement for the pattern (e.g., "read_to_string")
        replacement: String
    }
}

impl Fix {
    /// Checks if fix is available.
    ///
    /// # Returns
    ///
    /// `true` if fix can be applied automatically
    #[inline]
    pub fn is_available(&self) -> bool {
        !matches!(self, Fix::None)
    }

    /// Returns simple replacement string if available.
    ///
    /// # Returns
    ///
    /// Option<&str> - Replacement string for simple fixes
    #[inline]
    pub fn as_simple(&self) -> Option<&str> {
        match self {
            Fix::Simple(s) => Some(s.as_str()),
            _ => None
        }
    }

    /// Returns import, pattern, and replacement for import-based fixes.
    ///
    /// # Returns
    ///
    /// Option<(&str, &str, &str)> - (import, pattern, replacement) tuple
    #[inline]
    pub fn as_import(&self) -> Option<(&str, &str, &str)> {
        match self {
            Fix::WithImport {
                import,
                pattern,
                replacement
            } => Some((import.as_str(), pattern.as_str(), replacement.as_str())),
            _ => None
        }
    }
}

/// Location and description of a single finding in a source file.
///
/// Shared by every issue type so that reporting code works with one shape.
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzer::Diagnostic;
///
/// let diagnostic = Diagnostic {
///     line:    42,
///     column:  15,
///     message: "Use import instead of path".to_string()
/// };
/// assert_eq!(diagnostic.line, 42);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Line number where issue was found
    pub line:    usize,
    /// Column number
    pub column:  usize,
    /// Issue description
    pub message: String
}

/// Analysis issue found in code.
///
/// Represents a single quality issue detected by an analyzer, including
/// its location, description, and optional fix.
///
/// # Examples
///
/// ```
/// # use cargo_quality::analyzer::{Fix, Issue};
/// let issue = Issue::new(
///     42,
///     15,
///     "Use import instead of path".to_string(),
///     Fix::WithImport {
///         import:      "use std::fs::read_to_string;".to_string(),
///         pattern:     "std::fs::read_to_string".to_string(),
///         replacement: "read_to_string".to_string()
///     }
/// );
/// assert_eq!(issue.diagnostic.line, 42);
/// assert!(issue.fix.is_available());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    /// Where the issue was found and what it says
    pub diagnostic: Diagnostic,
    /// Automatic fix
    pub fix:        Fix
}

impl Issue {
    /// Creates an issue from its location, message, and fix.
    ///
    /// # Arguments
    ///
    /// * `line` - Line number where the issue was found
    /// * `column` - Column number
    /// * `message` - Issue description
    /// * `fix` - Automatic fix, or [`Fix::None`]
    ///
    /// # Returns
    ///
    /// The assembled issue
    #[inline]
    pub fn new(line: usize, column: usize, message: String, fix: Fix) -> Self {
        Self {
            diagnostic: Diagnostic {
                line,
                column,
                message
            },
            fix
        }
    }
}

/// Result of code analysis.
///
/// Contains all issues found during analysis and count of fixable issues.
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzer::AnalysisResult;
///
/// let result = AnalysisResult {
///     issues:        vec![],
///     fixable_count: 0
/// };
/// assert_eq!(result.issues.len(), 0);
/// ```
#[derive(Debug, Default)]
pub struct AnalysisResult {
    /// Issues found
    pub issues:        Vec<Issue>,
    /// Number of fixable issues
    pub fixable_count: usize
}

/// Trait for code analyzers.
///
/// Implement this trait to create custom quality analyzers. Each analyzer
/// must provide a unique name, analysis logic, and optional fix capability.
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzer::{AnalysisResult, Analyzer};
/// use masterror::AppResult;
/// use syn::File;
///
/// struct MyAnalyzer;
///
/// impl Analyzer for MyAnalyzer {
///     fn name(&self) -> &'static str {
///         "my_analyzer"
///     }
///
///     fn analyze(&self, ast: &File, content: &str) -> AppResult<AnalysisResult> {
///         Ok(AnalysisResult::default())
///     }
/// }
/// ```
pub trait Analyzer {
    /// Returns unique analyzer identifier.
    ///
    /// Used for reporting and configuration. Must be lowercase snake_case.
    fn name(&self) -> &'static str;

    /// Analyze Rust syntax tree for quality issues.
    ///
    /// # Arguments
    ///
    /// * `ast` - Parsed Rust syntax tree to analyze
    /// * `content` - Source code content for analyzers that need raw text
    ///
    /// # Returns
    ///
    /// `AppResult<AnalysisResult>` - Analysis results or error
    fn analyze(&self, ast: &File, content: &str) -> AppResult<AnalysisResult>;

    /// Produce fixable suggestions for the detected issues.
    ///
    /// Each suggestion is a byte-range edit (plus an optional import) applied
    /// against the original source, preserving everything outside the edited
    /// ranges (comments, blank lines, formatting). The default implementation
    /// returns none, for analyzers that are advisory only.
    ///
    /// # Arguments
    ///
    /// * `ast` - Parsed Rust syntax tree to fix
    /// * `content` - Original source code the edits apply to
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<Suggestion>>` - Non-overlapping suggestions, or error
    fn suggestions(&self, _ast: &File, _content: &str) -> AppResult<Vec<Suggestion>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_none() {
        let fix = Fix::None;
        assert!(!fix.is_available());
        assert!(fix.as_simple().is_none());
        assert!(fix.as_import().is_none());
    }

    #[test]
    fn test_fix_simple() {
        let fix = Fix::Simple("replacement".to_string());
        assert!(fix.is_available());
        assert_eq!(fix.as_simple(), Some("replacement"));
        assert!(fix.as_import().is_none());
    }

    #[test]
    fn test_fix_with_import() {
        let fix = Fix::WithImport {
            import:      "use std::fs::read;".to_string(),
            pattern:     "std::fs::read".to_string(),
            replacement: "read".to_string()
        };
        assert!(fix.is_available());
        assert!(fix.as_simple().is_none());
        assert_eq!(
            fix.as_import(),
            Some(("use std::fs::read;", "std::fs::read", "read"))
        );
    }

    #[test]
    fn test_issue_creation() {
        let issue = Issue::new(
            42,
            10,
            "Test issue".to_string(),
            Fix::Simple("Fix suggestion".to_string())
        );

        assert_eq!(issue.diagnostic.line, 42);
        assert_eq!(issue.diagnostic.column, 10);
        assert!(issue.fix.is_available());
    }

    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult::default();
        assert_eq!(result.issues.len(), 0);
        assert_eq!(result.fixable_count, 0);
    }
}
