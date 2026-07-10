// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Path import analyzer for detecting inline path usage.
//!
//! This analyzer identifies module paths with `::` that should be moved to
//! import statements. It distinguishes between:
//! - Free functions from modules (should be imported)
//! - Associated functions on types (should NOT be imported)
//! - Enum variants (should NOT be imported)
//! - Associated constants (should NOT be imported)

use std::collections::{HashMap, HashSet};

use masterror::AppResult;
use syn::{
    ExprPath, File, Item, ItemUse, Path,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::Visit,
    visit_mut::{self, VisitMut}
};

use crate::analyzer::{AnalysisResult, Analyzer, Fix, Issue};

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
    #[inline]
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

        if path.segments.len() >= 2 {
            let second_to_last = path.segments.iter().rev().nth(1);
            if let Some(seg) = second_to_last {
                let seg_name = seg.ident.to_string();
                if let Some(c) = seg_name.chars().next()
                    && c.is_uppercase()
                {
                    return false;
                }
            }
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

    fn analyze(&self, ast: &File, _content: &str) -> AppResult<AnalysisResult> {
        let mut visitor = PathVisitor {
            issues: Vec::new()
        };
        visitor.visit_file(ast);

        let fixable_count = visitor.issues.len();

        Ok(AnalysisResult {
            issues: visitor.issues,
            fixable_count
        })
    }

    fn fix(&self, ast: &mut File) -> AppResult<usize> {
        let mut collector = PathCollector {
            paths: HashMap::new()
        };
        collector.visit_file(ast);

        let blocked: HashSet<String> = collector
            .paths
            .into_iter()
            .filter(|(_, sources)| sources.len() > 1)
            .map(|(ident, _)| ident)
            .collect();

        let mut fixer = PathFixer {
            fixed_count: 0,
            imports: Vec::new(),
            seen: HashSet::new(),
            blocked
        };
        fixer.visit_file_mut(ast);

        if !fixer.imports.is_empty() {
            let mut items = std::mem::take(&mut fixer.imports);
            items.append(&mut ast.items);
            ast.items = items;
        }

        Ok(fixer.fixed_count)
    }
}

/// Full colon-joined path string of an expression path.
///
/// # Arguments
///
/// * `path` - Path to render
///
/// # Returns
///
/// Segments joined with `::`
fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Collects the final identifier of each extractable path and its source paths.
///
/// Used to detect short-name collisions: an identifier reachable from more than
/// one distinct full path cannot be safely rewritten to an import.
struct PathCollector {
    paths: HashMap<String, HashSet<String>>
}

impl<'ast> Visit<'ast> for PathCollector {
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if node.qself.is_none()
            && PathImportAnalyzer::should_extract_to_import(&node.path)
            && let Some(last) = node.path.segments.last()
        {
            let ident = last.ident.to_string();
            self.paths
                .entry(ident)
                .or_default()
                .insert(path_to_string(&node.path));
        }

        syn::visit::visit_expr_path(self, node);
    }
}

struct PathVisitor {
    issues: Vec<Issue>
}

