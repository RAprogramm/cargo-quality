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

/// Analysis issue found in code.
///
/// Represents a single quality issue detected by an analyzer, including
/// its location, description, and optional fix suggestion.
///
/// # Examples
///
/// ```
/// # use cargo_quality::analyzer::Issue;
/// let issue = Issue {
///     line:       42,
///     column:     15,
///     message:    "Use import instead of path".to_string(),
///     suggestion: Some("use std::fs::read_to_string;".to_string())
/// };
/// assert_eq!(issue.line, 42);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    /// Line number where issue was found
    pub line:       usize,
    /// Column number
    pub column:     usize,
    /// Issue description
    pub message:    String,
    /// Suggested fix
    pub suggestion: Option<String>
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
    fn test_issue_creation() {
        let issue = Issue {
            line:       42,
            column:     10,
            message:    "Test issue".to_string(),
            suggestion: Some("Fix suggestion".to_string())
        };

        assert_eq!(issue.line, 42);
        assert_eq!(issue.column, 10);
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult::default();
        assert_eq!(result.issues.len(), 0);
        assert_eq!(result.fixable_count, 0);
    }
}
