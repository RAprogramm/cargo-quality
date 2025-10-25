// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    fs,
    io::{self, Write},
    path::PathBuf
};

use masterror::AppResult;
use owo_colors::OwoColorize;

use crate::{
    analyzer::Analyzer,
    error::{IoError, ParseError}
};

/// Represents a single code change.
///
/// Stores the location and content of a proposed modification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    pub line:        usize,
    pub analyzer:    String,
    pub original:    String,
    pub modified:    String,
    pub description: String
}

/// Diff results for a single file.
///
/// Contains all proposed changes grouped by analyzer.
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path:    String,
    pub entries: Vec<DiffEntry>
}

impl FileDiff {
    /// Creates a new file diff result.
    ///
    /// # Arguments
    ///
    /// * `path` - File path
    ///
    /// # Returns
    ///
    /// Empty `FileDiff` structure
    pub fn new(path: String) -> Self {
        Self {
            path,
            entries: Vec::new()
        }
    }

    /// Adds a diff entry to the file.
    ///
    /// # Arguments
    ///
    /// * `entry` - Diff entry to add
    pub fn add_entry(&mut self, entry: DiffEntry) {
        self.entries.push(entry);
    }

    /// Returns total number of changes.
    ///
    /// # Returns
    ///
    /// Number of diff entries
    pub fn total_changes(&self) -> usize {
        self.entries.len()
    }
}

/// Complete diff results for all files.
///
/// Aggregates changes across multiple files.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub files: Vec<FileDiff>
}

impl DiffResult {
    /// Creates a new empty diff result.
    ///
    /// # Returns
    ///
    /// Empty `DiffResult` structure
    pub fn new() -> Self {
        Self {
            files: Vec::new()
        }
    }

    /// Adds file diff to results.
    ///
    /// # Arguments
    ///
    /// * `file_diff` - File diff to add
    pub fn add_file(&mut self, file_diff: FileDiff) {
        if file_diff.total_changes() > 0 {
            self.files.push(file_diff);
        }
    }

    /// Returns total number of changes across all files.
    ///
    /// # Returns
    ///
    /// Total change count
    pub fn total_changes(&self) -> usize {
        self.files.iter().map(|f| f.total_changes()).sum()
    }

