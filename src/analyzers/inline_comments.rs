// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Inline comments analyzer for detecting non-doc comments in function bodies.
//!
//! This analyzer identifies inline comments (`//`) within function and method
//! bodies, which violate the documentation standards. All explanations should
//! be in doc comments (`///`), specifically in the `# Notes` section.

use std::collections::{BTreeMap, HashSet};

use masterror::AppResult;
use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprLit, File, ImplItem, ImplItemFn, ItemFn, ItemImpl, Lit, Meta,
    spanned::Spanned, visit::Visit
};

use super::{
    line_deletion_range, line_offsets,
    visitor::{FunctionVisitor, ItemCheckers, SourceView}
};
use crate::analyzer::{AnalysisResult, Analyzer, Fix, Issue, Suggestion, TextEdit};

/// Maximum width of generated doc comment lines, matching the format profile.
const DOC_WIDTH: usize = 80;

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

            let trimmed = line.trim();

            if trimmed.starts_with("//") && !trimmed.starts_with("///") {
                let comment_text = trimmed.trim_start_matches("//").trim();

                let code_line = Self::find_related_code_line(lines, idx);

                let suggestion = if let Some((_code_idx, code)) = code_line {
                    format!(
                        "Move to doc block # Notes section:\n/// - {} - `{}`",
                        comment_text,
                        code.trim()
                    )
                } else {
                    format!("Move to doc block # Notes section:\n/// - {}", comment_text)
                };

                issues.push(Issue::new(
                    line_num,
                    1,
                    format!("Inline comment found: \"{}\"\n{}", comment_text, suggestion),
                    Fix::Simple("Move comment to doc block # Notes section".to_string())
                ));
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
    /// * `lines` - Source code split into lines
    fn check_function(func: &ItemFn, lines: &[&str], excluded: &HashSet<usize>) -> Vec<Issue> {
        let span = func.block.span();
        let start_line = span.start().line;
        let end_line = span.end().line;

        Self::check_block(start_line, end_line, lines, excluded)
    }

    /// Check impl block methods for inline comments.
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

/// Location facts about one function needed to relocate its comments.
struct FnSite {
    /// First line of the function body span
    body_start: usize,
    /// Last line of the function body span
    body_end:   usize,
    /// First line of the item, including attributes and doc comments
    item_line:  usize,
    /// Last line of the existing doc comment block, if any
    doc_end:    Option<usize>,
    /// Line of an existing `# Notes` heading inside the doc block, if any
    notes_line: Option<usize>
}

/// Collects [`FnSite`] entries for every function and method in a file.
struct FnSiteCollector {
    /// Sites collected so far
    sites: Vec<FnSite>
}

impl<'ast> Visit<'ast> for FnSiteCollector {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.sites
            .push(fn_site(&node.attrs, node.sig.span(), node.block.span()));
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.sites
            .push(fn_site(&node.attrs, node.sig.span(), node.block.span()));
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Builds the [`FnSite`] for one function from its attributes and spans.
///
/// # Arguments
///
/// * `attrs` - Attributes of the function, including doc comments
/// * `sig_span` - Span of the function signature
/// * `body_span` - Span of the function body block
///
/// # Returns
///
/// Site describing where the function's doc block fix must be placed
fn fn_site(attrs: &[Attribute], sig_span: Span, body_span: Span) -> FnSite {
    let sig_line = sig_span.start().line;
    let mut item_line = sig_line;
    let mut doc_end = None;
    let mut notes_line = None;
    for attr in attrs {
        let span = attr.span();
        item_line = item_line.min(span.start().line);
        if let Some(text) = doc_attr_text(attr) {
            let end = span.end().line;
            doc_end = Some(doc_end.map_or(end, |current: usize| current.max(end)));
            if text.trim() == "# Notes" {
                notes_line = Some(span.start().line);
            }
        }
    }
    FnSite {
        body_start: body_span.start().line,
        body_end: body_span.end().line,
        item_line,
        doc_end,
        notes_line
    }
}

/// Extracts the text of an outer doc comment attribute.
///
/// # Arguments
///
/// * `attr` - Attribute to inspect
///
/// # Returns
///
/// The doc string, or `None` for non-doc attributes
fn doc_attr_text(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("doc") {
        return None;
    }
    if let Meta::NameValue(name_value) = &attr.meta
        && let Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) = &name_value.value
    {
        return Some(value.value());
    }
    None
}

/// Checks whether a source line is an inline `//` comment.
///
/// Doc comments (`///`) and `////` separators are not inline comments.
///
/// # Arguments
///
/// * `line` - Source line to test
///
/// # Returns
///
/// `true` when the line holds a plain `//` comment
fn is_inline_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("//") && !trimmed.starts_with("///")
}

