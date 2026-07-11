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
    fs::{read_dir, remove_dir as remove_directory, rename},
    io,
    path::{Path, PathBuf}
};

use ignore::WalkBuilder;
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

    collect_mod_rs_walked(path, &mut result);
    Ok(result)
}

/// Collects mod.rs files while respecting ignore rules.
///
/// Walks the tree with [`WalkBuilder`], which honors `.gitignore`/`.ignore`
/// and skips hidden directories (e.g. `.git`), matching
/// [`crate::file_utils::collect_rust_files`]. This prevents scanning build
/// artifacts and vendored dependencies under `target/`.
///
/// # Arguments
///
/// * `path` - Root directory to search in
/// * `result` - Result accumulator
fn collect_mod_rs_walked(path: &str, result: &mut ModRsResult) {
    for entry in WalkBuilder::new(path)
        .follow_links(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build()
        .flatten()
    {
        let entry_path = entry.path();

        if entry.file_type().is_some_and(|ft| ft.is_file())
            && is_mod_rs(entry_path)
            && let Some(issue) = create_issue(entry_path)
        {
            result.issues.push(issue);
        }
    }
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
    if issue.suggested.exists() {
        return Err(IoError::from(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "target already exists, refusing to overwrite: {}",
                issue.suggested.display()
            )
        ))
        .into());
    }

    rename(&issue.path, &issue.suggested).map_err(IoError::from)?;
    if let Some(parent) = issue.path.parent()
        && is_directory_empty(parent)?
    {
        remove_directory(parent).map_err(IoError::from)?;
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
    let mut applied = 0;

    for issue in result.issues {
        if issue.suggested.exists() {
            eprintln!(
                "Skipping {}: target {} already exists",
                issue.path.display(),
                issue.suggested.display()
            );
            continue;
        }

        fix_mod_rs(&issue)?;
        applied += 1;
    }

    Ok(applied)
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
    let mut entries = read_dir(dir).map_err(IoError::from)?;
    Ok(entries.next().is_none())
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, read_to_string, write};

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_find_no_mod_rs() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("lib.rs");
        write(&file, "fn main() {}").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_mod_rs() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("analyzers");
        create_dir(&subdir).unwrap();
        let mod_rs = subdir.join("mod.rs");
        write(&mod_rs, "pub mod test;").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.issues[0].message.contains("analyzers"));
    }

    #[test]
    fn test_find_multiple_mod_rs() {
        let temp = TempDir::new().unwrap();

        let dir1 = temp.path().join("foo");
        create_dir(&dir1).unwrap();
        write(dir1.join("mod.rs"), "// foo").unwrap();

        let dir2 = temp.path().join("bar");
        create_dir(&dir2).unwrap();
        write(dir2.join("mod.rs"), "// bar").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fix_mod_rs() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("utils");
        create_dir(&subdir).unwrap();
        let mod_rs = subdir.join("mod.rs");
        write(&mod_rs, "pub fn helper() {}").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);

        fix_mod_rs(&result.issues[0]).unwrap();

        assert!(!mod_rs.exists());
        let new_file = temp.path().join("utils.rs");
        assert!(new_file.exists());
        assert_eq!(read_to_string(&new_file).unwrap(), "pub fn helper() {}");
        assert!(!subdir.exists());
    }

    #[test]
    fn test_fix_mod_rs_keeps_dir_with_other_files() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("services");
        create_dir(&subdir).unwrap();
        write(subdir.join("mod.rs"), "pub mod api;").unwrap();
        write(subdir.join("api.rs"), "fn api() {}").unwrap();

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
        create_dir(&dir1).unwrap();
        write(dir1.join("mod.rs"), "// 1").unwrap();

        let dir2 = temp.path().join("module2");
        create_dir(&dir2).unwrap();
        write(dir2.join("mod.rs"), "// 2").unwrap();

        let fixed = fix_all_mod_rs(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(fixed, 2);

        assert!(temp.path().join("module1.rs").exists());
        assert!(temp.path().join("module2.rs").exists());
    }

    #[test]
    fn test_fix_mod_rs_refuses_to_overwrite_existing() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("foo");
        create_dir(&subdir).unwrap();
        write(subdir.join("mod.rs"), "MOD CONTENT").unwrap();
        let sibling = temp.path().join("foo.rs");
        write(&sibling, "KEEP ME").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert!(fix_mod_rs(&result.issues[0]).is_err());
        assert_eq!(read_to_string(&sibling).unwrap(), "KEEP ME");
        assert!(subdir.join("mod.rs").exists());
    }

    #[test]
    fn test_fix_all_skips_conflicting_target() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("foo");
        create_dir(&subdir).unwrap();
        write(subdir.join("mod.rs"), "MOD CONTENT").unwrap();
        write(temp.path().join("foo.rs"), "KEEP ME").unwrap();

        let applied = fix_all_mod_rs(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(applied, 0);
        assert_eq!(read_to_string(temp.path().join("foo.rs")).unwrap(), "KEEP ME");
        assert!(subdir.join("mod.rs").exists());
    }

    #[test]
    fn test_scan_respects_gitignore_and_hidden_dirs() {
        let temp = TempDir::new().unwrap();

        create_dir(temp.path().join(".git")).unwrap();
        create_dir(temp.path().join(".git").join("h")).unwrap();
        write(temp.path().join(".git").join("h").join("mod.rs"), "x").unwrap();

        write(temp.path().join(".gitignore"), "target/\n").unwrap();
        create_dir(temp.path().join("target")).unwrap();
        create_dir(temp.path().join("target").join("dep")).unwrap();
        write(temp.path().join("target").join("dep").join("mod.rs"), "x").unwrap();

        create_dir(temp.path().join("src")).unwrap();
        let src_foo = temp.path().join("src").join("foo");
        create_dir(&src_foo).unwrap();
        write(src_foo.join("mod.rs"), "pub mod a;").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.issues[0].message.contains("foo"));
    }

    #[test]
    fn test_issue_message() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("handlers");
        create_dir(&subdir).unwrap();
        write(subdir.join("mod.rs"), "").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert!(result.issues[0].message.contains("handlers.rs"));
        assert!(result.issues[0].message.contains("handlers/mod.rs"));
    }

    #[test]
    fn test_suggested_path() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("core");
        create_dir(&subdir).unwrap();
        write(subdir.join("mod.rs"), "").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.issues[0].suggested, temp.path().join("core.rs"));
    }

    #[test]
    fn test_nested_mod_rs() {
        let temp = TempDir::new().unwrap();
        let level1 = temp.path().join("level1");
        let level2 = level1.join("level2");
        create_dir(&level1).unwrap();
        create_dir(&level2).unwrap();
        write(level2.join("mod.rs"), "// nested").unwrap();

        let result = find_mod_rs_issues(temp.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.issues[0].message.contains("level2"));
        assert_eq!(result.issues[0].suggested, level1.join("level2.rs"));
    }

    #[test]
    fn test_single_file_check() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("single");
        create_dir(&subdir).unwrap();
        let mod_rs = subdir.join("mod.rs");
        write(&mod_rs, "").unwrap();

        let result = find_mod_rs_issues(mod_rs.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_non_mod_rs_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("lib.rs");
        write(&file, "fn main() {}").unwrap();

        let result = find_mod_rs_issues(file.to_str().unwrap()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_line_column() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("pos");
        create_dir(&subdir).unwrap();
        write(subdir.join("mod.rs"), "").unwrap();

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
