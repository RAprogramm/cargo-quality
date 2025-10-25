use masterror::AppResult;
use syn::{ExprMacro, File, Macro};

use crate::analyzer::{AnalysisResult, Analyzer, Issue};

/// Analyzer for format macro arguments
pub struct FormatArgsAnalyzer;

impl FormatArgsAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn analyze_format_macro(mac: &Macro) -> Option<Issue> {
        let tokens = &mac.tokens;
        let token_str = tokens.to_string();

        if token_str.contains("{}") {
            let has_comma = token_str.contains(',');
            if has_comma {
                return Some(Issue {
                    line: 0,
                    column: 0,
                    message: "Use named format arguments".to_string(),
                    suggestion: Some("Replace {} with {name}".to_string())
                });
            }
        }

        None
    }
}

impl Analyzer for FormatArgsAnalyzer {
    fn name(&self) -> &'static str {
        "format_args"
    }

    fn analyze(&self, ast: &File) -> AppResult<AnalysisResult> {
        let mut visitor = FormatVisitor { issues: Vec::new() };
        syn::visit::visit_file(&mut visitor, ast);

        let fixable_count = visitor.issues.len();

        Ok(AnalysisResult { issues: visitor.issues, fixable_count })
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

        if path.is_ident("format")
            || path.is_ident("println")
            || path.is_ident("print")
            || path.is_ident("write")
            || path.is_ident("writeln")
        {
            if let Some(issue) = FormatArgsAnalyzer::analyze_format_macro(mac) {
                self.issues.push(issue);
            }
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
                let name = "World";
                println!("Hello {}", name);
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert!(result.issues.len() > 0);
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

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_format_macro() {
        let analyzer = FormatArgsAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let msg = format!("Value: {}", 42);
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert!(result.issues.len() > 0);
    }
}
