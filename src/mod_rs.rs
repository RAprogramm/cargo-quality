// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Module for detecting and fixing `mod.rs` files.
//!
//! This module provides functionality to find `mod.rs` files in a project
//! and convert them to the modern module naming convention where modules
//! are named after their parent directory.
//!
//! # Example
//!
//! ```text
//! Before: src/analyzers/mod.rs
//! After:  src/analyzers.rs
//! ```
//!
//! The `mod.rs` file content is moved to a file named after the parent
//! directory, placed one level up in the directory hierarchy.

use std::{
    fs,
    path::{Path, PathBuf}
};

use masterror::AppResult;

use crate::error::IoError;

/// Result of mod.rs detection.
///
/// Contains information about a found `mod.rs` file and the suggested fix.
#[derive(Debug, Clone)]
pub struct ModRsIssue {
    /// Path to the mod.rs file
    pub path:      PathBuf,
    /// Suggested new path after fix
    pub suggested: PathBuf,
    /// Human-readable message
    pub message:   String,
    /// Line number (always 1 for file-level issues)
    pub line:      usize,
    /// Column number (always 1 for file-level issues)
    pub column:    usize
}

/// Result of mod.rs analysis.
///
/// Contains all found `mod.rs` files in the analyzed path.
#[derive(Debug, Default)]
pub struct ModRsResult {
    /// List of found mod.rs issues
    pub issues: Vec<ModRsIssue>
}

impl ModRsResult {
    /// Creates new empty result.
    #[inline]
    pub fn new() -> Self {
        Self {
            issues: Vec::new()
        }
    }

    /// Returns total number of issues found.
    #[inline]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Checks if no issues were found.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Finds all `mod.rs` files in the given path.
///
/// Recursively searches for files named `mod.rs` that should be converted
/// to the modern module naming convention.
///
/// # Arguments
///
/// * `path` - Root path to search in
///
/// # Returns
///
/// `AppResult<ModRsResult>` containing all found `mod.rs` files
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::mod_rs::find_mod_rs_issues;
///
/// let result = find_mod_rs_issues("src/").unwrap();
/// println!("Found {} mod.rs files", result.len());
/// ```
pub fn find_mod_rs_issues(path: &str) -> AppResult<ModRsResult> {
    let root = Path::new(path);
    let mut result = ModRsResult::new();

    if root.is_file() {
        if is_mod_rs(root)
            && let Some(issue) = create_issue(root)
        {
            result.issues.push(issue);
        }
        return Ok(result);
    }

    collect_mod_rs_recursive(root, &mut result)?;
    Ok(result)
}

/// Recursively collects mod.rs files from directory.
///
/// # Arguments
///
/// * `dir` - Directory to search in
/// * `result` - Result accumulator
fn collect_mod_rs_recursive(dir: &Path, result: &mut ModRsResult) -> AppResult<()> {
    let entries = fs::read_dir(dir).map_err(IoError::from)?;

    for entry in entries {
        let entry = entry.map_err(IoError::from)?;
        let path = entry.path();

        if path.is_dir() {
            collect_mod_rs_recursive(&path, result)?;
        } else if is_mod_rs(&path)
            && let Some(issue) = create_issue(&path)
        {
            result.issues.push(issue);
        }
    }

    Ok(())
}

/// Checks if path points to a mod.rs file.
///
/// # Arguments
///
/// * `path` - Path to check
///
/// # Returns
///
/// `true` if the file is named `mod.rs`
#[inline]
fn is_mod_rs(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == "mod.rs")
        .unwrap_or(false)
}

/// Creates an issue for a mod.rs file.
///
/// # Arguments
///
/// * `path` - Path to the mod.rs file
///
/// # Returns
///
/// `Some(ModRsIssue)` if the file has a valid parent directory
fn create_issue(path: &Path) -> Option<ModRsIssue> {
    let parent = path.parent()?;
    let module_name = parent.file_name()?.to_str()?;
    let grandparent = parent.parent()?;

    let suggested = grandparent.join(format!("{}.rs", module_name));

    Some(ModRsIssue {
        path: path.to_path_buf(),
        suggested,
        message: format!(
            "Use `{}.rs` instead of `{}/mod.rs` (modern module style)",
            module_name, module_name
        ),
        line: 1,
        column: 1
    })
}

/// Fixes a single mod.rs file by renaming and moving it.
///
/// Converts `src/foo/mod.rs` to `src/foo.rs` by:
/// 1. Reading the content of mod.rs
/// 2. Writing it to the new location (parent_name.rs)
/// 3. Removing the original mod.rs file
/// 4. Removing the empty parent directory if it becomes empty
///
/// # Arguments
///
/// * `issue` - The mod.rs issue to fix
///
/// # Returns
///
/// `AppResult<()>` - Ok if fix was successful
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::mod_rs::{find_mod_rs_issues, fix_mod_rs};
///
/// let result = find_mod_rs_issues("src/").unwrap();
/// for issue in result.issues {
///     fix_mod_rs(&issue).unwrap();
/// }
/// ```
pub fn fix_mod_rs(issue: &ModRsIssue) -> AppResult<()> {
    let content = fs::read_to_string(&issue.path).map_err(IoError::from)?;

    fs::write(&issue.suggested, content).map_err(IoError::from)?;

    fs::remove_file(&issue.path).map_err(IoError::from)?;

    if let Some(parent) = issue.path.parent()
        && is_directory_empty(parent)?
    {
        fs::remove_dir(parent).map_err(IoError::from)?;
    }

    Ok(())
}

