// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Professional Rust code quality analysis with hardcoded standards.
//!
//! `cargo-quality` is a zero-configuration code quality tool that enforces
//! consistent standards across Rust projects. All rules are embedded in the
//! binary, ensuring uniform analysis across your entire codebase.
//!
//! # Overview
//!
//! This library provides:
//!
//! - **[`analyzer`]** - Core trait and types for building analyzers
//! - **[`analyzers`]** - Built-in analyzers for common code quality issues
//! - **[`formatter`]** - Code formatting with hardcoded standards
//! - **[`differ`]** - Diff generation and visualization
//! - **[`report`]** - Analysis report generation
//! - **[`error`]** - Error types for quality operations
//!
//! # Quick Start
//!
//! Analyze code for path imports that should be extracted:
//!
//! ```rust
//! use cargo_quality::{analyzer::Analyzer, analyzers::PathImportAnalyzer};
//!
//! let analyzer = PathImportAnalyzer::new();
//! let code = r#"
//!     fn main() {
//!         let content = std::fs::read_to_string("file.txt");
//!     }
//! "#;
//! let ast = syn::parse_file(code).unwrap();
//! let result = analyzer.analyze(&ast, code).unwrap();
//!
//! assert!(!result.issues.is_empty());
//! assert!(result.issues[0].message.contains("Use import"));
//! ```
//!
//! # Available Analyzers
//!
//! | Analyzer | Description |
//! |----------|-------------|
//! | [`PathImportAnalyzer`] | Detects `std::fs::read` paths that should use `use` |
//! | [`FormatArgsAnalyzer`] | Finds `println!("{}", x)` that should use `{x}` |
//! | [`EmptyLinesAnalyzer`] | Finds empty lines in function bodies |
//! | [`InlineCommentsAnalyzer`] | Finds `//` comments that should be `///` |
//!
//! [`PathImportAnalyzer`]: analyzers::PathImportAnalyzer
//! [`FormatArgsAnalyzer`]: analyzers::FormatArgsAnalyzer
//! [`EmptyLinesAnalyzer`]: analyzers::EmptyLinesAnalyzer
//! [`InlineCommentsAnalyzer`]: analyzers::InlineCommentsAnalyzer
//!
//! # Running All Analyzers
//!
//! Use [`analyzers::get_analyzers()`] to get all built-in analyzers:
//!
//! ```rust
//! use cargo_quality::{analyzer::Analyzer, analyzers::get_analyzers};
//!
//! let code = r#"
//!     fn main() {
//!         let x = std::fs::read("file");
//!     }
//! "#;
//! let ast = syn::parse_file(code).unwrap();
//!
//! for analyzer in get_analyzers() {
//!     let result = analyzer.analyze(&ast, code).unwrap();
//!     println!("[{}] {} issues", analyzer.name(), result.issues.len());
//! }
//! ```
//!
//! # Custom Analyzers
//!
//! Implement the [`analyzer::Analyzer`] trait to create custom analyzers:
//!
//! ```rust
//! use cargo_quality::analyzer::{AnalysisResult, Analyzer};
//! use masterror::AppResult;
//! use syn::File;
//!
//! struct MyAnalyzer;
//!
//! impl Analyzer for MyAnalyzer {
//!     fn name(&self) -> &'static str {
//!         "my_analyzer"
//!     }
//!
//!     fn analyze(&self, _ast: &File, _content: &str) -> AppResult<AnalysisResult> {
//!         Ok(AnalysisResult::default())
//!     }
//!
//!     fn fix(&self, _ast: &mut File) -> AppResult<usize> {
//!         Ok(0)
//!     }
//! }
//! ```
//!
//! # Feature Flags
//!
//! This crate has no optional features. All functionality is enabled by
//! default.
//!
//! # Standards
//!
//! This tool enforces standards from [RustManifest](https://github.com/RAprogramm/RustManifest):
//!
//! - No inline `::` paths in code (use `use` statements)
//! - Named format arguments for readability
//! - No empty lines inside function bodies
//! - Doc comments only (no inline `//` comments)

pub mod analyzer;
pub mod analyzers;
pub mod differ;
pub mod error;
pub mod file_utils;
pub mod formatter;
pub mod mod_rs;
pub mod report;