/// Extracts the comment text of an inline comment line.
///
/// # Arguments
///
/// * `line` - Source line holding the comment
///
/// # Returns
///
/// The comment text without the `//` marker and surrounding whitespace
fn comment_text(line: &str) -> &str {
    line.trim().trim_start_matches("//").trim()
}

/// Assigns each inline comment line to its smallest enclosing function.
///
/// Nested functions overlap their parents' body spans, so every comment line
/// is claimed exactly once by the tightest containing site.
///
/// # Arguments
///
/// * `sites` - Function sites collected from the file
/// * `lines` - Source code split into lines
/// * `excluded` - Line numbers inside multi-line literals to skip
///
/// # Returns
///
/// Map from site index to its sorted comment line numbers
fn assign_comment_lines(
    sites: &[FnSite],
    lines: &[&str],
    excluded: &HashSet<usize>
) -> BTreeMap<usize, Vec<usize>> {
    let mut grouped: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for line_num in 1..=lines.len() {
        if excluded.contains(&line_num) {
            continue;
        }
        let Some(line) = lines.get(line_num.saturating_sub(1)) else {
            continue;
        };
        if !is_inline_comment(line) {
            continue;
        }
        let owner = sites
            .iter()
            .enumerate()
            .filter(|(_, site)| {
                site.body_start < site.body_end
                    && site.body_start <= line_num
                    && line_num < site.body_end
            })
            .min_by_key(|(_, site)| site.body_end - site.body_start)
            .map(|(index, _)| index);
        if let Some(index) = owner {
            grouped.entry(index).or_default().push(line_num);
        }
    }
    grouped
}

/// Merges a function's comment lines into `# Notes` paragraphs.
///
/// Consecutive comment lines join into one paragraph; a gap in line numbers or
/// an empty `//` line closes the current paragraph.
///
/// # Arguments
///
/// * `comment_lines` - Sorted comment line numbers of one function
/// * `lines` - Source code split into lines
///
/// # Returns
///
/// Paragraph texts in source order, empty paragraphs dropped
fn comment_paragraphs(comment_lines: &[usize], lines: &[&str]) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut prev: Option<usize> = None;
    for &line_num in comment_lines {
        let text = lines
            .get(line_num.saturating_sub(1))
            .map_or("", |line| comment_text(line));
        let adjacent = prev.is_some_and(|previous| line_num == previous + 1);
        if (!adjacent || text.is_empty()) && !current.is_empty() {
            paragraphs.push(std::mem::take(&mut current));
        }
        if !text.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(text);
        }
        prev = Some(line_num);
    }
    if !current.is_empty() {
        paragraphs.push(current);
    }
    paragraphs
}

/// Returns the leading whitespace of a source line.
///
/// # Arguments
///
/// * `lines` - Source code split into lines
/// * `line_num` - 1-based line number
///
/// # Returns
///
/// The line's indentation string
fn indent_of(lines: &[&str], line_num: usize) -> String {
    lines
        .get(line_num.saturating_sub(1))
        .map_or(String::new(), |line| {
            line[..line.len() - line.trim_start().len()].to_string()
        })
}

/// Finds the last content line of an existing `# Notes` section.
///
/// Scans doc lines after the heading until the doc block ends or the next
/// heading starts.
///
/// # Arguments
///
/// * `lines` - Source code split into lines
/// * `heading` - Line number of the `# Notes` heading
///
/// # Returns
///
/// Line number of the last non-blank doc line in the section, if any
fn notes_section_last_content(lines: &[&str], heading: usize) -> Option<usize> {
    let mut last = None;
    let mut line_num = heading + 1;
    while let Some(line) = lines.get(line_num.saturating_sub(1)) {
        let trimmed = line.trim();
        if !trimmed.starts_with("///") || trimmed.starts_with("/// #") {
            break;
        }
        if trimmed != "///" {
            last = Some(line_num);
        }
        line_num += 1;
    }
    last
}

