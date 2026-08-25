// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Path import analyzer for detecting inline path usage.
//!
//! This analyzer identifies module paths with `::` that should be moved to
//! import statements. It distinguishes between:
//! - Free functions from absolute module paths (should be imported)
//! - Associated functions on types (should NOT be imported)
//! - Enum variants (should NOT be imported)
//! - Associated constants (should NOT be imported)
//!
//! Only absolute paths — rooted at `std`, `core`, `alloc`, or `crate` — are
//! rewritten. Relative paths (including `self::` and `super::`) resolve
//! against the module they appear in, so hoisting them into a `use` statement
//! would change their meaning.
//!
//! Fixes are scope-aware: each rewrite consults a per-module symbol table
//! built from the file (item definitions, `use`-bound names, glob imports,
//! and local bindings), the required `use` statement is inserted into the
//! module containing the rewritten path, and a rewrite is skipped whenever
//! the bare name could collide with or silently rebind an existing name.

use std::{
    collections::{HashMap, HashSet},
    ops::Range
};

use masterror::AppResult;
use syn::{ExprPath, File, Item, ItemUse, Path, UseTree, spanned::Spanned, visit::Visit};

use crate::{
    analyzer::{AnalysisResult, Analyzer, Fix, ImportEdit, Issue, Suggestion, TextEdit},
    fixer::import_insertion_offset
};

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
    /// Only absolute paths rooted at `std`, `core`, `alloc`, or `crate` whose
    /// final segment names a free function are accepted. Type paths, enum
    /// variants, associated items, and relative module paths are rejected.
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

        Self::is_extractable_root(&first_name)
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

    /// Check if name roots an absolute path that is safe to import from.
    ///
    /// Relative roots (module names, `self`, `super`) resolve against the
    /// module the path appears in, so hoisting them into a `use` statement
    /// would change their meaning.
    ///
    /// # Arguments
    ///
    /// * `name` - Root segment name to check
    ///
    /// # Returns
    ///
    /// `true` if name is `std`, `core`, `alloc`, or `crate`
    fn is_extractable_root(name: &str) -> bool {
        matches!(name, "std" | "core" | "alloc" | "crate")
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

    fn suggestions(&self, ast: &File, content: &str) -> AppResult<Vec<Suggestion>> {
        let root = ModuleScope::build(&ast.items, Some(import_insertion_offset(content)));

        let mut suggestions = Vec::new();
        root.collect_suggestions(&HashSet::new(), false, &mut suggestions);

        Ok(suggestions)
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

/// Name an item binds in its enclosing module, if any.
///
/// # Arguments
///
/// * `item` - Item to inspect
///
/// # Returns
///
/// The bound identifier, or `None` for items that bind no single name
fn item_bound_name(item: &Item) -> Option<String> {
    match item {
        Item::Const(item) => Some(item.ident.to_string()),
        Item::Enum(item) => Some(item.ident.to_string()),
        Item::ExternCrate(item) => Some(
            item.rename
                .as_ref()
                .map_or_else(|| item.ident.to_string(), |(_, rename)| rename.to_string())
        ),
        Item::Fn(item) => Some(item.sig.ident.to_string()),
        Item::Macro(item) => item.ident.as_ref().map(|ident| ident.to_string()),
        Item::Mod(item) => Some(item.ident.to_string()),
        Item::Static(item) => Some(item.ident.to_string()),
        Item::Struct(item) => Some(item.ident.to_string()),
        Item::Trait(item) => Some(item.ident.to_string()),
        Item::TraitAlias(item) => Some(item.ident.to_string()),
        Item::Type(item) => Some(item.ident.to_string()),
        Item::Union(item) => Some(item.ident.to_string()),
        _ => None
    }
}

/// A qualified path eligible for rewriting to a bare imported name.
struct Candidate {
    /// Full colon-joined path
    path:  String,
    /// Final segment the rewrite leaves behind
    ident: String,
    /// Byte range of the leading segments to delete
    range: Range<usize>
}

/// Names one module binds and the bare identifiers its code relies on.
///
/// Mirrors how name resolution sees the module: names bound by items, `use`
/// statements, and local bindings, whether glob imports bring in unknown
/// names, and which bare identifiers the module's code already uses.
#[derive(Default)]
struct SymbolTable {
    /// Names bound in this module: items, `use`-bound names, local bindings
    bound:            HashSet<String>,
    /// Bare (single-segment) identifiers used in expressions in this module
    bare_idents:      HashSet<String>,
    /// Whether the module has `use super::*`
    has_super_glob:   bool,
    /// Whether the module has a glob import other than `use super::*`
    has_foreign_glob: bool
}

impl SymbolTable {
    /// Records the names and glob imports a `use` statement introduces.
    ///
    /// # Arguments
    ///
    /// * `item` - The `use` statement to record
    fn record_use(&mut self, item: &ItemUse) {
        let mut prefix = Vec::new();
        self.record_use_tree(&item.tree, &mut prefix);
    }

    /// Walks a use tree, recording bound names and glob imports.
    ///
    /// `use a::b::{self}` binds `b`, renames bind the new name, and globs set
    /// the matching flag: `use super::*` inherits the parent scope, while any
    /// other glob brings in names this analysis cannot enumerate.
    ///
    /// # Arguments
    ///
    /// * `tree` - Use tree node to walk
    /// * `prefix` - Path segments accumulated above this node
    fn record_use_tree(&mut self, tree: &UseTree, prefix: &mut Vec<String>) {
        match tree {
            UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.record_use_tree(&path.tree, prefix);
                prefix.pop();
            }
            UseTree::Name(name) => {
                let ident = name.ident.to_string();
                if ident == "self" {
                    if let Some(parent) = prefix.last() {
                        self.bound.insert(parent.clone());
                    }
                } else {
                    self.bound.insert(ident);
                }
            }
            UseTree::Rename(rename) => {
                self.bound.insert(rename.rename.to_string());
            }
            UseTree::Glob(_) => {
                if prefix.len() == 1 && prefix[0] == "super" {
                    self.has_super_glob = true;
                } else {
                    self.has_foreign_glob = true;
                }
            }
            UseTree::Group(group) => {
                for tree in &group.items {
                    self.record_use_tree(tree, prefix);
                }
            }
        }
    }
}

/// Scope tree node: one module's symbols, rewrite candidates, and children.
struct ModuleScope {
    /// Names bound in and bare identifiers used by this module
    symbols:       SymbolTable,
    /// Byte offset at which to insert `use` statements for this module
    insert_offset: Option<usize>,
    /// Rewrite candidates found directly in this module
    candidates:    Vec<Candidate>,
    /// Inline child modules
    children:      Vec<ModuleScope>
}

impl ModuleScope {
    /// Builds the scope tree for a module's items.
    ///
    /// Inline child modules become child scopes; `use` statements and item
    /// definitions populate the symbol table; function bodies are scanned for
    /// rewrite candidates, bare identifier usage, and local bindings.
    ///
    /// # Arguments
    ///
    /// * `items` - Items of the module
    /// * `insert_offset` - Byte offset for this module's `use` insertions
    ///
    /// # Returns
    ///
    /// The populated scope for this module and its descendants
    fn build(items: &[Item], insert_offset: Option<usize>) -> Self {
        let mut scope = Self {
            symbols: SymbolTable::default(),
            insert_offset,
            candidates: Vec::new(),
            children: Vec::new()
        };

        for item in items {
            if let Item::Mod(module) = item {
                scope.symbols.bound.insert(module.ident.to_string());
                if let Some((_, child_items)) = &module.content {
                    let child_offset = child_items
                        .first()
                        .map(|first| first.span().byte_range().start);
                    scope.children.push(Self::build(child_items, child_offset));
                }
                continue;
            }

            let mut collector = BodyCollector {
                scope: &mut scope
            };
            collector.visit_item(item);
        }

        scope
    }

    /// Final identifiers reachable from more than one distinct path here.
    ///
    /// Rewriting such an identifier would create ambiguous imports inside
    /// this module, so those paths are left qualified.
    ///
    /// # Returns
    ///
    /// Set of ambiguous final identifiers
    fn ambiguous_idents(&self) -> HashSet<String> {
        let mut sources: HashMap<&str, &str> = HashMap::new();
        let mut ambiguous = HashSet::new();

        for candidate in &self.candidates {
            match sources.get(candidate.ident.as_str()) {
                Some(path) if *path != candidate.path.as_str() => {
                    ambiguous.insert(candidate.ident.clone());
                }
                Some(_) => {}
                None => {
                    sources.insert(&candidate.ident, &candidate.path);
                }
            }
        }

        ambiguous
    }

    /// Whether importing a name here would rebind a descendant's bare usage.
    ///
    /// A descendant module reachable through a chain of `use super::*` globs
    /// sees names imported here. If such a descendant already uses the name
    /// bare without binding it itself, adding the import could silently
    /// change what that usage resolves to.
    ///
    /// # Arguments
    ///
    /// * `name` - Candidate import name to check
    ///
    /// # Returns
    ///
    /// `true` if a glob-inheriting descendant uses the name unbound
    fn descendant_bare_conflict(&self, name: &str) -> bool {
        self.children.iter().any(|child| {
            child.symbols.has_super_glob
                && ((child.symbols.bare_idents.contains(name)
                    && !child.symbols.bound.contains(name))
                    || child.descendant_bare_conflict(name))
        })
    }

    /// Emits scope-safe suggestions for this module and its descendants.
    ///
    /// A candidate is rewritten only when its bare name is not already bound
    /// in the effective scope (own names plus names inherited through
    /// `use super::*`), is unambiguous among this module's candidates, and
    /// cannot rebind a descendant's bare usage. Modules whose scope contains
    /// a glob of unknown names produce no rewrites at all.
    ///
    /// # Arguments
    ///
    /// * `inherited_bound` - Names visible from ancestors via `use super::*`
    /// * `inherited_foreign_glob` - Whether ancestors leak unknown glob names
    /// * `suggestions` - Output collection
    fn collect_suggestions(
        &self,
        inherited_bound: &HashSet<String>,
        inherited_foreign_glob: bool,
        suggestions: &mut Vec<Suggestion>
    ) {
        let foreign_glob = self.symbols.has_foreign_glob
            || (self.symbols.has_super_glob && inherited_foreign_glob);

        let mut visible = self.symbols.bound.clone();
        if self.symbols.has_super_glob {
            visible.extend(inherited_bound.iter().cloned());
        }

        if !foreign_glob && let Some(offset) = self.insert_offset {
            let ambiguous = self.ambiguous_idents();

            for candidate in &self.candidates {
                if ambiguous.contains(&candidate.ident)
                    || visible.contains(&candidate.ident)
                    || self.descendant_bare_conflict(&candidate.ident)
                {
                    continue;
                }

                suggestions.push(Suggestion {
                    edit:   TextEdit {
                        range:       candidate.range.clone(),
                        replacement: String::new()
                    },
                    import: Some(ImportEdit {
                        offset,
                        statement: format!("use {};", candidate.path)
                    })
                });
            }
        }

        for child in &self.children {
            child.collect_suggestions(&visible, foreign_glob, suggestions);
        }
    }
}

/// Scans one non-module item of a module for names and rewrite candidates.
///
/// Collects rewrite candidates, bare identifier usage, and local bindings
/// into the owning [`ModuleScope`]. Function-local `use` statements are
/// recorded conservatively as if module-level; function-local modules are
/// left untouched (no candidates, no bindings).
struct BodyCollector<'scope> {
    /// Scope receiving the collected facts
    scope: &'scope mut ModuleScope
}