impl PathVisitor {
    fn check_path(&mut self, path: &Path) {
        if PathImportAnalyzer::should_extract_to_import(path) {
            let span = path.span();
            let start = span.start();

            let path_str = path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let function_name = path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            self.issues.push(Issue {
                line:    start.line,
                column:  start.column,
                message: format!("Use import instead of path: {}", path_str),
                fix:     Fix::WithImport {
                    import:      format!("use {};", path_str),
                    pattern:     path_str.clone(),
                    replacement: function_name
                }
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

/// Rewrites qualified paths to short names and collects the imports to add.
///
/// For each expression path that
/// [`PathImportAnalyzer::should_extract_to_import`] approves, the leading
/// segments are dropped so only the final segment (with its generic arguments)
/// remains, and a matching `use` item is queued for insertion at the top of the
/// file.
struct PathFixer {
    fixed_count: usize,
    imports:     Vec<Item>,
    seen:        HashSet<String>,
    blocked:     HashSet<String>
}

impl VisitMut for PathFixer {
    fn visit_expr_path_mut(&mut self, node: &mut ExprPath) {
        if node.qself.is_none() && PathImportAnalyzer::should_extract_to_import(&node.path) {
            let path_str = path_to_string(&node.path);

            let is_blocked = node
                .path
                .segments
                .last()
                .is_some_and(|last| self.blocked.contains(&last.ident.to_string()));

            if let Some(last) = node.path.segments.last().cloned()
                && !is_blocked
            {
                if self.seen.insert(path_str.clone())
                    && let Ok(item_use) = syn::parse_str::<ItemUse>(&format!("use {};", path_str))
                {
                    self.imports.push(Item::Use(item_use));
                }

                let mut segments = Punctuated::new();
                segments.push(last);
                node.path.segments = segments;
                node.path.leading_colon = None;

                self.fixed_count += 1;
            }
        }

        visit_mut::visit_expr_path_mut(self, node);
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

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_ignore_enum_variants() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let err = AppError::NotFound;
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
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

        let result = analyzer.analyze(&code, "").unwrap();
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
                let m = std::collections::HashMap::new();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
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

        let result = analyzer.analyze(&code, "").unwrap();
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

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_module_paths_3plus_segments() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let content = std::fs::read("file");
                let data = std::io::stdin();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 2);
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

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_fix_rewrites_path_and_adds_import() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let content = std::fs::read_to_string("file.txt");
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 1);

        let output = prettyplease::unparse(&code);
        assert!(output.contains("use std::fs::read_to_string;"));
        assert!(output.contains("read_to_string(\"file.txt\")"));
        assert!(!output.contains("std::fs::read_to_string("));
    }

    #[test]
    fn test_fix_returns_zero_without_issues() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let v = Vec::new();
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 0);
    }

    #[test]
    fn test_fix_dedups_repeated_import() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let a = std::fs::read_to_string("a");
                let b = std::fs::read_to_string("b");
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 2);

        let output = prettyplease::unparse(&code);
        assert_eq!(output.matches("use std::fs::read_to_string;").count(), 1);
    }

    #[test]
    fn test_fix_skips_short_name_collision() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let a = std::fs::read("x");
                let b = other::helpers::read("y");
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 0);

        let output = prettyplease::unparse(&code);
        assert!(output.contains("std::fs::read(\"x\")"));
        assert!(output.contains("other::helpers::read(\"y\")"));
        assert!(!output.contains("use std::fs::read;"));
        assert!(!output.contains("use other::helpers::read;"));
    }

    #[test]
    fn test_fix_same_path_repeated_is_not_collision() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let a = std::fs::read("x");
                let b = std::fs::read("y");
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 2);

        let output = prettyplease::unparse(&code);
        assert_eq!(output.matches("use std::fs::read;").count(), 1);
    }

    #[test]
    fn test_fix_preserves_generic_arguments() {
        let analyzer = PathImportAnalyzer::new();
        let mut code: File = parse_quote! {
            fn main() {
                let size = core::mem::size_of::<u32>();
            }
        };

        let fixed = analyzer.fix(&mut code).unwrap();
        assert_eq!(fixed, 1);

        let output = prettyplease::unparse(&code);
        assert!(output.contains("use core::mem::size_of;"));
        assert!(output.contains("size_of::<u32>()"));
    }

    #[test]
    fn test_default_implementation() {
        let analyzer = PathImportAnalyzer;
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

        let result = analyzer.analyze(&code, "").unwrap();
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

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_alloc_module_functions() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let data = alloc::format::format(format_args!("test"));
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_two_segment_path() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let x = fs::read("file");
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_screaming_snake_case_constant() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let x = some::module::MAX_VALUE;
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_path_with_generics() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let content = std::fs::read_to_string("file.txt");
                let data = std::io::stdin();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_result_fixable_count() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let a = std::fs::read_to_string("f");
                let b = std::io::stdin();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.fixable_count, result.issues.len());
    }

    #[test]
    fn test_issue_format() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let x = std::fs::read("file");
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert!(!result.issues.is_empty());
        let issue = &result.issues[0];
        assert!(issue.message.contains("Use import instead of path"));
        assert!(issue.fix.is_available());
        if let Some((import, pattern, replacement)) = issue.fix.as_import() {
            assert!(import.contains("use"));
            assert_eq!(pattern, "std::fs::read");
            assert_eq!(replacement, "read");
        } else {
            panic!("Expected Fix::WithImport");
        }
    }
}
