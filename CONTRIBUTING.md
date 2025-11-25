<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
SPDX-License-Identifier: MIT
-->

# Contributing to cargo-quality

Thank you for your interest in contributing to this project!

## PR Size Limits

This project enforces PR size limits using [rust-prod-diff-checker](https://github.com/RAprogramm/rust-prod-diff-checker).

**Maximum 200 lines of production code per PR.**

- Only production code counts (src/*.rs)
- Tests, benchmarks, examples are excluded
- Documentation changes don't count

Large PRs are harder to review and more likely to contain bugs. If your change exceeds 200 lines, split it into smaller PRs.

## Code Style & Standards

This project follows the [RustManifest](https://github.com/RAprogramm/RustManifest) coding standards. Please read it thoroughly before contributing.

Key points:
- Use `cargo +nightly fmt` for formatting
- No `unwrap()` or `expect()` in production code
- Documentation via Rustdoc only (no inline comments)
- Descriptive naming conventions

## Development Setup

### Prerequisites

- Rust nightly toolchain
- cargo-nextest (for running tests)

### Installation

```bash
git clone https://github.com/RAprogramm/cargo-quality
cd cargo-quality

# Install nightly toolchain
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly
rustup component add clippy

# Install test runner (optional but recommended)
cargo install cargo-nextest
```

### Pre-commit Checks

Before committing, ensure all checks pass:

```bash
# Format check
cargo +nightly fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Or with nextest
cargo nextest run --all-features
```

## Git Workflow

### Branch Naming

Use issue number as branch name:
```
123
```

### Commit Messages

Format: `#<issue_number> <type>: <description>`

```
#123 feat: add new analyzer
#123 fix: correct line counting in parser
#45 docs: update API examples
#78 test: add property tests for extractor
#90 refactor: simplify config loading
```

Types:
- `feat` - new feature
- `fix` - bug fix
- `docs` - documentation
- `test` - tests
- `refactor` - code refactoring
- `chore` - maintenance tasks

### Pull Requests

1. Create branch from `main`
2. Make your changes
3. Ensure all CI checks pass
4. Keep PR under 200 lines of production code
5. Create PR with descriptive title
6. Include `Closes #<issue>` in description

## Testing

### Test Organization

```
src/
├── analyzers/
│   ├── path_import.rs     # Path import analyzer + tests
│   ├── format_args.rs     # Format args analyzer + tests
│   ├── empty_lines.rs     # Empty lines analyzer + tests
│   └── inline_comments.rs # Inline comments analyzer + tests
tests/
└── integration.rs         # Integration tests
benches/
└── analyzers.rs           # Performance benchmarks
```

### Writing Tests

- Cover all public API functions
- Test error paths, not just happy paths
- No `unwrap()` in tests - use `?` with proper error types

```rust
#[test]
fn test_detect_path_import() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        fn main() {
            std::fs::read_to_string("file.txt");
        }
    "#;
    let issues = analyze(code)?;

    assert_eq!(issues.len(), 1);
    Ok(())
}
```

### Running Tests

```bash
# All tests
cargo test --all-features

# With coverage
cargo llvm-cov nextest --all-features

# Specific test
cargo test test_detect_path_import

# Benchmarks
cargo bench
```

## CI/CD Pipeline

### Automated Checks

Every PR triggers:

| Job | Description |
|-----|-------------|
| PR Size | Max 200 lines production code |
| Format | `cargo +nightly fmt --check` |
| Clippy | `cargo clippy -D warnings` |
| Test | `cargo test --all-features` |
| Doc | `cargo doc --no-deps` |
| Coverage | Upload to Codecov |
| Benchmark | Compile benchmarks |
| Audit | Security vulnerability scan |
| REUSE | License compliance |

### Coverage Requirements

- Project target: 95%+
- New code must be well-tested

## Architecture

### Module Structure

```
src/
├── lib.rs              # Public API exports
├── main.rs             # CLI entry point
├── cli.rs              # Command-line interface
├── error.rs            # Error types (AppError)
├── analyzers/
│   ├── mod.rs          # Analyzer trait and registry
│   ├── path_import.rs  # Path import detection
│   ├── format_args.rs  # Format args detection
│   ├── empty_lines.rs  # Empty lines detection
│   └── inline_comments.rs # Inline comments detection
├── formatter/
│   └── rustfmt.rs      # Formatting with hardcoded standards
├── differ/
│   └── display.rs      # Diff visualization
└── report/
    └── mod.rs          # Report generation
```

### Key Types

- `Analyzer` - Trait for implementing code analyzers
- `Issue` - Represents a detected code quality issue
- `Fix` - Represents an automatic fix for an issue
- `Report` - Collection of issues from analysis

## Adding New Analyzers

1. Create `src/analyzers/new_analyzer.rs`
2. Implement `Analyzer` trait
3. Register in `src/analyzers/mod.rs`
4. Add CLI flag in `src/cli.rs`
5. Add tests and documentation
6. Update README analyzer table

### Analyzer Trait

```rust
pub trait Analyzer {
    fn name(&self) -> &'static str;
    fn analyze(&self, file: &syn::File) -> Vec<Issue>;
}
```

## Release Process

Releases are automated via CI on tag push:

1. Update version in `Cargo.toml`
2. Commit: `chore(release): prepare v1.x.x`
3. Create and push tag:
   ```bash
   git tag v1.x.x
   git push origin v1.x.x
   ```
4. CI builds binaries for all platforms
5. GitHub Release is created automatically
6. Published to crates.io
7. Changelog is updated

### Versioning

Follow [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking API changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes

## Documentation

### Code Documentation

All public items must have Rustdoc:

```rust
/// Analyzes Rust source code for path imports that should use `use` statements.
///
/// # Errors
///
/// Returns `AppError::Parse` if the source code is invalid.
///
/// # Examples
///
/// ```
/// use cargo_quality::analyzers::PathImportAnalyzer;
///
/// let analyzer = PathImportAnalyzer::new();
/// let issues = analyzer.analyze(&file)?;
/// ```
pub fn analyze(&self, file: &syn::File) -> Result<Vec<Issue>, AppError> {
    // ...
}
```

### README Updates

Update README.md when:
- Adding new CLI options
- Adding new analyzers
- Modifying GitHub Action inputs/outputs

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Provide minimal reproduction for bugs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
