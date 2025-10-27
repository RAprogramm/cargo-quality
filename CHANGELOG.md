<!-- SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com> -->
<!-- SPDX-License-Identifier: MIT -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `diff` command for visualizing proposed changes before applying fixes
  - Three display modes: summary, full, and interactive
  - Color-coded output showing old vs new code
  - Import statements shown separately from code replacements
- Criterion benchmarks for performance tracking
  - `format_args_simple`: 160ns baseline
  - `path_import_simple`: 857ns baseline
  - Regression >10% triggers CI failure
- Library crate (`src/lib.rs`) for reusability and benchmarking
- Fix enum architecture for type-safe fix representation
  - `Fix::None` - No automatic fix available
  - `Fix::Simple` - Simple line replacement
  - `Fix::WithImport` - Import addition with pattern matching
- Pattern matching for preserving function arguments in replacements
- Comprehensive doctests for all public API

### Changed
- **BREAKING**: `Issue.suggestion: Option<String>` replaced with `Issue.fix: Fix` enum
- Improved diff display: imports shown on separate lines from replacements
- Enhanced path_import analyzer to correctly preserve function call arguments
- Optimized performance with Vec::with_capacity in hot paths
- Test coverage increased to 86.52% (105 tests, up from 68)

### Fixed
- Critical bug: path_import now preserves function arguments in replacements
- Fixed format_args false positives (133 → 5 issues)
- Changed threshold to 3+ placeholders per RFC 2795
- CI workflow improvements with sequential jobs and caching

## [0.1.0] - Initial Release

### Added
- Path import analyzer for detecting inline module paths
- Format args analyzer for detecting positional format arguments
- Check command for code quality analysis
- Fix command with dry-run support
- Format command using cargo +nightly fmt
- Shell completions support (fish, bash, zsh)
- Hardcoded quality standards (no configuration files)
- Professional error handling with masterror
- Comprehensive test suite (68 tests)
- MIT license with SPDX headers
- CI/CD with GitHub Actions

[Unreleased]: https://github.com/RAprogramm/cargo-quality/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/RAprogramm/cargo-quality/releases/tag/v0.1.0
