// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

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
    pub results:   Vec<(String, AnalysisResult)>
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
        Self {
            file_path,
            results: Vec::new()
        }
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
                if issue.fix.is_available() {
                    if let Some((import, _pattern, _replacement)) = issue.fix.as_import() {
                        write!(f, "\n    Fix: Add import: {}", import)?;
                        write!(f, "\n    (Will replace path with short name)")?;
                    } else if let Some(simple) = issue.fix.as_simple() {
                        write!(f, "\n    Fix: {}", simple)?;
                    }
                }
                writeln!(f)?;
            }
        }

        writeln!(f, "\nTotal issues: {}", self.total_issues())?;
        writeln!(f, "Fixable: {}", self.total_fixable())?;

        Ok(())
    }
}

/// Global report aggregator across multiple files.
///
/// Collects reports from multiple files and provides globally grouped output.
pub struct GlobalReport {
    /// Collection of per-file reports
    pub reports: Vec<Report>
}

impl GlobalReport {
    /// Create new global report.
    pub fn new() -> Self {
        Self {
            reports: Vec::new()
        }
    }

    /// Add a file report to the global collection.
    pub fn add_report(&mut self, report: Report) {
        self.reports.push(report);
    }

    /// Calculate total issues across all files.
    pub fn total_issues(&self) -> usize {
        self.reports.iter().map(|r| r.total_issues()).sum()
    }

    /// Calculate total fixable issues across all files.
    pub fn total_fixable(&self) -> usize {
        self.reports.iter().map(|r| r.total_fixable()).sum()
    }

    /// Display globally grouped report (compact mode).
    ///
    /// Groups issues by analyzer and message across all files,
    /// then shows which files have each issue.
    pub fn display_compact(&self) -> String {
        use std::collections::HashMap;

        type FileLines = Vec<(String, Vec<usize>)>;
        type MessageGroups = HashMap<String, FileLines>;
        type AnalyzerGroups = HashMap<String, MessageGroups>;

        let mut output = String::new();

        let mut analyzer_groups: AnalyzerGroups = HashMap::new();

        for report in &self.reports {
            for (analyzer_name, result) in &report.results {
                if result.issues.is_empty() {
                    continue;
                }

                let message_map = analyzer_groups.entry(analyzer_name.clone()).or_default();

                for issue in &result.issues {
                    let file_list = message_map.entry(issue.message.clone()).or_default();

                    if let Some((_, lines)) =
                        file_list.iter_mut().find(|(f, _)| f == &report.file_path)
                    {
                        lines.push(issue.line);
                    } else {
                        file_list.push((report.file_path.clone(), vec![issue.line]));
                    }
                }
            }
        }

        let mut analyzer_names: Vec<_> = analyzer_groups.keys().collect();
        analyzer_names.sort();

        for analyzer_name in analyzer_names {
            let message_map = &analyzer_groups[analyzer_name];
            let total_issues: usize = message_map
                .values()
                .map(|files| files.iter().map(|(_, lines)| lines.len()).sum::<usize>())
                .sum();

            output.push_str(&format!(
                "\n[{}] - {} issues\n",
                analyzer_name, total_issues
            ));

            for (message, file_list) in message_map {
                output.push_str(&format!("  {}\n", message));

                for (file_path, mut lines) in file_list.iter().map(|(f, l)| (f, l.clone())) {
                    lines.sort_unstable();
                    output.push_str(&format!("  {} → Lines: ", file_path));

                    let lines_str: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
                    let joined = lines_str.join(", ");

                    if joined.len() > 70 {
                        let mut line_chunks = Vec::new();
                        let mut current_line = String::new();

                        for (i, line_num) in lines_str.iter().enumerate() {
                            let separator = if i == 0 { "" } else { ", " };
                            let addition = format!("{}{}", separator, line_num);

                            if current_line.len() + addition.len() > 70 && !current_line.is_empty()
                            {
                                line_chunks.push(current_line.clone());
                                current_line = line_num.clone();
                            } else {
                                current_line.push_str(&addition);
                            }
                        }

                        if !current_line.is_empty() {
                            line_chunks.push(current_line);
                        }

                        for (i, chunk) in line_chunks.iter().enumerate() {
                            if i == 0 {
                                output.push_str(&format!("{}\n", chunk));
                            } else {
                                let indent = " ".repeat(file_path.len() + 11);
                                output.push_str(&format!("{}{}\n", indent, chunk));
                            }
                        }
                    } else {
                        output.push_str(&format!("{}\n", joined));
                    }
                }

                output.push('\n');
            }
        }

        output.push_str(&format!("Total issues: {}\n", self.total_issues()));
        output.push_str(&format!("Fixable: {}\n", self.total_fixable()));

        output
    }

