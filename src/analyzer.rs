// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Core analyzer trait and types for code quality analysis.
//!
//! This module defines the fundamental abstractions for building code
//! analyzers:
//! - `Analyzer` trait that all analyzers must implement
//! - `Issue` struct representing detected problems
//! - `AnalysisResult` struct containing analysis outcomes

use masterror::AppResult;
use syn::File;

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
///     import:      "use std::fs;".to_string(),
///     replacement: "fs::read()".to_string()
/// };
/// assert!(import_fix.is_available());
/// assert_eq!(import_fix.as_import(), Some(("use std::fs;", "fs::read()")));
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
        /// Replacement for the current line
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

    /// Returns import and replacement for import-based fixes.
    ///
    /// # Returns
    ///
    /// Option<(&str, &str)> - (import, replacement) tuple
    #[inline]
    pub fn as_import(&self) -> Option<(&str, &str)> {
        match self {
            Fix::WithImport {
                import,
                replacement
            } => Some((import.as_str(), replacement.as_str())),
            _ => None
        }
    }
}

/// Analysis issue found in code.
///
/// Represents a single quality issue detected by an analyzer, including
/// its location, description, and optional fix.
///
/// # Examples
///
/// ```
/// # use cargo_quality::analyzer::{Issue, Fix};
/// let issue = Issue {
///     line:    42,
///     column:  15,
///     message: "Use import instead of path".to_string(),
///     fix:     Fix::WithImport {
///         import:      "use std::fs::read_to_string;".to_string(),
///         replacement: "read_to_string(\"file.txt\")".to_string()
///     }
/// };
/// assert_eq!(issue.line, 42);
/// assert!(issue.fix.is_available());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    /// Line number where issue was found
    pub line:    usize,
    /// Column number
    pub column:  usize,
    /// Issue description
    pub message: String,
    /// Automatic fix
    pub fix:     Fix
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
///     fn analyze(&self, ast: &File) -> AppResult<AnalysisResult> {
///         Ok(AnalysisResult::default())
///     }
///
///     fn fix(&self, ast: &mut File) -> AppResult<usize> {
///         Ok(0)
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
    ///
    /// # Returns
    ///
    /// `AppResult<AnalysisResult>` - Analysis results or error
    fn analyze(&self, ast: &File) -> AppResult<AnalysisResult>;

    /// Apply automatic fixes to syntax tree.
    ///
    /// Modifies the AST in-place to fix detected issues.
    ///
    /// # Arguments
    ///
    /// * `ast` - Mutable syntax tree to fix
    ///
    /// # Returns
    ///
    /// `AppResult<usize>` - Number of fixes applied or error
    fn fix(&self, ast: &mut File) -> AppResult<usize>;
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
            import:      "use std::fs;".to_string(),
            replacement: "fs::read()".to_string()
        };
        assert!(fix.is_available());
        assert!(fix.as_simple().is_none());
        assert_eq!(fix.as_import(), Some(("use std::fs;", "fs::read()")));
    }

    #[test]
    fn test_issue_creation() {
        let issue = Issue {
            line:    42,
            column:  10,
            message: "Test issue".to_string(),
            fix:     Fix::Simple("Fix suggestion".to_string())
        };

        assert_eq!(issue.line, 42);
        assert_eq!(issue.column, 10);
        assert!(issue.fix.is_available());
    }

    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult::default();
        assert_eq!(result.issues.len(), 0);
        assert_eq!(result.fixable_count, 0);
    }
}