/// Renders paragraphs as wrapped `/// - ` doc bullets.
///
/// Lines wrap at [`DOC_WIDTH`] columns; continuation lines align under the
/// bullet text.
///
/// # Arguments
///
/// * `indent` - Indentation of the target doc block
/// * `paragraphs` - Paragraph texts to render
///
/// # Returns
///
/// Rendered bullet lines, each terminated by a newline
fn render_bullets(indent: &str, paragraphs: &[String]) -> String {
    let mut output = String::new();
    let continuation = format!("{}///   ", indent);
    for paragraph in paragraphs {
        let mut line = format!("{}/// - ", indent);
        let mut has_words = false;
        for word in paragraph.split_whitespace() {
            if has_words && line.len() + 1 + word.len() > DOC_WIDTH {
                output.push_str(&line);
                output.push('\n');
                line = continuation.clone();
                has_words = false;
            }
            if has_words {
                line.push(' ');
            }
            line.push_str(word);
            has_words = true;
        }
        output.push_str(&line);
        output.push('\n');
    }
    output
}

/// Computes the insertion point and text of a function's `# Notes` fix.
///
/// Appends to an existing `# Notes` section, extends an existing doc block
/// with a new section, or starts a fresh doc block above the item.
///
/// # Arguments
///
/// * `site` - Function site to fix
/// * `lines` - Source code split into lines
/// * `paragraphs` - Paragraph texts to insert
///
/// # Returns
///
/// Insertion line number and the block to insert there
fn insertion_block(site: &FnSite, lines: &[&str], paragraphs: &[String]) -> (usize, String) {
    if let Some(heading) = site.notes_line {
        let indent = indent_of(lines, heading);
        return match notes_section_last_content(lines, heading) {
            Some(last_content) => (last_content + 1, render_bullets(&indent, paragraphs)),
            None => (
                heading + 1,
                format!("{}///\n{}", indent, render_bullets(&indent, paragraphs))
            )
        };
    }
    if let Some(doc_end) = site.doc_end {
        let indent = indent_of(lines, doc_end);
        return (
            doc_end + 1,
            format!(
                "{indent}///\n{indent}/// # Notes\n{indent}///\n{}",
                render_bullets(&indent, paragraphs)
            )
        );
    }
    let indent = indent_of(lines, site.item_line);
    (
        site.item_line,
        format!(
            "{indent}/// # Notes\n{indent}///\n{}",
            render_bullets(&indent, paragraphs)
        )
    )
}

impl Analyzer for InlineCommentsAnalyzer {
    fn name(&self) -> &'static str {
        "inline_comments"
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
        let lines: Vec<&str> = content.lines().collect();
        let excluded = crate::analyzers::multiline_literal_lines(ast);
        let mut collector = FnSiteCollector {
            sites: Vec::new()
        };
        collector.visit_file(ast);
        let offsets = line_offsets(content);
        let grouped = assign_comment_lines(&collector.sites, &lines, &excluded);
        let mut suggestions = Vec::new();
        for (site_index, comment_lines) in &grouped {
            let Some(site) = collector.sites.get(*site_index) else {
                continue;
            };
            for &line in comment_lines {
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
            let paragraphs = comment_paragraphs(comment_lines, &lines);
            if paragraphs.is_empty() {
                continue;
            }
            let (insert_line, block) = insertion_block(site, &lines, &paragraphs);
            let offset = offsets
                .get(insert_line.saturating_sub(1))
                .copied()
                .unwrap_or(content.len());
            suggestions.push(Suggestion {
                edit:   TextEdit {
                    range:       offset..offset,
                    replacement: block
                },
                import: None
            });
        }
        Ok(suggestions)
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
    fn test_ignore_double_slash_inside_string_literal() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content =
            "fn f() {\n    let s = \"first\n// not a comment\nlast\";\n    let _ = s;\n}";
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.issues.len(), 0);
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
        assert!(
            result.issues[0]
                .diagnostic
                .message
                .contains("This is a comment")
        );
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
        assert!(
            result.issues[0]
                .diagnostic
                .message
                .contains("Calculate sum")
        );
        assert!(
            result.issues[0]
                .diagnostic
                .message
                .contains("`let sum = a + b;`")
        );
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
        assert!(result.issues[0].diagnostic.message.contains("Process data"));
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
    fn test_issues_are_fixable() {
        let analyzer = InlineCommentsAnalyzer::new();
        let content = r#"fn main() {
    // Comment
    let x = 1;
}"#;
        let code = syn::parse_str(content).unwrap();

