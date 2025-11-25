# cargo-quality

[![CI](https://github.com/RAprogramm/cargo-quality/actions/workflows/ci.yml/badge.svg)](https://github.com/RAprogramm/cargo-quality/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/RAprogramm/cargo-quality)](https://github.com/RAprogramm/cargo-quality/releases/latest)
[![Crates.io](https://img.shields.io/crates/v/cargo-quality.svg)](https://crates.io/crates/cargo-quality)
[![docs.rs](https://img.shields.io/docsrs/cargo-quality)](https://docs.rs/cargo-quality)
[![codecov](https://codecov.io/gh/RAprogramm/cargo-quality/graph/badge.svg)](https://codecov.io/gh/RAprogramm/cargo-quality)
[![Hits-of-Code](https://hitsofcode.com/github/RAprogramm/cargo-quality?branch=main&exclude=Cargo.lock,.gitignore,CHANGELOG.md)](https://hitsofcode.com/github/RAprogramm/cargo-quality/view?branch=main&exclude=Cargo.lock,.gitignore,CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/RAprogramm/cargo-quality/blob/main/LICENSES/MIT.txt)
[![REUSE](https://api.reuse.software/badge/github.com/RAprogramm/cargo-quality)](https://api.reuse.software/info/github.com/RAprogramm/cargo-quality)

Professional Rust code quality analysis tool with hardcoded standards.

## Overview

cargo-quality is a command-line tool that enforces consistent code quality standards across Rust projects without requiring local configuration files. All quality rules are hardcoded in the binary, ensuring uniform formatting and analysis across your entire codebase and organization.

<img width="1483" height="907" alt="image" src="https://github.com/user-attachments/assets/74a2449e-7231-468d-a660-28d2fedd1c5d" />


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
- **Selective Execution** - Run specific analyzers with `--analyzer` flag
- **Dual Output Modes** - Compact (grouped) and verbose (detailed) output
- **Format Integration** - Use cargo +nightly fmt with project standards
- **Beautiful CLI** - Colored output with helpful examples
- **CI/CD Ready** - Perfect for automated workflows

## Installation

Install from crates.io:

```bash
cargo install cargo-quality

# Setup shell completions (recommended)
cargo qual setup
```

Install from source:

```bash
git clone https://github.com/RAprogramm/cargo-quality
cd cargo-quality
cargo install --path .

# Setup shell completions (recommended)
cargo qual setup
```

### Shell Completions

After installation, set up tab completions:

```bash
# Automatic setup (recommended - detects your shell)
cargo qual setup

# Manual setup for specific shell
cargo qual completions fish > ~/.config/fish/completions/cargo.fish
cargo qual completions bash > ~/.local/share/bash-completion/completions/cargo-quality
cargo qual completions zsh > ~/.local/share/zsh/site-functions/_cargo-quality
```

**Note:** Completions will be available in new shell sessions. To use immediately, restart your shell or source the completion file.

## Requirements

- Rust 1.90 or higher
- cargo +nightly (for fmt command)

## Usage

### Quick Start

```bash
# Check code quality (compact output by default)
cargo qual check src/

# Check with detailed output
cargo qual check --verbose src/

# Check specific analyzer only
cargo qual check -a inline_comments

# Preview fixes
cargo qual fix --dry-run

# Apply fixes from specific analyzer
cargo qual fix -a path_import

# Format with hardcoded standards
cargo qual fmt

# Display help
cargo qual help
```

## Commands

### check

Analyze code quality without modifying files.

```bash
cargo qual check [PATH] [--verbose] [--analyzer <NAME>] [--color]
```

Options:
- `--verbose, -v` - Show detailed output for all files (every issue separately)
- `--analyzer, -a <NAME>` - Run specific analyzer only
- `--color, -c` - Enable colored output with syntax highlighting

**Output Modes:**

**Compact Mode (Default)** - Groups identical messages together with grid layout:
```
[empty_lines] - 42 issues          [format_args] - 7 issues
────────────────────────────       ────────────────────────────
Empty line in function body...     Use named format arguments...

src/report.rs → Lines:             src/report.rs → Lines: 167
74, 78, 83, 91, 93, 98...         src/differ/display.rs → Lines:
src/main.rs → Lines:               106, 116, 171, 183
49, 88, 102, 105, 113...

════════════════════════════       ════════════════════════════
```

Features:
- **Responsive grid layout** - Automatically arranges analyzers in columns based on terminal width
- **Beautiful separators** - Clear visual boundaries between analyzer blocks
- **Smart grouping** - Identical issues grouped across all files
- **File-by-file breakdown** - Shows which files have each issue

**Verbose Mode (--verbose flag)** - Shows every issue separately with full details:
```
[empty_lines]
  74:1 - Empty line in function body indicates untamed complexity
    Fix:
  78:1 - Empty line in function body indicates untamed complexity
    Fix:
  ...
```

**Colored Output (--color flag)** - Syntax highlighting for better readability:
- Analyzer names: yellow + bold
- Issue counts: cyan
- File paths: blue
- Line numbers: magenta
- Summary: green + bold

**Selective Execution** - Run specific analyzers:
```bash
# Run only inline comments analyzer
cargo qual check -a inline_comments

# Run only path import analyzer
cargo qual check -a path_import
```

Examples:
```bash
# Check with compact output (default)
cargo qual check src/

# Check with detailed output
cargo qual check --verbose .

# Check with colored output
cargo qual check --color src/

# Check only inline comments
cargo qual check -a inline_comments
```

### fix

Apply automatic quality fixes to your code.

```bash
cargo qual fix [PATH] [--dry-run] [--analyzer <NAME>]
```

Options:
- `--dry-run, -d` - Preview changes without modifying files
- `--analyzer, -a <NAME>` - Apply fixes from specific analyzer only

Examples:
```bash
# Preview all fixes
cargo qual fix --dry-run

# Apply all fixes
cargo qual fix src/

# Apply only path import fixes
cargo qual fix -a path_import
```

### fmt

Format code using cargo +nightly fmt with hardcoded project standards.

```bash
cargo qual fmt [PATH]
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
cargo qual fmt
cargo qual fmt src/
```

### format

Format code according to quality analyzer rules.

```bash
cargo qual format [PATH]
```

Examples:
```bash
cargo qual format .
```

### diff

Visualize proposed changes before applying fixes.

```bash
cargo qual diff [PATH] [--summary] [--interactive] [--analyzer <NAME>] [--color]
```

Options:
- `--summary, -s` - Show brief summary of changes per file
- `--interactive, -i` - Interactive mode to select which fixes to apply
- `--analyzer, -a <NAME>` - Show diff for specific analyzer only
- `--color, -c` - Enable colored output with syntax highlighting

Display modes:
- **Full** (default) - Shows complete diff with old/new code side-by-side
- **Summary** - Brief overview of changes grouped by analyzer
- **Interactive** - Review and approve each fix individually

Examples:
```bash
# Full diff view
cargo qual diff src/

# Summary view
cargo qual diff --summary

# Interactive mode
cargo qual diff --interactive

# Show only path import changes
cargo qual diff -a path_import

# Colored output
cargo qual diff --color --summary
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
cargo qual help
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

### Empty Lines Analyzer

Detects empty lines inside function and method bodies that indicate untamed complexity. Based on principles from [Empty Line Code Smell](https://www.yegor256.com/2014/11/03/empty-line-code-smell.html).

Bad:
```rust
fn process() {
    let x = read();

    let y = transform(x);

    write(y);
}
```

Good:
```rust
fn process() {
    let x = read();
    let y = transform(x);
    write(y);
}
```

When running `cargo qual diff`, empty lines are shown as a summary note:
```
Note: 3 empty lines will be removed from lines: 3, 5, 11
```

### Inline Comments Analyzer

Detects inline comments (`//`) inside function and method bodies. According to professional documentation standards, all explanations should be in doc comments (`///`), specifically in the `# Notes` section with code context.

Bad:
```rust
fn calculate(x: i32, y: i32) -> i32 {
    // Add the numbers
    let sum = x + y;
    // Multiply by 2
    let result = sum * 2;
    // Return final result
    result
}
```

Good:
```rust
/// Calculate something
///
/// # Notes
///
/// - Add the numbers - `let sum = x + y;`
/// - Multiply by 2 - `let result = sum * 2;`
/// - Return final result - `result`
fn calculate(x: i32, y: i32) -> i32 {
    let sum = x + y;
    let result = sum * 2;
    result
}
```

**Important:** This analyzer only detects issues and provides suggestions. It does not apply automatic fixes (`Fix::None`). Use `cargo qual check -a inline_comments` to see all inline comments that should be moved to doc blocks.

When running `cargo qual check -a inline_comments`, the output shows:
```
[inline_comments] - 3 issues
  Inline comment found: "Add the numbers"
Move to doc block # Notes section:
/// - Add the numbers - `let sum = x + y;`
  → Lines: 2

  Inline comment found: "Multiply by 2"
Move to doc block # Notes section:
/// - Multiply by 2 - `let result = sum * 2;`
  → Lines: 4
```

## Available Analyzers

Run specific analyzers using the `--analyzer` or `-a` flag:

- `path_import` - Path Import Analyzer
- `format_args` - Format Args Analyzer
- `empty_lines` - Empty Lines Analyzer
- `inline_comments` - Inline Comments Analyzer

Example:
```bash
# Check only inline comments
cargo qual check -a inline_comments

# Fix only path imports
cargo qual fix -a path_import

# Show diff only for empty lines
cargo qual diff -a empty_lines
```

## Workflow

Typical development workflow:

1. Check your code:
```bash
cargo qual check src/
```

2. Preview fixes:
```bash
cargo qual fix --dry-run
```

3. Apply fixes:
```bash
cargo qual fix
```

4. Format code:
```bash
cargo qual fmt
```

## GitHub Action

Use cargo-quality directly in your CI/CD pipelines:

```yaml
name: Quality Check

on: [pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run cargo-quality
        uses: RAprogramm/cargo-quality@v1
        with:
          path: 'src/'
          fail_on_issues: 'true'
          post_comment: 'true'
```

### Action Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `path` | Path to analyze | `src/` |
| `analyzer` | Specific analyzer to run | (all) |
| `fail_on_issues` | Fail if issues found | `true` |
| `post_comment` | Post results as PR comment | `false` |
| `update_comment` | Update existing comment | `true` |

### Action Outputs

| Output | Description |
|--------|-------------|
| `total_issues` | Total number of issues found |
| `path_import_issues` | Issues from path_import analyzer |
| `format_args_issues` | Issues from format_args analyzer |
| `empty_lines_issues` | Issues from empty_lines analyzer |
| `inline_comments_issues` | Issues from inline_comments analyzer |
| `has_issues` | Whether any issues were found |

### Advanced Usage

Run specific analyzer only:

```yaml
- uses: RAprogramm/cargo-quality@v1
  with:
    analyzer: 'path_import'
    fail_on_issues: 'false'
```

Use outputs in subsequent steps:

```yaml
- uses: RAprogramm/cargo-quality@v1
  id: quality

- name: Check results
  if: steps.quality.outputs.has_issues == 'true'
  run: echo "Found ${{ steps.quality.outputs.total_issues }} issues"
```

## CI/CD Integration (Manual)

If you prefer manual installation:

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
        run: cargo qual check

      - name: Format check
        run: cargo qual fmt
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
