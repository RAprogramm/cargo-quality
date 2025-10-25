//! Report formatting for analysis results.
//!
//! Provides structured output of quality issues found during analysis,
//! grouping results by analyzer and file.

use std::fmt;

use crate::analyzer::AnalysisResult;

/// Report formatter for analysis results.
///
/// Aggregates results from multiple analyzers for a single file and
/// provides formatted output with issue counts and suggestions.
pub struct Report {
    /// File path being analyzed
    pub file_path: String,
    /// Analysis results grouped by analyzer name
    pub results: Vec<(String, AnalysisResult)>
}

impl Report {
    /// Create new report for a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to file being analyzed
    ///
    /// # Returns
    ///
    /// Empty report ready to accumulate results
    pub fn new(file_path: String) -> Self {
        Self { file_path, results: Vec::new() }
    }

    /// Add analysis result from an analyzer.
    ///
    /// # Arguments
    ///
    /// * `analyzer_name` - Name of analyzer that produced results
    /// * `result` - Analysis result to add
    pub fn add_result(&mut self, analyzer_name: String, result: AnalysisResult) {
        self.results.push((analyzer_name, result));
    }

    /// Calculate total issues across all analyzers.
    ///
    /// # Returns
    ///
    /// Sum of all issues found
    pub fn total_issues(&self) -> usize {
        self.results.iter().map(|(_, r)| r.issues.len()).sum()
    }

    /// Calculate total fixable issues across all analyzers.
    ///
    /// # Returns
    ///
    /// Sum of all fixable issues
    pub fn total_fixable(&self) -> usize {
        self.results.iter().map(|(_, r)| r.fixable_count).sum()
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Quality report for: {}", self.file_path)?;
        writeln!(f, "=")?;

        for (analyzer_name, result) in &self.results {
            if result.issues.is_empty() {
                continue;
            }

            writeln!(f, "\n[{}]", analyzer_name)?;
            for issue in &result.issues {
                write!(f, "  {}:{} - {}", issue.line, issue.column, issue.message)?;
                if let Some(suggestion) = &issue.suggestion {
                    write!(f, "\n    Suggestion: {}", suggestion)?;
                }
                writeln!(f)?;
            }
        }

        writeln!(f, "\nTotal issues: {}", self.total_issues())?;
        writeln!(f, "Fixable: {}", self.total_fixable())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Issue;

    use super::*;

    #[test]
    fn test_report_creation() {
        let report = Report::new("test.rs".to_string());
        assert_eq!(report.file_path, "test.rs");
        assert_eq!(report.results.len(), 0);
    }

    #[test]
    fn test_report_add_result() {
        let mut report = Report::new("test.rs".to_string());
        let result = AnalysisResult { issues: vec![], fixable_count: 0 };

        report.add_result("test_analyzer".to_string(), result);
        assert_eq!(report.results.len(), 1);
    }

    #[test]
    fn test_report_total_issues() {
        let mut report = Report::new("test.rs".to_string());

        let issue = Issue {
            line: 1,
            column: 1,
            message: "Test".to_string(),
            suggestion: None
        };

        let result = AnalysisResult { issues: vec![issue], fixable_count: 1 };

        report.add_result("analyzer1".to_string(), result);
        assert_eq!(report.total_issues(), 1);
        assert_eq!(report.total_fixable(), 1);
    }

    #[test]
    fn test_report_display_with_issues() {
        let mut report = Report::new("test.rs".to_string());

        let issue = Issue {
            line: 42,
            column: 15,
            message: "Test issue".to_string(),
            suggestion: Some("Fix suggestion".to_string())
        };

        let result = AnalysisResult { issues: vec![issue], fixable_count: 1 };

        report.add_result("test_analyzer".to_string(), result);

        let output = format!("{}", report);
        assert!(output.contains("Quality report for: test.rs"));
        assert!(output.contains("test_analyzer"));
        assert!(output.contains("42:15 - Test issue"));
        assert!(output.contains("Suggestion: Fix suggestion"));
        assert!(output.contains("Total issues: 1"));
        assert!(output.contains("Fixable: 1"));
    }

    #[test]
    fn test_report_display_without_issues() {
        let mut report = Report::new("test.rs".to_string());

        let result = AnalysisResult { issues: vec![], fixable_count: 0 };

        report.add_result("empty_analyzer".to_string(), result);

        let output = format!("{}", report);
        assert!(output.contains("Quality report for: test.rs"));
        assert!(!output.contains("empty_analyzer"));
        assert!(output.contains("Total issues: 0"));
        assert!(output.contains("Fixable: 0"));
    }

    #[test]
    fn test_report_display_issue_without_suggestion() {
        let mut report = Report::new("file.rs".to_string());

        let issue = Issue {
            line: 10,
            column: 5,
            message: "Warning message".to_string(),
            suggestion: None
        };

        let result = AnalysisResult { issues: vec![issue], fixable_count: 0 };

        report.add_result("warn_analyzer".to_string(), result);

        let output = format!("{}", report);
        assert!(output.contains("10:5 - Warning message"));
        assert!(!output.contains("Suggestion:"));
    }
}
