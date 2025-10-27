// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use ignore::WalkBuilder;
use masterror::AppResult;

/// Collects all Rust source files from given path.
///
/// Recursively walks through directories and finds all `.rs` files.
/// Respects .gitignore, .ignore, and other ignore files.
///
/// # Arguments
///
/// * `path` - File or directory path to search
///
/// # Returns
///
/// `AppResult<Vec<PathBuf>>` - List of Rust file paths or error
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::file_utils::collect_rust_files;
/// let files = collect_rust_files("src/").unwrap();
/// ```
#[inline]
pub fn collect_rust_files(path: &str) -> AppResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path_buf = PathBuf::from(path);

    if path_buf.is_file() && path_buf.extension().is_some_and(|e| e == "rs") {
        files.push(path_buf);
    } else if path_buf.is_dir() {
        for entry in WalkBuilder::new(path)
            .follow_links(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build()
            .flatten()
        {
            if entry.file_type().is_some_and(|ft| ft.is_file())
                && let Some(ext) = entry.path().extension()
                && ext == "rs"
            {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_collect_rust_files_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let files = collect_rust_files(file_path.to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn test_collect_rust_files_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test1.rs");
        let file2 = temp_dir.path().join("test2.rs");
        fs::write(&file1, "fn test1() {}").unwrap();
        fs::write(&file2, "fn test2() {}").unwrap();

        let files = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_collect_rust_files_non_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "not rust").unwrap();

        let result = collect_rust_files(file_path.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_collect_rust_files_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file1 = temp_dir.path().join("test1.rs");
        let file2 = subdir.join("test2.rs");
        fs::write(&file1, "fn test1() {}").unwrap();
        fs::write(&file2, "fn test2() {}").unwrap();

        let files = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_collect_rust_files_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_collect_rust_files_respects_ignore() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("included.rs");
        fs::write(&file1, "fn main() {}").unwrap();

        let ignored_dir = temp_dir.path().join("target");
        fs::create_dir(&ignored_dir).unwrap();
        let file2 = ignored_dir.join("ignored.rs");
        fs::write(&file2, "fn ignored() {}").unwrap();

        let ignore_file = temp_dir.path().join(".ignore");
        fs::write(&ignore_file, "target/\n").unwrap();

        let files = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file1);
    }

    #[test]
    fn test_collect_rust_files_respects_gitignore_in_git_repo() {
        let temp_dir = TempDir::new().unwrap();

        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let file1 = temp_dir.path().join("included.rs");
        fs::write(&file1, "fn main() {}").unwrap();

        let ignored_dir = temp_dir.path().join("target");
        fs::create_dir(&ignored_dir).unwrap();
        let file2 = ignored_dir.join("ignored.rs");
        fs::write(&file2, "fn ignored() {}").unwrap();

        let gitignore = temp_dir.path().join(".gitignore");
        fs::write(&gitignore, "target/\n").unwrap();

        let files = collect_rust_files(temp_dir.path().to_str().unwrap()).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file1);
    }
}
