// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Empty lines analyzer for detecting blank lines inside function bodies.
//!
//! This analyzer identifies empty lines within function and method bodies,
//! which violate the Single Responsibility Principle by suggesting the
//! function does multiple things.

use std::collections::HashSet;

use masterror::AppResult;
use syn::{File, ImplItem, ItemFn, ItemImpl, spanned::Spanned, visit::Visit};

use super::{
    line_deletion_range, line_offsets,
    visitor::{FunctionVisitor, ItemCheckers, SourceView}
};
use crate::analyzer::{AnalysisResult, Analyzer, Fix, Issue, Suggestion, TextEdit};

/// Analyzer for detecting empty lines inside functions and methods.
///
/// Finds blank lines within function bodies that indicate a function
/// is doing multiple things and should be refactored into smaller functions.
///
/// # Examples
///
/// Detects this pattern:
/// ```ignore
/// fn process() {
///     let x = read_data();
///
///     let y = transform(x);
/// }
/// ```
///
/// Suggests removing the empty line or refactoring into separate functions.
pub struct EmptyLinesAnalyzer;

impl EmptyLinesAnalyzer {
    /// Create new empty lines analyzer instance.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Check function body for empty lines.
    ///
    /// Analyzes source code to find empty lines within function boundaries.
    ///
    /// # Arguments
    ///
    /// * `func` - Function item to analyze
    /// * `lines` - Source code split into lines
    ///
    /// # Returns
    ///
    /// Vector of issues found
    fn check_block(
        start_line: usize,
        end_line: usize,
        lines: &[&str],
        excluded: &HashSet<usize>
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        if start_line >= end_line {
            return issues;
        }

        for line_num in start_line..end_line {
            if excluded.contains(&line_num) {
                continue;
            }

            let idx = line_num.saturating_sub(1);

            let Some(line) = lines.get(idx) else {
                continue;
            };

            if line.trim().is_empty() {
                let is_first = line_num == start_line;
                let is_last = line_num == end_line.saturating_sub(1);

                if is_first || is_last {
                    continue;
                }

                if Self::is_after_opening_brace(lines, idx)
                    || Self::is_before_closing_brace(lines, idx)
                {
                    continue;
                }

                issues.push(Issue::new(
                    line_num,
                    1,
                    "Empty line in function body indicates untamed complexity".to_string(),
                    Fix::Simple("Remove empty line".to_string())
                ));
            }
        }

        issues
    }

    /// Check if empty line is right after opening brace.
    ///
    /// Handles both same-line and next-line brace styles.
    ///
    /// # Arguments
    ///
    /// * `lines` - Source code lines
    /// * `idx` - Index of empty line (0-based)
    #[inline]
    fn is_after_opening_brace(lines: &[&str], idx: usize) -> bool {
        if idx == 0 {
            return false;
        }

        let prev_idx = idx.saturating_sub(1);

        if let Some(prev) = lines.get(prev_idx) {
            let trimmed = prev.trim();
            if trimmed.ends_with('{') || trimmed == "{" {
                return true;
            }
        }

        false
    }

    /// Check if empty line is right before closing brace.
    ///
    /// Handles both same-line and next-line brace styles.
    ///
    /// # Arguments
    ///
    /// * `lines` - Source code lines
    /// * `idx` - Index of empty line (0-based)
    #[inline]
    fn is_before_closing_brace(lines: &[&str], idx: usize) -> bool {
        let next_idx = idx + 1;

        if let Some(next) = lines.get(next_idx) {
            let trimmed = next.trim();
            if trimmed == "}" || trimmed.starts_with('}') {
                return true;
            }
        }

        false
    }

    /// Check standalone function for empty lines.
    ///
    /// # Arguments
    ///
    /// * `func` - Function item to analyze
    /// * `lines` - Source code split into lines
    fn check_function(func: &ItemFn, lines: &[&str], excluded: &HashSet<usize>) -> Vec<Issue> {
        let span = func.block.span();
        let start_line = span.start().line;
        let end_line = span.end().line;

        Self::check_block(start_line, end_line, lines, excluded)
    }

    /// Check impl block methods for empty lines.
    ///
    /// # Arguments
    ///
    /// * `impl_block` - Impl block to analyze
    /// * `lines` - Source code split into lines
    fn check_impl_block(
        impl_block: &ItemImpl,
        lines: &[&str],
        excluded: &HashSet<usize>
    ) -> Vec<Issue> {
        let mut issues = Vec::new();

        for item in &impl_block.items {
            if let ImplItem::Fn(method) = item {
                let span = method.block.span();
                let start_line = span.start().line;
                let end_line = span.end().line;

                issues.extend(Self::check_block(start_line, end_line, lines, excluded));
            }
        }

        issues
    }
}