impl<'scope, 'ast> Visit<'ast> for BodyCollector<'scope> {
    fn visit_item(&mut self, node: &'ast Item) {
        match node {
            Item::Mod(_) => {}
            Item::Use(item) => {
                self.scope.symbols.record_use(item);
            }
            other => {
                if let Some(name) = item_bound_name(other) {
                    self.scope.symbols.bound.insert(name);
                }
                syn::visit::visit_item(self, node);
            }
        }
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if node.qself.is_none() {
            if node.path.segments.len() == 1 {
                if let Some(only) = node.path.segments.first() {
                    self.scope
                        .symbols
                        .bare_idents
                        .insert(only.ident.to_string());
                }
            } else if PathImportAnalyzer::should_extract_to_import(&node.path)
                && let Some(last) = node.path.segments.last()
            {
                let path_start = node.path.span().byte_range().start;
                let last_start = last.ident.span().byte_range().start;

                if last_start > path_start {
                    self.scope.candidates.push(Candidate {
                        path:  path_to_string(&node.path),
                        ident: last.ident.to_string(),
                        range: path_start..last_start
                    });
                }
            }
        }

        syn::visit::visit_expr_path(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.scope.symbols.bound.insert(node.ident.to_string());
        syn::visit::visit_pat_ident(self, node);
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

            let path_str = path_to_string(path);

            let function_name = path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            self.issues.push(Issue::new(
                start.line,
                start.column,
                format!("Use import instead of path: {}", path_str),
                Fix::WithImport {
                    import:      format!("use {};", path_str),
                    pattern:     path_str.clone(),
                    replacement: function_name
                }
            ));
        }
    }
}