/// Fixes all mod.rs files found in the given path.
///
/// # Arguments
///
/// * `path` - Root path to search and fix
///
/// # Returns
///
/// `AppResult<usize>` - Number of files fixed
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::mod_rs::fix_all_mod_rs;
///
/// let fixed = fix_all_mod_rs("src/").unwrap();
/// println!("Fixed {} mod.rs files", fixed);
/// ```
pub fn fix_all_mod_rs(path: &str) -> AppResult<usize> {
    let result = find_mod_rs_issues(path)?;
    let count = result.len();

    for issue in result.issues {
        fix_mod_rs(&issue)?;
    }

    Ok(count)
}

/// Checks if a directory is empty.
///
/// # Arguments
///
/// * `dir` - Directory path to check
///
/// # Returns
///
/// `AppResult<bool>` - true if directory has no entries
fn is_directory_empty(dir: &Path) -> AppResult<bool> {
    let mut entries = fs::read_dir(dir).map_err(IoError::from)?;
    Ok(entries.next().is_none())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_find_no_mod_rs() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("lib.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_mod_rs() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("analyzers");
        fs::create_dir(&subdir).unwrap();
        let mod_rs = subdir.join("mod.rs");
        fs::write(&mod_rs, "pub mod test;").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.issues[0].message.contains("analyzers"));
    }

    #[test]
    fn test_find_multiple_mod_rs() {
        let temp = TempDir::new().unwrap();

        let dir1 = temp.path().join("foo");
        fs::create_dir(&dir1).unwrap();
        fs::write(dir1.join("mod.rs"), "// foo").unwrap();

        let dir2 = temp.path().join("bar");
        fs::create_dir(&dir2).unwrap();
        fs::write(dir2.join("mod.rs"), "// bar").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fix_mod_rs() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("utils");
        fs::create_dir(&subdir).unwrap();
        let mod_rs = subdir.join("mod.rs");
        fs::write(&mod_rs, "pub fn helper() {}").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);

        fix_mod_rs(&result.issues[0]).unwrap();

        assert!(!mod_rs.exists());
        let new_file = temp.path().join("utils.rs");
        assert!(new_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "pub fn helper() {}");
        assert!(!subdir.exists());
    }

    #[test]
    fn test_fix_mod_rs_keeps_dir_with_other_files() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("services");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("mod.rs"), "pub mod api;").unwrap();
        fs::write(subdir.join("api.rs"), "fn api() {}").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        fix_mod_rs(&result.issues[0]).unwrap();

        assert!(subdir.exists());
        assert!(subdir.join("api.rs").exists());
        assert!(temp.path().join("services.rs").exists());
    }

    #[test]
    fn test_fix_all_mod_rs() {
        let temp = TempDir::new().unwrap();

        let dir1 = temp.path().join("module1");
        fs::create_dir(&dir1).unwrap();
        fs::write(dir1.join("mod.rs"), "// 1").unwrap();

        let dir2 = temp.path().join("module2");
        fs::create_dir(&dir2).unwrap();
        fs::write(dir2.join("mod.rs"), "// 2").unwrap();

        let fixed = fix_all_mod_rs(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(fixed, 2);

        assert!(temp.path().join("module1.rs").exists());
        assert!(temp.path().join("module2.rs").exists());
    }

    #[test]
    fn test_issue_message() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("handlers");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("mod.rs"), "").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert!(result.issues[0].message.contains("handlers.rs"));
        assert!(result.issues[0].message.contains("handlers/mod.rs"));
    }

    #[test]
    fn test_suggested_path() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("core");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("mod.rs"), "").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.issues[0].suggested, temp.path().join("core.rs"));
    }

    #[test]
    fn test_nested_mod_rs() {
        let temp = TempDir::new().unwrap();
        let level1 = temp.path().join("level1");
        let level2 = level1.join("level2");
        fs::create_dir_all(&level2).unwrap();
        fs::write(level2.join("mod.rs"), "// nested").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.issues[0].message.contains("level2"));
        assert_eq!(result.issues[0].suggested, level1.join("level2.rs"));
    }

    #[test]
    fn test_single_file_check() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("single");
        fs::create_dir(&subdir).unwrap();
        let mod_rs = subdir.join("mod.rs");
        fs::write(&mod_rs, "").unwrap();

        let result = find_mod_rs_issues(mod_rs.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_non_mod_rs_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("lib.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let result = find_mod_rs_issues(file.to_str().unwrap()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_line_column() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("pos");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("mod.rs"), "").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.issues[0].line, 1);
        assert_eq!(result.issues[0].column, 1);
    }

    #[test]
    fn test_empty_directory() {
        let temp = TempDir::new().unwrap();
        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_result_default() {
        let result = ModRsResult::default();
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
    }
}