impl Analyzer for EmptyLinesAnalyzer {
    fn name(&self) -> &'static str {
        "empty_lines"
    }

    fn analyze(&self, ast: &File, content: &str) -> AppResult<AnalysisResult> {
        let lines: Vec<&str> = content.lines().collect();
        let excluded = crate::analyzers::multiline_literal_lines(ast);
        let mut visitor = FunctionVisitor {
            issues:   Vec::new(),
            source:   SourceView {
                lines:    &lines,
                excluded: &excluded
            },
            checkers: ItemCheckers {
                function:   Self::check_function,
                impl_block: Self::check_impl_block
            }
        };
        visitor.visit_file(ast);
        let fixable_count = visitor.issues.len();

        Ok(AnalysisResult {
            issues: visitor.issues,
            fixable_count
        })
    }

    fn suggestions(&self, ast: &File, content: &str) -> AppResult<Vec<Suggestion>> {
        let result = self.analyze(ast, content)?;
        let offsets = line_offsets(content);
        let mut seen = HashSet::new();
        let mut suggestions = Vec::new();
        for issue in result.issues {
            let line = issue.diagnostic.line;
            if !seen.insert(line) {
                continue;
            }
            let Some(range) = line_deletion_range(&offsets, content.len(), line) else {
                continue;
            };
            suggestions.push(Suggestion {
                edit:   TextEdit {
                    range,
                    replacement: String::new()
                },
                import: None
            });
        }
        Ok(suggestions)
    }
}

impl Default for EmptyLinesAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_name() {
        let analyzer = EmptyLinesAnalyzer::new();
        assert_eq!(analyzer.name(), "empty_lines");
    }

    #[test]
    fn test_ignore_blank_line_inside_string_literal() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = "fn f() {\n    let s = \"line one\n\nline two\";\n    let _ = s;\n}";
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_empty_line_in_function() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;

    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_ignore_function_without_empty_lines() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;
    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_empty_line_after_opening_brace() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {

    let x = 1;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_empty_line_before_closing_brace() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;

}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_multiple_empty_lines() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn process() {
    let x = read();

    let y = transform(x);

    write(y);
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_single_line_function() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = "fn main() { let x = 1; }";
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_empty_function() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = "fn main() {}";
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_issues_are_fixable() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;

    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.fixable_count, 1);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].fix.is_available());
    }

    #[test]
    fn test_suggestions_delete_empty_line() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = "fn main() {\n    let x = 1;\n\n    let y = 2;\n}";
        let code = syn::parse_str(content).unwrap();

        let suggestions = analyzer.suggestions(&code, content).unwrap();
        assert_eq!(suggestions.len(), 1);

        let fixed = crate::fixer::apply_suggestions(content, &suggestions);
        assert_eq!(fixed, "fn main() {\n    let x = 1;\n    let y = 2;\n}");
    }

    #[test]
    fn test_suggestions_delete_multiple_lines_bottom_up() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content =
            "fn process() {\n    let x = read();\n\n    let y = transform(x);\n\n    write(y);\n}";
        let code = syn::parse_str(content).unwrap();

        let suggestions = analyzer.suggestions(&code, content).unwrap();
        assert_eq!(suggestions.len(), 2);

        let fixed = crate::fixer::apply_suggestions(content, &suggestions);
        assert_eq!(
            fixed,
            "fn process() {\n    let x = read();\n    let y = transform(x);\n    write(y);\n}"
        );
    }

    #[test]
    fn test_default_implementation() {
        let analyzer = EmptyLinesAnalyzer;
        assert_eq!(analyzer.name(), "empty_lines");
    }

    #[test]
    fn test_nested_blocks() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    if true {
        let x = 1;

        let y = 2;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_multiple_functions() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn first() {
    let x = 1;

    let y = 2;
}

fn second() {
    let a = 3;

    let b = 4;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_detect_empty_line_in_method() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"struct Foo;

impl Foo {
    fn method(&self) {
        let x = 1;

        let y = 2;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].diagnostic.line, 6);
    }

    #[test]
    fn test_ignore_empty_line_after_opening_brace_on_new_line() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main()
{

    let x = 1;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_empty_line_before_closing_brace_on_own_line() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main()
{
    let x = 1;

}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_formatted_code_with_braces_on_new_lines() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main()
{
    let x = 1;

    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].diagnostic.line, 4);
    }

    #[test]
    fn test_impl_block_with_multiple_methods() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"struct Foo;

impl Foo
{
    fn first(&self)
    {
        let a = 1;

        let b = 2;
    }

    fn second(&self)
    {
        let x = 3;

        let y = 4;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_ignore_empty_lines_between_methods() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"struct Foo;

impl Foo {
    fn first(&self) {
        let x = 1;
    }

    fn second(&self) {
        let y = 2;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }
}