    /// Returns number of files with changes.
    ///
    /// # Returns
    ///
    /// File count
    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates diff showing proposed changes.
///
/// Analyzes files and compares current state with proposed fixes.
///
/// # Arguments
///
/// * `file_path` - Path to analyze
/// * `analyzers` - List of analyzers to apply
///
/// # Returns
///
/// `AppResult<DiffResult>` - Diff results or error
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::{analyzers::get_analyzers, differ::generate_diff};
/// let diff = generate_diff("src/main.rs", &get_analyzers()).unwrap();
/// ```
pub fn generate_diff(file_path: &str, analyzers: &[Box<dyn Analyzer>]) -> AppResult<FileDiff> {
    let content = fs::read_to_string(file_path).map_err(IoError::from)?;
    let mut ast = syn::parse_file(&content).map_err(ParseError::from)?;

    let mut file_diff = FileDiff::new(file_path.to_string());

    for analyzer in analyzers {
        let result = analyzer.analyze(&ast)?;

        for issue in result.issues {
            let original_content = content
                .lines()
                .nth(issue.line.saturating_sub(1))
                .unwrap_or("");

            let original_ast = ast.clone();
            analyzer.fix(&mut ast)?;
            let modified_content = prettyplease::unparse(&ast);
            let modified_line = modified_content
                .lines()
                .nth(issue.line.saturating_sub(1))
                .unwrap_or("");

            ast = original_ast;

            let entry = DiffEntry {
                line:        issue.line,
                analyzer:    analyzer.name().to_string(),
                original:    original_content.to_string(),
                modified:    modified_line.to_string(),
                description: issue.message
            };

            file_diff.add_entry(entry);
        }
    }

    Ok(file_diff)
}

/// Displays diff in summary mode.
///
/// Shows brief statistics for each file.
///
/// # Arguments
///
/// * `result` - Diff results to display
pub fn show_summary(result: &DiffResult) {
    println!("\n{}\n", "DIFF SUMMARY".bold());

    for file in &result.files {
        println!("{}:", file.path.cyan().bold());

        let mut analyzer_counts = std::collections::HashMap::new();
        for entry in &file.entries {
            *analyzer_counts.entry(&entry.analyzer).or_insert(0) += 1;
        }

        for (analyzer, count) in analyzer_counts {
            println!(
                "  {}: {} {}",
                analyzer.green(),
                count,
                if count == 1 { "issue" } else { "issues" }
            );
        }
        println!();
    }

    println!(
        "{}",
        format!(
            "Total: {} changes in {} files",
            result.total_changes(),
            result.total_files()
        )
        .yellow()
        .bold()
    );
}

/// Displays full unified diff output.
///
/// Shows detailed line-by-line changes.
///
/// # Arguments
///
/// * `result` - Diff results to display
pub fn show_full(result: &DiffResult) {
    println!("\n{}\n", "DIFF OUTPUT".bold());

    for file in &result.files {
        println!("{}", format!("File: {}", file.path).cyan().bold());
        println!("{}", "────────────────────────────────────────".dimmed());

        let mut last_analyzer = String::new();
        for entry in &file.entries {
            if entry.analyzer != last_analyzer {
                if !last_analyzer.is_empty() {
                    println!();
                }
                println!(
                    "{} ({} issues)",
                    entry.analyzer.green().bold(),
                    file.entries
                        .iter()
                        .filter(|e| e.analyzer == entry.analyzer)
                        .count()
                );
                println!();
                last_analyzer = entry.analyzer.clone();
            }

            println!("{}", format!("--- Line {}", entry.line).red());
            println!("{}", format!("+++ Line {}", entry.line).green());
            println!("{}", format!("-    {}", entry.original).red());
            println!("{}", format!("+    {}", entry.modified).green());
            println!();
        }

        println!("{}", "════════════════════════════════════════".dimmed());
    }

    println!(
        "\n{}",
        format!(
            "Total: {} changes in {} files",
            result.total_changes(),
            result.total_files()
        )
        .yellow()
        .bold()
    );
}

/// Displays interactive diff with user prompts.
///
/// Shows each change and asks for confirmation.
///
/// # Arguments
///
/// * `result` - Diff results to display
///
/// # Returns
///
/// `AppResult<Vec<DiffEntry>>` - Selected entries or error
pub fn show_interactive(result: &DiffResult) -> AppResult<Vec<DiffEntry>> {
    let mut selected = Vec::new();
    let mut apply_all = false;

    println!("\n{}\n", "INTERACTIVE DIFF".bold());
    println!("{}", "Commands: y=yes, n=no, a=all, q=quit\n".dimmed());

    for file in &result.files {
        println!("{}", format!("File: {}", file.path).cyan().bold());
        println!();

        for (idx, entry) in file.entries.iter().enumerate() {
            println!(
                "{} {}",
                format!("[{}/{}]", idx + 1, file.entries.len()).yellow(),
                entry.analyzer.green()
            );
            println!("{}", format!("Line {}:", entry.line).dimmed());
            println!("{}", format!("- {}", entry.original).red());
            println!("{}", format!("+ {}", entry.modified).green());
            println!();

            if apply_all {
                selected.push(entry.clone());
                continue;
            }

            print!("{}", "Apply this fix? [y/n/a/q]: ".bold());
            io::stdout().flush().map_err(IoError::from)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(IoError::from)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => {
                    selected.push(entry.clone());
                    println!("{}", "Applied".green());
                }
                "n" | "no" => {
                    println!("{}", "Skipped".yellow());
                }
                "a" | "all" => {
                    apply_all = true;
                    selected.push(entry.clone());
                    println!("{}", "Applying all remaining changes".green().bold());
                }
                "q" | "quit" => {
                    println!("{}", "Quit".red());
                    break;
                }
                _ => {
                    println!("{}", "Invalid input, skipping".red());
                }
            }
            println!();
        }
    }

    println!(
        "\n{}",
        format!("Selected {} changes for application", selected.len())
            .yellow()
            .bold()
    );

    Ok(selected)
}

