// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Built-in code quality analyzers.
//!
//! This module contains all analyzers that ship with cargo-quality.
//! Each analyzer detects a specific code quality issue and optionally
//! provides automatic fixes.
//!
//! # Available Analyzers
//!
//! | Analyzer | Issue Detected | Auto-fix |
//! |----------|---------------|----------|
//! | [`PathImportAnalyzer`] | `std::fs::read()` paths | Yes |
//! | [`FormatArgsAnalyzer`] | `println!("{}", x)` positional args | No |
//! | [`EmptyLinesAnalyzer`] | Empty lines in functions | Yes |
//! | [`InlineCommentsAnalyzer`] | `//` comments in code | No |
//!
//! # Usage
//!
//! Get all analyzers:
//!
//! ```rust
//! use cargo_quality::analyzers::get_analyzers;
//!
//! let analyzers = get_analyzers();
//! assert_eq!(analyzers.len(), 4);
//! ```
//!
//! Use a specific analyzer:
//!
//! ```rust
//! use cargo_quality::{analyzer::Analyzer, analyzers::PathImportAnalyzer};
//!
//! let analyzer = PathImportAnalyzer::new();
//! let code = "fn main() { std::fs::read(\"f\"); }";
//! let ast = syn::parse_file(code).unwrap();
//!
//! let result = analyzer.analyze(&ast, code).unwrap();
//! println!("Found {} issues", result.issues.len());
//! ```
//!
//! # Analyzer Details
//!
//! ## Path Import Analyzer
//!
//! Detects module paths with `::` that should be moved to `use` statements.
//! Only flags free functions from modules, not associated functions or enum
//! variants.
//!
//! ```rust
//! # use cargo_quality::{analyzer::Analyzer, analyzers::PathImportAnalyzer};
//! let analyzer = PathImportAnalyzer::new();
//! let code = r#"
//!     fn main() {
//!         // Flagged: should use `use std::fs::read_to_string;`
//!         let _ = std::fs::read_to_string("file.txt");
//!
//!         // NOT flagged: associated function on type
//!         let _ = Vec::new();
//!     }
//! "#;
//! let ast = syn::parse_file(code).unwrap();
//! let result = analyzer.analyze(&ast, code).unwrap();
//! assert_eq!(result.issues.len(), 1);
//! ```
//!
//! ## Format Args Analyzer
//!
//! Detects positional format arguments that should use named arguments
//! for better readability (3+ placeholders triggers a warning).
//!
//! ```rust
//! # use cargo_quality::{analyzer::Analyzer, analyzers::FormatArgsAnalyzer};
//! let analyzer = FormatArgsAnalyzer::new();
//! let code = r#"
//!     fn main() {
//!         // Flagged: 3+ positional args
//!         println!("{} {} {}", a, b, c);
//!
//!         // NOT flagged: only 2 args
//!         println!("{} {}", a, b);
//!     }
//! "#;
//! let ast = syn::parse_file(code).unwrap();
//! let result = analyzer.analyze(&ast, code).unwrap();
//! assert_eq!(result.issues.len(), 1);
//! ```
//!
//! ## Empty Lines Analyzer
//!
//! Detects empty lines inside function bodies that indicate the function
//! is doing multiple things and should be refactored.
//!
//! ```rust
//! # use cargo_quality::{analyzer::Analyzer, analyzers::EmptyLinesAnalyzer};
//! let analyzer = EmptyLinesAnalyzer::new();
//! let code = "fn main() {\n    let x = 1;\n\n    let y = 2;\n}";
//! let ast = syn::parse_file(code).unwrap();
//! let result = analyzer.analyze(&ast, code).unwrap();
//! assert_eq!(result.issues.len(), 1);
//! ```
//!
//! ## Inline Comments Analyzer
//!
//! Detects `//` comments inside function bodies. These should be moved
//! to doc comments (`///`) in the `# Notes` section.
//!
//! ```rust
//! # use cargo_quality::{analyzer::Analyzer, analyzers::InlineCommentsAnalyzer};
//! let analyzer = InlineCommentsAnalyzer::new();
//! let code = "fn main() {\n    // This is flagged\n    let x = 1;\n}";
//! let ast = syn::parse_file(code).unwrap();
//! let result = analyzer.analyze(&ast, code).unwrap();
//! assert_eq!(result.issues.len(), 1);
//! ```

pub mod empty_lines;
pub mod format_args;
pub mod inline_comments;
pub mod path_import;

pub use empty_lines::EmptyLinesAnalyzer;
pub use format_args::FormatArgsAnalyzer;
pub use inline_comments::InlineCommentsAnalyzer;
pub use path_import::PathImportAnalyzer;

use crate::analyzer::Analyzer;

/// Returns all built-in analyzers.
///
/// This function creates new instances of all available analyzers.
/// Use this to run comprehensive code quality checks.
///
/// # Returns
///
/// Vector of boxed analyzer trait objects, in order:
/// 1. [`PathImportAnalyzer`] - path separator detection
/// 2. [`FormatArgsAnalyzer`] - format argument detection
/// 3. [`EmptyLinesAnalyzer`] - empty line detection
/// 4. [`InlineCommentsAnalyzer`] - inline comment detection
///
/// # Examples
///
/// ```rust
/// use cargo_quality::{analyzer::Analyzer, analyzers::get_analyzers};
///
/// let analyzers = get_analyzers();
/// assert_eq!(analyzers.len(), 4);
///
/// for analyzer in &analyzers {
///     println!("Analyzer: {}", analyzer.name());
/// }
/// ```
pub fn get_analyzers() -> Vec<Box<dyn Analyzer>> {
    vec![
        Box::new(PathImportAnalyzer::new()),
        Box::new(FormatArgsAnalyzer::new()),
        Box::new(EmptyLinesAnalyzer::new()),
        Box::new(InlineCommentsAnalyzer::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_analyzers() {
        let analyzers = get_analyzers();
        assert_eq!(analyzers.len(), 4);
    }

    #[test]
    fn test_analyzer_names() {
        let analyzers = get_analyzers();
        let names: Vec<&str> = analyzers.iter().map(|a| a.name()).collect();

        assert!(names.contains(&"path_import"));
        assert!(names.contains(&"format_args"));
        assert!(names.contains(&"empty_lines"));
        assert!(names.contains(&"inline_comments"));
    }
}
