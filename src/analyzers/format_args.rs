// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use masterror::AppResult;
use proc_macro2::TokenTree;
use syn::{ExprMacro, File, LitStr, Macro, spanned::Spanned};

use crate::analyzer::{AnalysisResult, Analyzer, Fix, Issue};

/// Analyzer for format macro arguments
pub struct FormatArgsAnalyzer;

impl FormatArgsAnalyzer {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    fn analyze_format_macro(mac: &Macro) -> Option<Issue> {
        let format = Self::extract_format_string(mac)?;
        let placeholder_count = Self::count_positional_placeholders(&format);

        if placeholder_count >= 3 {
            let span = mac.span();
            let start = span.start();

            return Some(Issue {
                line:    start.line,
                column:  start.column,
                message: format!(
                    "Use named format arguments for better readability ({} placeholders)",
                    placeholder_count
                ),
                fix:     Fix::None
            });
        }

        None
    }

    /// Extract the format string literal from macro tokens.
    ///
    /// Returns the unescaped value of the first top-level string literal, which
    /// is the format string for both `print`/`format` and `write`/`writeln`
    /// families (the writer argument is not a string literal).
    ///
    /// # Arguments
    ///
    /// * `mac` - Macro invocation to inspect
    ///
    /// # Returns
    ///
    /// `Some(String)` with the format string value, or `None` if absent
    fn extract_format_string(mac: &Macro) -> Option<String> {
        for token in mac.tokens.clone() {
            if let TokenTree::Literal(literal) = token
                && let Ok(lit_str) = syn::parse_str::<LitStr>(&literal.to_string())
            {
                return Some(lit_str.value());
            }
        }

        None
    }

    /// Count positional placeholders in a format string.
    ///
    /// Counts `{}`, indexed `{0}`, and spec-only `{:?}` placeholders. Named
    /// placeholders such as `{name}` or `{name:spec}` are ignored, and
    /// `{{`/`}}` are treated as escapes.
    ///
    /// # Arguments
    ///
    /// * `format` - Unescaped format string value
    ///
    /// # Returns
    ///
    /// Number of positional placeholders
    fn count_positional_placeholders(format: &str) -> usize {
        let bytes = format.as_bytes();
        let mut count = 0;
        let mut index = 0;

        while index < bytes.len() {
            match bytes[index] {
                b'{' => {
                    if bytes.get(index + 1) == Some(&b'{') {
                        index += 2;
                        continue;
                    }

                    let name_start = index + 1;
                    let mut name_end = name_start;
                    while name_end < bytes.len()
                        && bytes[name_end] != b'}'
                        && bytes[name_end] != b':'
                    {
                        name_end += 1;
                    }

                    let name = &format[name_start..name_end];
                    if name.is_empty() || name.bytes().all(|b| b.is_ascii_digit()) {
                        count += 1;
                    }

                    while index < bytes.len() && bytes[index] != b'}' {
                        index += 1;
                    }
                    index += 1;
                }
                b'}' => {
                    index += if bytes.get(index + 1) == Some(&b'}') {
                        2
                    } else {
                        1
                    };
                }
                _ => index += 1
            }
        }

        count
    }
}

impl Analyzer for FormatArgsAnalyzer {
    fn name(&self) -> &'static str {
        "format_args"
    }

    fn analyze(&self, ast: &File, _content: &str) -> AppResult<AnalysisResult> {
        let mut visitor = FormatVisitor {
            issues: Vec::new()
        };
        syn::visit::visit_file(&mut visitor, ast);

        Ok(AnalysisResult {
            issues:        visitor.issues,
            fixable_count: 0
        })
    }

    fn fix(&self, _ast: &mut File) -> AppResult<usize> {
        Ok(0)
    }
}

struct FormatVisitor {
    issues: Vec<Issue>
}

impl<'ast> syn::visit::Visit<'ast> for FormatVisitor {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        self.check_macro(&node.mac);
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        self.check_macro(&node.mac);
        syn::visit::visit_stmt_macro(self, node);
    }
}

impl FormatVisitor {
    fn check_macro(&mut self, mac: &Macro) {
        let path = &mac.path;

        if (path.is_ident("format")
            || path.is_ident("println")
            || path.is_ident("print")
            || path.is_ident("write")
            || path.is_ident("writeln"))
            && let Some(issue) = FormatArgsAnalyzer::analyze_format_macro(mac)
        {
            self.issues.push(issue);
        }
    }
}

impl Default for FormatArgsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_analyzer_name() {
        let analyzer = FormatArgsAnalyzer::new();
        assert_eq!(analyzer.name(), "format_args");
    }

    #[test]
    fn test_detect_positional_args() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("Values: {} {} {}", 1, 2, 3);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_advisory_only_not_fixable() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("Values: {} {} {}", 1, 2, 3);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
        assert_eq!(result.fixable_count, 0);
        assert!(!result.issues[0].fix.is_available());
    }

    #[test]
    fn test_ignore_simple_positional() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("Value: {}", 42);
                println!("Two: {} {}", 1, 2);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_named_args() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let name = "World";
                println!("Hello {name}");
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_format_macro() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let msg = format!("Values: {} {} {}", 1, 2, 3);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_detect_print_macro() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                print!("Values: {} {} {}", 1, 2, 3);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_detect_write_macro() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                use std::io::Write;
                let mut buf = Vec::new();
                write!(&mut buf, "Values: {} {} {}", 1, 2, 3).unwrap();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_detect_writeln_macro() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                use std::io::Write;
                let mut buf = Vec::new();
                writeln!(&mut buf, "Values: {} {} {}", 1, 2, 3).unwrap();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_fix_returns_zero() {
        let analyzer = FormatArgsAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                println!("Hello {} {} {}", 1, 2, 3);
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 0);
    }

    #[test]
    fn test_default_implementation() {
        let analyzer = FormatArgsAnalyzer;
        assert_eq!(analyzer.name(), "format_args");
    }

    #[test]
    fn test_format_without_args() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("Hello world");
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_count_debug_and_spec_placeholders() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("{} {} {:?}", a, b, c);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].message.contains("3 placeholders"));
    }

    #[test]
    fn test_ignore_named_placeholders_over_threshold() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("{a} {b} {c}", a = 1, b = 2, c = 3);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_escaped_braces_not_counted() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("{{}} {{}} {{}} {}", x);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_indexed_placeholders_counted() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("{0} {1} {2}", a, b, c);
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_count_positional_placeholders_helper() {
        assert_eq!(
            FormatArgsAnalyzer::count_positional_placeholders("{} {} {:?}"),
            3
        );
        assert_eq!(
            FormatArgsAnalyzer::count_positional_placeholders("{a} {b} {c}"),
            0
        );
        assert_eq!(
            FormatArgsAnalyzer::count_positional_placeholders("{{}} literal {}"),
            1
        );
        assert_eq!(
            FormatArgsAnalyzer::count_positional_placeholders("{0} {1:>8} {2:.2}"),
            3
        );
    }
}
