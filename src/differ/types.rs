// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Represents a single code change.
///
/// Stores the location and content of a proposed modification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    pub line:        usize,
    pub analyzer:    String,
    pub original:    String,
    pub modified:    String,
    pub description: String,
    pub import:      Option<String>
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
    #[inline]
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
    #[inline]
    pub fn add_entry(&mut self, entry: DiffEntry) {
        self.entries.push(entry);
    }

    /// Returns total number of changes.
    ///
    /// # Returns
    ///
    /// Number of diff entries
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn total_changes(&self) -> usize {
        self.files.iter().map(|f| f.total_changes()).sum()
    }

    /// Returns number of files with changes.
    ///
    /// # Returns
    ///
    /// File count
    #[inline]
    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
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
            description: "desc".to_string(),
            import:      None
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
            description: "desc".to_string(),
            import:      None
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
            description: "desc".to_string(),
            import:      None
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
    fn test_diff_result_multiple_files() {
        let mut result = DiffResult::new();

        let mut file1 = FileDiff::new("file1.rs".to_string());
        file1.add_entry(DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        });

        let mut file2 = FileDiff::new("file2.rs".to_string());
        file2.add_entry(DiffEntry {
            line:        1,
            analyzer:    "test".to_string(),
            original:    "old".to_string(),
            modified:    "new".to_string(),
            description: "desc".to_string(),
            import:      None
        });

        result.add_file(file1);
        result.add_file(file2);

        assert_eq!(result.total_files(), 2);
        assert_eq!(result.total_changes(), 2);
    }
}
