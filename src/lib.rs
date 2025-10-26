// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Cargo quality analysis library.
//!
//! This library provides tools for analyzing and improving Rust code quality.
//! It includes various analyzers that detect common issues and suggest fixes.
//!
//! # Examples
//!
//! ```rust
//! use cargo_quality::{analyzer::Analyzer, analyzers::path_import::PathImportAnalyzer};
//!
//! let analyzer = PathImportAnalyzer::new();
//! let code = r#"
//!     fn main() {
//!         let content = std::fs::read_to_string("file.txt");
//!     }
//! "#;
//! let ast = syn::parse_file(code).unwrap();
//! let result = analyzer.analyze(&ast).unwrap();
//! assert!(!result.issues.is_empty());
//! ```

pub mod analyzer;
pub mod analyzers;
pub mod differ;
pub mod error;
pub mod file_utils;
pub mod formatter;
pub mod report;