        let result = analyzer.analyze(&code, content).unwrap();
        assert_eq!(result.fixable_count, 1);
        assert!(result.issues[0].fix.is_available());
    }

    fn apply(content: &str) -> String {
        let analyzer = InlineCommentsAnalyzer::new();
        let code = syn::parse_str(content).unwrap();
        let suggestions = analyzer.suggestions(&code, content).unwrap();
        crate::fixer::apply_suggestions(content, &suggestions)
    }

    #[test]
    fn test_fix_moves_comment_to_new_doc_block() {
        let fixed = apply("fn main() {\n    // Comment\n    let x = 1;\n}");
        assert_eq!(
            fixed,
            "/// # Notes\n///\n/// - Comment\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_merges_consecutive_comment_lines() {
        let fixed = apply("fn main() {\n    // first part\n    // second part\n    let x = 1;\n}");
        assert_eq!(
            fixed,
            "/// # Notes\n///\n/// - first part second part\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_splits_paragraphs_on_empty_comment() {
        let fixed = apply("fn main() {\n    // first\n    //\n    // second\n    let x = 1;\n}");
        assert_eq!(
            fixed,
            "/// # Notes\n///\n/// - first\n/// - second\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_separate_runs_become_separate_bullets() {
        let fixed =
            apply("fn main() {\n    // read\n    let x = 1;\n    // write\n    let y = 2;\n}");
        assert_eq!(
            fixed,
            "/// # Notes\n///\n/// - read\n/// - write\nfn main() {\n    let x = 1;\n    let y = 2;\n}"
        );
    }

    #[test]
    fn test_fix_extends_existing_doc_block() {
        let fixed = apply("/// Does things.\nfn main() {\n    // Comment\n    let x = 1;\n}");
        assert_eq!(
            fixed,
            "/// Does things.\n///\n/// # Notes\n///\n/// - Comment\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_appends_to_existing_notes_section() {
        let content = "/// Does things.\n///\n/// # Notes\n///\n/// - existing\nfn main() {\n    // Comment\n    let x = 1;\n}";
        let fixed = apply(content);
        assert_eq!(
            fixed,
            "/// Does things.\n///\n/// # Notes\n///\n/// - existing\n/// - Comment\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_keeps_notes_before_following_heading() {
        let content = "/// Does things.\n///\n/// # Notes\n///\n/// - existing\n///\n/// # Errors\n///\n/// - never\nfn main() {\n    // Comment\n    let x = 1;\n}";
        let fixed = apply(content);
        assert_eq!(
            fixed,
            "/// Does things.\n///\n/// # Notes\n///\n/// - existing\n/// - Comment\n///\n/// # Errors\n///\n/// - never\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_indents_method_doc_block() {
        let content = "struct Foo;\n\nimpl Foo {\n    fn method(&self) {\n        // Process data\n        let x = 1;\n    }\n}";
        let fixed = apply(content);
        assert_eq!(
            fixed,
            "struct Foo;\n\nimpl Foo {\n    /// # Notes\n    ///\n    /// - Process data\n    fn method(&self) {\n        let x = 1;\n    }\n}"
        );
    }

    #[test]
    fn test_fix_inserts_before_attributes() {
        let content = "#[inline]\nfn main() {\n    // Comment\n    let x = 1;\n}";
        let fixed = apply(content);
        assert_eq!(
            fixed,
            "/// # Notes\n///\n/// - Comment\n#[inline]\nfn main() {\n    let x = 1;\n}"
        );
    }

    #[test]
    fn test_fix_targets_nested_function() {
        let content = "fn outer() {\n    fn inner() {\n        // nested\n        let x = 1;\n    }\n    inner();\n}";
        let fixed = apply(content);
        assert_eq!(
            fixed,
            "fn outer() {\n    /// # Notes\n    ///\n    /// - nested\n    fn inner() {\n        let x = 1;\n    }\n    inner();\n}"
        );
    }

    #[test]
    fn test_fix_deletes_empty_comment_without_bullet() {
        let fixed = apply("fn main() {\n    //\n    let x = 1;\n}");
        assert_eq!(fixed, "fn main() {\n    let x = 1;\n}");
    }

    #[test]
    fn test_fix_ignores_quadruple_slash() {
        let content = "fn main() {\n    //// Comment\n    let x = 1;\n}";
        let analyzer = InlineCommentsAnalyzer::new();
        let code = syn::parse_str(content).unwrap();
        let suggestions = analyzer.suggestions(&code, content).unwrap();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_fix_wraps_long_comment() {
        let long = "a".repeat(40);
        let content = format!(
            "fn main() {{\n    // {long} {long} {long}\n    let x = 1;\n}}",
            long = long
        );
        let fixed = apply(&content);
        let expected = format!(
            "/// # Notes\n///\n/// - {long}\n///   {long}\n///   {long}\nfn main() {{\n    let x = 1;\n}}",
            long = long
        );
        assert_eq!(fixed, expected);
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
