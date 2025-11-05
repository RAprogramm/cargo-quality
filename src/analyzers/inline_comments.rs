// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Inline comments analyzer for detecting non-doc comments in function bodies.
//!
//! This analyzer identifies inline comments (`//`) within function and method
//! bodies, which violate the documentation standards. All explanations should
//! be in doc comments (`///`), specifically in the `# Notes` section.

use masterror::AppResult;
use syn::{File, ImplItem, Item, ItemFn, ItemImpl, spanned::Spanned, visit::Visit};

use crate::analyzer::{AnalysisResult, Analyzer, Fix, Issue};

/// Analyzer for detecting inline comments inside functions and methods.
///
/// Finds non-doc comments within function bodies and suggests moving them
/// to the function's doc block `# Notes` section with code context.
///
/// # Examples
///
/// Detects this pattern:
/// ```ignore
/// fn calculate() {
///     let x = read_data();
///     // Process the data
///     let y = transform(x);
/// }
/// ```
///
/// Suggests adding to doc block:
/// ```ignore
/// /// Calculate something
/// ///
/// /// # Notes
/// ///
/// /// - Line 3: `let y = transform(x);` - Process the data
/// fn calculate() {
///     let x = read_data();
///     let y = transform(x);
/// }
/// ```
pub struct InlineCommentsAnalyzer;

impl InlineCommentsAnalyzer {
    /// Create new inline comments analyzer instance.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Check function body for inline comments.
    ///
    /// Analyzes source code to find inline comments within function boundaries
    /// and creates issues with suggestions to move them to doc blocks.
    ///
    /// # Arguments
    ///
    /// * `start_line` - First line of function body
    /// * `end_line` - Last line of function body
    /// * `content` - Source code content
    ///
    /// # Returns
    ///
    /// Vector of issues found
    fn check_block(start_line: usize, end_line: usize, content: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        if start_line >= end_line {
            return issues;
        }

        let lines: Vec<&str> = content.lines().collect();

        for line_num in start_line..end_line {
            let idx = line_num.saturating_sub(1);

            let Some(line) = lines.get(idx) else {
                continue;
            };

            let trimmed = line.trim();

            if trimmed.starts_with("//") && !trimmed.starts_with("///") {
                let comment_text = trimmed.trim_start_matches("//").trim();

                let code_line = Self::find_related_code_line(&lines, idx);

                let suggestion = if let Some((_code_idx, code)) = code_line {
                    format!(
                        "Move to doc block # Notes section:\n/// - {} - `{}`",
                        comment_text,
                        code.trim()
                    )
                } else {
                    format!("Move to doc block # Notes section:\n/// - {}", comment_text)
                };

                issues.push(Issue {
                    line:    line_num,
                    column:  1,
                    message: format!("Inline comment found: \"{}\"\n{}", comment_text, suggestion),
                    fix:     Fix::None
                });
            }
        }

        issues
    }

    /// Find the code line that this comment describes.
    ///
    /// Looks for the next non-empty, non-comment line after the comment.
    ///
    /// # Arguments
    ///
    /// * `lines` - All source code lines
    /// * `comment_idx` - Index of the comment line (0-based)
    ///
    /// # Returns
    ///
    /// Option with (line_index, line_content) of related code
    fn find_related_code_line<'a>(
        lines: &[&'a str],
        comment_idx: usize
    ) -> Option<(usize, &'a str)> {
        for (offset, line) in lines.iter().enumerate().skip(comment_idx + 1) {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            if !trimmed.starts_with('}') {
                return Some((offset, line));
            }
        }

        None
    }

    /// Check standalone function for inline comments.
    ///
    /// # Arguments
    ///
    /// * `func` - Function item to analyze
    /// * `content` - Source code content
    fn check_function(func: &ItemFn, content: &str) -> Vec<Issue> {
        let span = func.block.span();
        let start_line = span.start().line;
        let end_line = span.end().line;

        Self::check_block(start_line, end_line, content)
    }

    /// Check impl block methods for inline comments.
    ///
    /// # Arguments
    ///
    /// * `impl_block` - Impl block to analyze
    /// * `content` - Source code content
    fn check_impl_block(impl_block: &ItemImpl, content: &str) -> Vec<Issue> {
        let mut issues = Vec::new();

        for item in &impl_block.items {
            if let ImplItem::Fn(method) = item {
                let span = method.block.span();
                let start_line = span.start().line;
                let end_line = span.end().line;

                issues.extend(Self::check_block(start_line, end_line, content));
            }
        }

        issues
    }
}

impl Analyzer for InlineCommentsAnalyzer {
    fn name(&self) -> &'static str {
        "inline_comments"
    }

    fn analyze(&self, ast: &File, content: &str) -> AppResult<AnalysisResult> {
        let mut visitor = FunctionVisitor {
            issues:  Vec::new(),
            content: content.to_string()
        };
        visitor.visit_file(ast);

        Ok(AnalysisResult {
            issues:        visitor.issues,
            fixable_count: 0
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
        match node {
            Item::Fn(func) => {
                let func_issues = InlineCommentsAnalyzer::check_function(func, &self.content);
                self.issues.extend(func_issues);
            }
            Item::Impl(impl_block) => {
                let impl_issues =
                    InlineCommentsAnalyzer::check_impl_block(impl_block, &self.content);
                self.issues.extend(impl_issues);
            }
            _ => {}
        }
        syn::visit::visit_item(self, node);
    }
}

impl Default for InlineCommentsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_name() {
        let analyzer = InlineCommentsAnalyzer::new();
        assert_eq!(analyzer.name(), "inline_comments");
    }

    #[test]
    fn test_detect_inline_comment_in_function() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;
    // This is a comment
    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].message.contains("This is a comment"));
    }

    #[test]
    fn test_ignore_doc_comments() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;
    /// This is a doc comment
    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_function_without_comments() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;
    let y = 2;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_multiple_comments() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn process() {
    // Read data
    let x = read();
    // Transform
    let y = transform(x);
    // Write result
    write(y);
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 3);
    }

    #[test]
    fn test_comment_with_code_context() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    // Calculate sum
    let sum = a + b;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].message.contains("Calculate sum"));
        assert!(result.issues[0].message.contains("`let sum = a + b;`"));
    }

    #[test]
    fn test_detect_comment_in_method() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"struct Foo;

impl Foo {
    fn method(&self) {
        // Process data
        let x = 1;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].message.contains("Process data"));
    }

    #[test]
    fn test_multiple_methods_with_comments() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"struct Foo;

impl Foo {
    fn first(&self) {
        // Comment 1
        let a = 1;
    }

    fn second(&self) {
        // Comment 2
        let b = 2;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_fixable_count_is_zero() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    // Comment
    let x = 1;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.fixable_count, 0);
    }

    #[test]
    fn test_fix_returns_zero() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    // Comment
    let x = 1;
}"#;
        let mut code = syn::parse_str(content).unwrap();

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 0);
    }

    #[test]
    fn test_default_implementation() {
        let analyzer = InlineCommentsAnalyzer;
        assert_eq!(analyzer.name(), "inline_comments");
    }

    #[test]
    fn test_comment_before_closing_brace() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    let x = 1;
    // Final comment
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_empty_comment() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    //
    let x = 1;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_comment_with_multiple_slashes() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    //// Comment
    let x = 1;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_nested_blocks_with_comments() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    if true {
        // Nested comment
        let x = 1;
    }
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 1);
    }
}
