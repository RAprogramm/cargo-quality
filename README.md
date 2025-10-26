# cargo-quality

Professional Rust code quality analysis tool with hardcoded standards.

## Overview

cargo-quality is a command-line tool that enforces consistent code quality standards across Rust projects without requiring local configuration files. All quality rules are hardcoded in the binary, ensuring uniform formatting and analysis across your entire codebase and organization.

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
- **Comprehensive Testing** - 68 tests with full coverage
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
- **License**: MIT
