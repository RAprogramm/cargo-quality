// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Shared syntax visitor for line-based analyzers.
//!
//! Both the empty-lines and inline-comments analyzers walk the same item
//! kinds (functions and impl blocks) against the same source view. This
//! module holds the single visitor they parameterize with their checkers.

use std::collections::HashSet;

use syn::{Item, ItemFn, ItemImpl, visit::Visit};

use crate::analyzer::Issue;

/// Checker applied to one item kind against the source view.
pub type ItemCheck<T> = fn(&T, &[&str], &HashSet<usize>) -> Vec<Issue>;

/// Checker callbacks for the item kinds the visitor walks.
pub struct ItemCheckers {
    /// Checker invoked for each free function
    pub function:   ItemCheck<ItemFn>,
    /// Checker invoked for each impl block
    pub impl_block: ItemCheck<ItemImpl>
}

/// Source lines and excluded line numbers shared by line-based analyzers.
pub struct SourceView<'src> {
    /// Source split into lines
    pub lines:    &'src [&'src str],
    /// Line numbers the checkers must skip
    pub excluded: &'src HashSet<usize>
}

/// Visits functions and impl blocks, applying the configured checkers.
pub struct FunctionVisitor<'src> {
    /// Issues collected so far
    pub issues:   Vec<Issue>,
    /// Source view handed to every checker
    pub source:   SourceView<'src>,
    /// Checkers to run per item kind
    pub checkers: ItemCheckers
}

impl<'ast, 'src> Visit<'ast> for FunctionVisitor<'src> {
    fn visit_item(&mut self, node: &'ast Item) {
        match node {
            Item::Fn(func) => {
                let found =
                    (self.checkers.function)(func, self.source.lines, self.source.excluded);
                self.issues.extend(found);
            }
            Item::Impl(impl_block) => {
                let found = (self.checkers.impl_block)(
                    impl_block,
                    self.source.lines,
                    self.source.excluded
                );
                self.issues.extend(found);
            }
            _ => {}
        }
        syn::visit::visit_item(self, node);
    }
}