/// Collects Rust files from path.
///
/// # Arguments
///
/// * `path` - Directory or file path
///
/// # Returns
///
/// `AppResult<Vec<PathBuf>>` - List of Rust files
pub fn collect_files(path: &str) -> AppResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path_buf = PathBuf::from(path);

    if path_buf.is_file() && path_buf.extension().map_or(false, |e| e == "rs") {
        files.push(path_buf);
    } else if path_buf.is_dir() {
        for entry in walkdir::WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "rs" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_entry_creation() {
        let entry = DiffEntry {
            line:        10,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string()
        };

        assert_eq!(entry.line, 10);
        assert_eq!(entry.analyzer, "test");
    }

    #[test]
    fn test_file_diff_new() {
        let diff = FileDiff::new("test.rs".to_string());
        assert_eq!(diff.path, "test.rs");
        assert_eq!(diff.total_changes(), 0);
    }

    #[test]
    fn test_file_diff_add_entry() {
        let mut diff = FileDiff::new("test.rs".to_string());
        let entry = DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string()
        };

        diff.add_entry(entry);
        assert_eq!(diff.total_changes(), 1);
    }

    #[test]
    fn test_diff_result_new() {
        let result = DiffResult::new();
        assert_eq!(result.total_changes(), 0);
        assert_eq!(result.total_files(), 0);
    }

    #[test]
    fn test_diff_result_add_file() {
        let mut result = DiffResult::new();
        let mut file_diff = FileDiff::new("test.rs".to_string());

        let entry = DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string()
        };

        file_diff.add_entry(entry);
        result.add_file(file_diff);

        assert_eq!(result.total_files(), 1);
        assert_eq!(result.total_changes(), 1);
    }

    #[test]
    fn test_diff_result_skip_empty_files() {
        let mut result = DiffResult::new();
        let file_diff = FileDiff::new("test.rs".to_string());
        result.add_file(file_diff);

        assert_eq!(result.total_files(), 0);
    }

    #[test]
    fn test_show_summary_no_panic() {
        let result = DiffResult::new();
        show_summary(&result);
    }

    #[test]
    fn test_show_full_no_panic() {
        let result = DiffResult::new();
        show_full(&result);
    }

    #[test]
    fn test_show_summary_with_data() {
        let mut result = DiffResult::new();
        let mut file_diff = FileDiff::new("test.rs".to_string());

        let entry1 = DiffEntry {
            line:        1,
            analyzer:    "test_analyzer".to_string(),
            original:    "old line".to_string(),
            modified:    "new line".to_string(),
            description: "test change".to_string()
        };

        let entry2 = DiffEntry {
            line:        2,
            analyzer:    "test_analyzer".to_string(),
            original:    "old line 2".to_string(),
            modified:    "new line 2".to_string(),
            description: "test change 2".to_string()
        };

        file_diff.add_entry(entry1);
        file_diff.add_entry(entry2);
        result.add_file(file_diff);

        show_summary(&result);
    }

    #[test]
    fn test_show_full_with_data() {
        let mut result = DiffResult::new();
        let mut file_diff = FileDiff::new("test.rs".to_string());

        let entry = DiffEntry {
            line:        10,
            analyzer:    "format_args".to_string(),
            original:    "println!(\"Hello {}\", name)".to_string(),
            modified:    "println!(\"Hello {name}\")".to_string(),
            description: "Use named arguments".to_string()
        };

        file_diff.add_entry(entry);
        result.add_file(file_diff);

        show_full(&result);
    }

    #[test]
    fn test_collect_files_single_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let result = collect_files(file_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_collect_files_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test1.rs");
        let file2 = temp_dir.path().join("test2.rs");
        let file3 = temp_dir.path().join("test.txt");

        std::fs::write(&file1, "fn test1() {}").unwrap();
        std::fs::write(&file2, "fn test2() {}").unwrap();
        std::fs::write(&file3, "not rust").unwrap();

        let result = collect_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_collect_files_empty_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = collect_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_collect_files_non_rust() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "not rust").unwrap();

        let result = collect_files(file_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_generate_diff_integration() {
        use tempfile::TempDir;

        use crate::analyzers::get_analyzers;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(
            &file_path,
            "fn main() { let x = std::fs::read_to_string(\"f\"); }"
        )
        .unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers);

        assert!(result.is_ok());
        let file_diff = result.unwrap();
        assert!(!file_diff.entries.is_empty());
    }

    #[test]
    fn test_generate_diff_no_issues() {
        use tempfile::TempDir;

        use crate::analyzers::get_analyzers;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "fn main() {}").unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_diff_invalid_syntax() {
        use tempfile::TempDir;

        use crate::analyzers::get_analyzers;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "fn main() { invalid syntax +++").unwrap();

        let analyzers = get_analyzers();
        let result = generate_diff(file_path.to_str().unwrap(), &analyzers);

        assert!(result.is_err());
    }

    #[test]
    fn test_diff_result_multiple_files() {
        let mut result = DiffResult::new();

        let mut file1 = FileDiff::new("file1.rs".to_string());
        file1.add_entry(DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string()
        });

        let mut file2 = FileDiff::new("file2.rs".to_string());
        file2.add_entry(DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string()
        });

        result.add_file(file1);
        result.add_file(file2);

        assert_eq!(result.total_files(), 2);
        assert_eq!(result.total_changes(), 2);
    }
}
