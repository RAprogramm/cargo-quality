// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Empty lines analyzer for detecting blank lines inside function bodies.
//!
//! This analyzer identifies empty lines within function and method bodies,
//! which violate the Single Responsibility Principle by suggesting the
//! function does multiple things.

use masterror::AppResult;
use syn::{File, Item, ItemFn, spanned::Spanned, visit::Visit};

use crate::analyzer::{AnalysisResult, Analyzer, Fix, Issue};

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
    /// * `content` - Source code content
    ///
    /// # Returns
    ///
    /// Vector of issues found
    fn check_function(func: &ItemFn, content: &str) -> Vec<Issue> {
        let mut issues = Vec::new();
        let span = func.block.span();
        let start_line = span.start().line;
        let end_line = span.end().line;

        if start_line >= end_line {
            return issues;
        }

        let lines: Vec<&str> = content.lines().collect();

        for line_num in start_line..end_line {
            let idx = line_num.saturating_sub(1);

            let Some(line) = lines.get(idx) else {
                continue;
            };

            if line.trim().is_empty() {
                let prev_idx = idx.saturating_sub(1);
                let next_idx = idx + 1;

                let is_first = line_num == start_line;
                let is_last = line_num == end_line.saturating_sub(1);

                if is_first || is_last {
                    continue;
                }

                let prev_is_brace = lines
                    .get(prev_idx)
                    .map(|l| l.trim().ends_with('{'))
                    .unwrap_or(false);
                let next_is_brace = lines
                    .get(next_idx)
                    .map(|l| l.trim() == "}")
                    .unwrap_or(false);

                if prev_is_brace || next_is_brace {
                    continue;
                }

                issues.push(Issue {
                    line:    line_num,
                    column:  1,
                    message: "Empty line in function body indicates untamed complexity"
                        .to_string(),
                    fix:     Fix::Simple(String::new())
                });
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
        let mut visitor = FunctionVisitor {
            issues:  Vec::new(),
            content: content.to_string()
        };
        visitor.visit_file(ast);

        let fixable_count = visitor.issues.len();

        Ok(AnalysisResult {
            issues: visitor.issues,
            fixable_count
        })
    }

    fn fix(&self, _ast: &mut File) -> AppResult<usize> {
        Ok(0)
    }
}

struct FunctionVisitor {
    issues:  Vec<Issue>,
    content: String
}

impl<'ast> Visit<'ast> for FunctionVisitor {
    fn visit_item(&mut self, node: &'ast Item) {
        if let Item::Fn(func) = node {
            let func_issues = EmptyLinesAnalyzer::check_function(func, &self.content);
            self.issues.extend(func_issues);
        }
        syn::visit::visit_item(self, node);
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
    fn test_fixable_count() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;

    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.fixable_count, 1);
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_fix_returns_zero() {
        let analyzer = EmptyLinesAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;

    let y = 2;
}"#;
        let mut code = syn::parse_str(content).unwrap();

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 0);
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
}
