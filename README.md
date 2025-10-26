# cargo-quality

[![CI](https://github.com/RAprogramm/cargo-quality/actions/workflows/rust.yml/badge.svg)](https://github.com/RAprogramm/cargo-quality/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/cargo-quality.svg)](https://crates.io/crates/cargo-quality)
[![Documentation](https://docs.rs/cargo-quality/badge.svg)](https://docs.rs/cargo-quality)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![REUSE compliant](https://api.reuse.software/badge/github.com/RAprogramm/cargo-quality)](https://api.reuse.software/info/github.com/RAprogramm/cargo-quality)

Professional Rust code quality analysis tool with hardcoded standards.

## Overview

cargo-quality is a command-line tool that enforces consistent code quality standards across Rust projects without requiring local configuration files. All quality rules are hardcoded in the binary, ensuring uniform formatting and analysis across your entire codebase and organization.

## Philosophy & Standards

This tool is built on principles defined in [RustManifest](https://github.com/RAprogramm/RustManifest) - a comprehensive Rust engineering standards template.

### Why cargo-quality exists

Modern Rust development lacks a unified, zero-configuration quality tool that:

1. **Eliminates Configuration Sprawl** - Projects accumulate `.rustfmt.toml`, `.clippy.toml`, and custom scripts, each requiring maintenance and synchronization across repositories.

2. **Enforces Team Standards** - Without a central tool, each developer interprets "good code" differently, leading to inconsistent code reviews and merge conflicts.

3. **Provides Instant Feedback** - Developers need immediate, actionable feedback on code quality without waiting for CI pipelines or manual reviews.

4. **Bridges the Gap** - While `rustfmt` handles formatting and `clippy` catches bugs, neither enforces higher-level architectural patterns like import organization or argument naming conventions.

### The Solution

cargo-quality embeds battle-tested standards from RustManifest directly into its binary:

- **Single Source of Truth** - All rules versioned with the tool, not scattered across repositories
- **Zero Configuration** - Install once, use everywhere with identical behavior
- **Automated Enforcement** - From local development to CI/CD, same checks, same results
- **Actionable Fixes** - Not just detection, but automatic corrections with preview capabilities

This tool serves teams that value consistency, automation, and professional engineering practices over ad-hoc configurations.

### RustManifest Integration

[RustManifest](https://github.com/RAprogramm/RustManifest) defines:
- Code formatting standards (line width, brace style, import organization)
- Naming conventions (snake_case, PascalCase, SCREAMING_SNAKE_CASE)
- Error handling patterns (Result types, no panics outside tests)
- Documentation practices (/// docblocks, no inline comments)
- Git workflow (issue-linked branches, structured commits)
- Testing requirements (unit, integration, doctests)

cargo-quality implements these standards as enforceable rules, making RustManifest principles executable and verifiable across your entire codebase.

## Features

- **Hardcoded Quality Standards** - Single source of truth for code quality
- **Zero Configuration** - No .rustfmt.toml or config files needed
- **Code Analysis** - Detect common code quality issues
- **Automatic Fixes** - Apply fixes automatically with dry-run support
- **Format Integration** - Use cargo +nightly fmt with project standards
- **Beautiful CLI** - Colored output with helpful examples
- **CI/CD Ready** - Perfect for automated workflows

## Installation

Install from crates.io:

```bash
cargo install cargo-quality

# Setup shell completions (recommended)
cargo quality setup
```

Install from source:

```bash
git clone https://github.com/RAprogramm/cargo-quality
cd cargo-quality
cargo install --path .

# Setup shell completions (recommended)
cargo quality setup
```

### Shell Completions

After installation, set up tab completions:

```bash
# Automatic setup (recommended - detects your shell)
cargo quality setup

# Manual setup for specific shell
cargo quality completions fish > ~/.config/fish/completions/cargo.fish
cargo quality completions bash > ~/.local/share/bash-completion/completions/cargo-quality
cargo quality completions zsh > ~/.local/share/zsh/site-functions/_cargo-quality
```

**Note:** Completions will be available in new shell sessions. To use immediately, restart your shell or source the completion file.

## Requirements

- Rust 1.90 or higher
- cargo +nightly (for fmt command)

## Usage

### Quick Start

```bash
# Check code quality
cargo quality check src/

# Preview fixes
cargo quality fix --dry-run

# Apply fixes
cargo quality fix

# Format with hardcoded standards
cargo quality fmt

# Display help
cargo quality help
```

## Commands

### check

Analyze code quality without modifying files.

```bash
cargo quality check [PATH] [--verbose]
```

Options:
- `--verbose, -v` - Show detailed output for all files

Examples:
```bash
cargo quality check src/
cargo quality check --verbose .
```

### fix

Apply automatic quality fixes to your code.

```bash
cargo quality fix [PATH] [--dry-run]
```

Options:
- `--dry-run, -d` - Preview changes without modifying files

Examples:
```bash
cargo quality fix --dry-run
cargo quality fix src/
```

### fmt

Format code using cargo +nightly fmt with hardcoded project standards.

```bash
cargo quality fmt [PATH]
```

This command uses the following hardcoded configuration:
- `max_width = 99`
- `trailing_comma = "Never"`
- `brace_style = "SameLineWhere"`
- `imports_granularity = "Crate"`
- `group_imports = "StdExternalCrate"`
- `struct_field_align_threshold = 20`
- `wrap_comments = true`
- `format_code_in_doc_comments = true`
- `reorder_imports = true`
- `unstable_features = true`

The configuration is passed via command-line arguments and does not create or modify any .rustfmt.toml files.

Examples:
```bash
cargo quality fmt
cargo quality fmt src/
```

### format

Format code according to quality analyzer rules.

```bash
cargo quality format [PATH]
```

Examples:
```bash
cargo quality format .
```

### diff

Visualize proposed changes before applying fixes.

```bash
cargo quality diff [PATH] [--summary] [--interactive]
```

Options:
- `--summary, -s` - Show brief summary of changes per file
- `--interactive, -i` - Interactive mode to select which fixes to apply

Display modes:
- **Full** (default) - Shows complete diff with old/new code side-by-side
- **Summary** - Brief overview of changes grouped by analyzer
- **Interactive** - Review and approve each fix individually

Examples:
```bash
# Full diff view
cargo quality diff src/

# Summary view
cargo quality diff --summary

# Interactive mode
cargo quality diff --interactive
```

Output format:
```
Line 529
-    std::fs::write(buffer, data);
+    use std::fs::write;
+    write(buffer, data);
```

### help

Display detailed help with examples and usage patterns.

```bash
cargo quality help
```

## Analyzers

### Path Import Analyzer

Detects direct module path usage that should be moved to import statements.

Bad:
```rust
let content = std::fs::read_to_string("file.txt");
```

Good:
```rust
use std::fs::read_to_string;
let content = read_to_string("file.txt");
```

The analyzer correctly distinguishes between:
- Free functions from modules (should be imported)
- Associated functions on types (should NOT be imported, e.g., `Vec::new`)
- Enum variants (should NOT be imported, e.g., `Option::Some`)
- Associated constants (should NOT be imported, e.g., `u32::MAX`)

### Format Args Analyzer

Detects positional arguments in format macros and suggests named arguments.

Bad:
```rust
println!("Hello {}, you are {}", name, age);
```

Good:
```rust
println!("Hello {name}, you are {age}");
```

## Workflow

Typical development workflow:

1. Check your code:
```bash
cargo quality check src/
```

2. Preview fixes:
```bash
cargo quality fix --dry-run
```

3. Apply fixes:
```bash
cargo quality fix
```

4. Format code:
```bash
cargo quality fmt
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Quality Check

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: dtolnay/rust-toolchain@nightly

      - name: Install cargo-quality
        run: cargo install cargo-quality

      - name: Check code quality
        run: cargo quality check

      - name: Format check
        run: cargo quality fmt
```

## Benefits

- **Consistency** - Same standards across all projects
- **No Configuration Duplication** - Install once, use everywhere
- **Zero File I/O** - Fast execution with command-line arguments
- **Version Controlled Standards** - Update standards by updating the tool
- **Team Alignment** - Everyone uses the same quality rules

## Architecture

- **Modular Design** - Clean separation of concerns
- **Analyzer Trait** - Easy to add new analyzers
- **Zero-Cost Abstractions** - Efficient implementation
- **Comprehensive Testing** - 105 tests with 86.52% coverage
- **Performance Benchmarks** - Blazing fast (format_args: 160ns, path_import: 857ns)
- **Professional Error Handling** - Using masterror for consistency

## Development

Build from source:

```bash
cargo build --release
```

Run tests:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

Check license compliance:

```bash
reuse lint
```

## Contributing

Contributions are welcome. Please ensure:

- All tests pass
- Code follows project quality standards
- Documentation is updated
- SPDX license headers are present

## License

This project is licensed under the MIT License. See the LICENSE file for details.

SPDX-License-Identifier: MIT

## Project Information

- **Author**: RAprogramm
- **Repository**: https://github.com/RAprogramm/cargo-quality
- **Documentation**: https://docs.rs/cargo-quality
- **Standards**: [RustManifest](https://github.com/RAprogramm/RustManifest)
- **License**: MIT