    /// Display globally grouped report (verbose mode).
    ///
    /// Shows all reports in full detail, one file at a time.
    pub fn display_verbose(&self) -> String {
        let mut output = String::new();
        for report in &self.reports {
            output.push_str(&format!("{}", report));
        }
        output.push_str(&format!("\nTotal issues: {}\n", self.total_issues()));
        output.push_str(&format!("Fixable: {}\n", self.total_fixable()));
        output
    }
}

impl Default for GlobalReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Issue;

    #[test]
    fn test_report_creation() {
        let report = Report::new("test.rs".to_string());
        assert_eq!(report.file_path, "test.rs");
        assert_eq!(report.results.len(), 0);
    }

    #[test]
    fn test_report_add_result() {
        let mut report = Report::new("test.rs".to_string());
        let result = AnalysisResult {
            issues:        vec![],
            fixable_count: 0
        };

        report.add_result("test_analyzer".to_string(), result);
        assert_eq!(report.results.len(), 1);
    }

    #[test]
    fn test_report_total_issues() {
        let mut report = Report::new("test.rs".to_string());

        let issue = Issue {
            line:    1,
            column:  1,
            message: "Test".to_string(),
            fix:     crate::analyzer::Fix::None
        };

        let result = AnalysisResult {
            issues:        vec![issue],
            fixable_count: 1
        };

        report.add_result("analyzer1".to_string(), result);
        assert_eq!(report.total_issues(), 1);
        assert_eq!(report.total_fixable(), 1);
    }

    #[test]
    fn test_report_display_with_issues() {
        let mut report = Report::new("test.rs".to_string());

        let issue = Issue {
            line:    42,
            column:  15,
            message: "Test issue".to_string(),
            fix:     crate::analyzer::Fix::Simple("Fix suggestion".to_string())
        };

        let result = AnalysisResult {
            issues:        vec![issue],
            fixable_count: 1
        };

        report.add_result("test_analyzer".to_string(), result);

        let output = format!("{}", report);
        assert!(output.contains("Quality report for: test.rs"));
        assert!(output.contains("test_analyzer"));
        assert!(output.contains("42:15 - Test issue"));
        assert!(output.contains("Fix: Fix suggestion"));
        assert!(output.contains("Total issues: 1"));
        assert!(output.contains("Fixable: 1"));
    }

    #[test]
    fn test_report_display_without_issues() {
        let mut report = Report::new("test.rs".to_string());

        let result = AnalysisResult {
            issues:        vec![],
            fixable_count: 0
        };

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
            line:    10,
            column:  5,
            message: "Warning message".to_string(),
            fix:     crate::analyzer::Fix::None
        };

        let result = AnalysisResult {
            issues:        vec![issue],
            fixable_count: 0
        };

        report.add_result("warn_analyzer".to_string(), result);

        let output = format!("{}", report);
        assert!(output.contains("10:5 - Warning message"));
        assert!(!output.contains("Fix:"));
    }

    #[test]
    fn test_report_multiple_analyzers() {
        let mut report = Report::new("code.rs".to_string());

        let issue1 = Issue {
            line:    1,
            column:  1,
            message: "Issue 1".to_string(),
            fix:     crate::analyzer::Fix::Simple("Fix 1".to_string())
        };

        let issue2 = Issue {
            line:    2,
            column:  2,
            message: "Issue 2".to_string(),
            fix:     crate::analyzer::Fix::None
        };

        report.add_result(
            "analyzer1".to_string(),
            AnalysisResult {
                issues:        vec![issue1],
                fixable_count: 1
            }
        );

        report.add_result(
            "analyzer2".to_string(),
            AnalysisResult {
                issues:        vec![issue2],
                fixable_count: 0
            }
        );

        assert_eq!(report.total_issues(), 2);
        assert_eq!(report.total_fixable(), 1);

        let output = format!("{}", report);
        assert!(output.contains("analyzer1"));
        assert!(output.contains("analyzer2"));
        assert!(output.contains("Total issues: 2"));
    }

    #[test]
    fn test_report_total_fixable() {
        let mut report = Report::new("test.rs".to_string());

        report.add_result(
            "analyzer1".to_string(),
            AnalysisResult {
                issues:        vec![],
                fixable_count: 3
            }
        );

        report.add_result(
            "analyzer2".to_string(),
            AnalysisResult {
                issues:        vec![],
                fixable_count: 2
            }
        );

        assert_eq!(report.total_fixable(), 5);
    }
}
