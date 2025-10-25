//! Path import analyzer for detecting inline path usage.
//!
//! This analyzer identifies module paths with `::` that should be moved to
//! import statements. It distinguishes between:
//! - Free functions from modules (should be imported)
//! - Associated functions on types (should NOT be imported)
//! - Enum variants (should NOT be imported)
//! - Associated constants (should NOT be imported)

use masterror::AppResult;
use quote::ToTokens;
use syn::{
    ExprMethodCall, ExprPath, File, Path,
    visit::Visit,
    visit_mut::{self, VisitMut}
};

use crate::analyzer::{AnalysisResult, Analyzer, Issue};

/// Analyzer for detecting path separators that should be imports.
///
/// Detects module-level function calls using `::` syntax that should be
/// converted to proper import statements for cleaner, more idiomatic code.
///
/// # Examples
///
/// Detects this pattern:
/// ```ignore
/// let content = std::fs::read_to_string("file.txt");
/// ```
///
/// Suggests:
/// ```ignore
/// use std::fs::read_to_string;
/// let content = read_to_string("file.txt");
/// ```
pub struct PathImportAnalyzer;

impl PathImportAnalyzer {
    /// Create new path import analyzer instance.
    pub fn new() -> Self {
        Self
    }

    /// Determine if path should be extracted to import statement.
    ///
    /// Analyzes path segments to distinguish module paths from type paths.
    ///
    /// # Arguments
    ///
    /// * `path` - Syntax path to analyze
    ///
    /// # Returns
    ///
    /// `true` if path represents free function that should be imported
    fn should_extract_to_import(path: &Path) -> bool {
        if path.segments.len() < 2 {
            return false;
        }

        let first_segment = match path.segments.first() {
            Some(seg) => seg,
            None => return false
        };

        let first_name = first_segment.ident.to_string();

        let first_char = match first_name.chars().next() {
            Some(c) => c,
            None => return false
        };

        if first_char.is_uppercase() {
            return false;
        }

        let last_segment = match path.segments.last() {
            Some(seg) => seg,
            None => return false
        };

        let last_name = last_segment.ident.to_string();

        if Self::is_screaming_snake_case(&last_name) {
            return false;
        }

        let last_first_char = match last_name.chars().next() {
            Some(c) => c,
            None => return false
        };

        if last_first_char.is_uppercase() {
            return false;
        }

        if Self::is_stdlib_root(&first_name) {
            return true;
        }

        if path.segments.len() >= 3 && first_char.is_lowercase() {
            return true;
        }

        false
    }

    /// Check if identifier is SCREAMING_SNAKE_CASE constant.
    ///
    /// # Arguments
    ///
    /// * `s` - Identifier string to check
    ///
    /// # Returns
    ///
    /// `true` if all characters are uppercase, underscore, or numeric
    fn is_screaming_snake_case(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
    }

    /// Check if name is standard library root module.
    ///
    /// # Arguments
    ///
    /// * `name` - Module name to check
    ///
    /// # Returns
    ///
    /// `true` if name is `std`, `core`, or `alloc`
    fn is_stdlib_root(name: &str) -> bool {
        matches!(name, "std" | "core" | "alloc")
    }
}

impl Analyzer for PathImportAnalyzer {
    fn name(&self) -> &'static str {
        "path_import"
    }

    fn analyze(&self, ast: &File) -> AppResult<AnalysisResult> {
        let mut visitor = PathVisitor {
            issues: Vec::new()
        };
        visitor.visit_file(&mut ast.clone());

        let fixable_count = visitor.issues.len();

        Ok(AnalysisResult {
            issues: visitor.issues,
            fixable_count
        })
    }

    fn fix(&self, ast: &mut File) -> AppResult<usize> {
        let mut fixer = PathFixer {
            fixed_count: 0
        };
        fixer.visit_file_mut(ast);
        Ok(fixer.fixed_count)
    }
}

struct PathVisitor {
    issues: Vec<Issue>
}

impl PathVisitor {
    fn check_path(&mut self, path: &Path) {
        if PathImportAnalyzer::should_extract_to_import(path) {
            self.issues.push(Issue {
                line:       0,
                column:     0,
                message:    format!("Use import instead of path: {}", path.to_token_stream()),
                suggestion: Some(format!("use {};", path.to_token_stream()))
            });
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for PathVisitor {
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        self.check_path(&node.path);
        syn::visit::visit_expr_path(self, node);
    }
}

struct PathFixer {
    fixed_count: usize
}

impl VisitMut for PathFixer {
    fn visit_expr_method_call_mut(&mut self, node: &mut ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, node);
    }
}

impl Default for PathImportAnalyzer {
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
        let analyzer = PathImportAnalyzer::new();
        assert_eq!(analyzer.name(), "path_import");
    }

    #[test]
    fn test_detect_path_separator() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let content = std::fs::read_to_string("file.txt");
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert!(result.issues.len() > 0);
    }

    #[test]
    fn test_ignore_enum_variants() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let err = AppError::NotFound;
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_stdlib_free_functions() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let content = std::fs::read_to_string("file.txt");
                let result = std::io::stdin();
                let data = core::mem::size_of::<u32>();
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 3);
    }

    #[test]
    fn test_ignore_associated_functions() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let v = Vec::new();
                let s = String::from("hello");
                let p = PathBuf::from("/path");
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_option_result_variants() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let x = Option::Some(42);
                let y = Option::None;
                let ok = Result::Ok(1);
                let err = Result::Err("error");
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_associated_constants() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let max = u32::MAX;
                let min = i64::MIN;
                let pi = f64::consts::PI;
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_module_paths_3plus_segments() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let handle = tokio::runtime::Runtime::new();
                let client = reqwest::blocking::Client::new();
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert!(result.issues.len() >= 2);
    }

    #[test]
    fn test_mixed_scenarios() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let content = std::fs::read_to_string("file.txt");
                let v = Vec::new();
                let opt = Option::Some(42);
                let max = u32::MAX;
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_fix_returns_zero() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let content = std::fs::read_to_string("file.txt");
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 0);
    }

    #[test]
    fn test_default_implementation() {
        let analyzer = PathImportAnalyzer::default();
        assert_eq!(analyzer.name(), "path_import");
    }

    #[test]
    fn test_single_segment_path() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                println!("test");
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_core_module_functions() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let size = core::mem::size_of::<u32>();
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert!(result.issues.len() > 0);
    }

    #[test]
    fn test_alloc_module_functions() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                use alloc::vec;
                let v = alloc::vec::Vec::new();
            }
        };

        let result = analyzer.analyze(&code).unwrap();
        assert!(result.issues.len() > 0);
    }
}