impl<'ast> Visit<'ast> for PathVisitor {
    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        self.check_path(&node.path);
        syn::visit::visit_expr_path(self, node);
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
    fn test_ignore_relative_module_paths() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                let data = helpers::io::load("file");
                let more = my_mod::sub::func();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_ignore_super_and_self_paths() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            mod inner {
                fn f() {
                    super::helpers::run();
                    self::local::call();
                }
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_crate_rooted_paths() {
        let analyzer = PathImportAnalyzer::new();
        let code: File = parse_quote! {
            fn main() {
                crate::util::helper();
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 1);
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

    fn apply_fix(content: &str) -> (usize, String) {
        let analyzer = PathImportAnalyzer::new();
        let ast = syn::parse_file(content).unwrap();
        let suggestions = analyzer.suggestions(&ast, content).unwrap();
        let fixed = suggestions.len();
        let output = crate::fixer::apply_suggestions(content, &suggestions);
        (fixed, output)
    }

    #[test]
    fn test_fix_rewrites_path_and_adds_import() {
        let content = "fn main() {\n    let content = std::fs::read_to_string(\"file.txt\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 1);
        assert!(output.contains("use std::fs::read_to_string;"));
        assert!(output.contains("read_to_string(\"file.txt\")"));
        assert!(!output.contains("std::fs::read_to_string("));
    }

    #[test]
    fn test_fix_preserves_comments_and_blank_lines() {
        let content = "// top comment\nfn main() {\n    // inline note\n    let x = std::fs::read_to_string(\"f\");\n\n    let _ = x;\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 1);
        assert!(output.contains("// top comment"), "top comment preserved");
        assert!(
            output.contains("// inline note"),
            "inline comment preserved"
        );
        assert!(
            output.contains("\n\n    let _ = x;"),
            "blank line preserved"
        );
        assert!(output.contains("use std::fs::read_to_string;"));
        assert!(output.contains("let x = read_to_string(\"f\");"));
    }

    #[test]
    fn test_fix_returns_zero_without_issues() {
        let content = "fn main() {\n    let v = Vec::new();\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_dedups_repeated_import() {
        let content = "fn main() {\n    let a = std::fs::read_to_string(\"a\");\n    let b = std::fs::read_to_string(\"b\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 2);
        assert_eq!(output.matches("use std::fs::read_to_string;").count(), 1);
    }

    #[test]
    fn test_fix_skips_short_name_collision() {
        let content = "fn main() {\n    let a = std::fs::read(\"x\");\n    let b = crate::helpers::read(\"y\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert!(output.contains("std::fs::read(\"x\")"));
        assert!(output.contains("crate::helpers::read(\"y\")"));
        assert!(!output.contains("use std::fs::read;"));
        assert!(!output.contains("use crate::helpers::read;"));
    }

    #[test]
    fn test_fix_same_path_repeated_is_not_collision() {
        let content = "fn main() {\n    let a = std::fs::read(\"x\");\n    let b = std::fs::read(\"y\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 2);
        assert_eq!(output.matches("use std::fs::read;").count(), 1);
    }

    #[test]
    fn test_fix_preserves_generic_arguments() {
        let content = "fn main() {\n    let size = core::mem::size_of::<u32>();\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 1);
        assert!(output.contains("use core::mem::size_of;"));
        assert!(output.contains("size_of::<u32>()"));
    }

    #[test]
    fn test_fix_skips_name_bound_by_existing_import() {
        let content =
            "use crate::util::read;\n\nfn main() {\n    let a = std::fs::read(\"x\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_skips_already_imported_path() {
        let content = "use std::fs::read;\n\nfn main() {\n    let a = std::fs::read(\"x\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_skips_name_bound_by_local_fn() {
        let content = "fn read(path: &str) -> &str {\n    path\n}\n\nfn main() {\n    let a = std::fs::read(\"x\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_skips_name_bound_by_local_binding() {
        let content = "fn main() {\n    let read = 1;\n    let a = std::fs::read(\"x\");\n    let _ = read;\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_skips_module_with_foreign_glob() {
        let content = "use helpers::*;\n\nfn main() {\n    let a = std::fs::read(\"x\");\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_inserts_import_into_nested_module() {
        let content = "fn top() {\n    let a = std::fs::read_to_string(\"a\");\n}\n\nmod inner {\n    fn f() {\n        let b = std::fs::read_to_string(\"b\");\n    }\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 2);
        assert_eq!(output.matches("use std::fs::read_to_string;").count(), 2);
        assert!(!output.contains("std::fs::read_to_string("));
        let import_pos = output.find("mod inner").unwrap();
        assert!(
            output[import_pos..].contains("use std::fs::read_to_string;"),
            "nested module receives its own import"
        );
    }

    #[test]
    fn test_fix_respects_names_inherited_via_super_glob() {
        let content = "fn read() {}\n\nmod inner {\n    use super::*;\n\n    fn f() {\n        let x = std::fs::read(\"f\");\n    }\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_skips_when_descendant_uses_name_bare() {
        let content = "fn parent_call() {\n    let a = std::fs::read(\"f\");\n}\n\nmod inner {\n    use super::*;\n\n    fn g() {\n        read(\"x\");\n    }\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 0);
        assert_eq!(output, content);
    }

    #[test]
    fn test_fix_allows_test_module_with_super_glob() {
        let content = "fn top() {\n    let a = std::fs::read_to_string(\"a\");\n}\n\nmod tests {\n    use super::*;\n\n    fn t() {\n        let b = std::fs::read_to_string(\"b\");\n    }\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 2);
        assert!(!output.contains("std::fs::read_to_string("));
    }

    #[test]
    fn test_fix_crate_rooted_path() {
        let content = "fn main() {\n    crate::util::helper();\n}\n";
        let (fixed, output) = apply_fix(content);

        assert_eq!(fixed, 1);
        assert!(output.contains("use crate::util::helper;"));
        assert!(output.contains("    helper();"));
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
                let x = std::u32::MAX_VALUE;
            }
        };

        let result = analyzer.analyze(&code, "").unwrap();
        assert_eq!(result.issues.len(), 0);
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
        assert!(
            issue
                .diagnostic
                .message
                .contains("Use import instead of path")
        );
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
