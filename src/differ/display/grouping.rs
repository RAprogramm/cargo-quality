// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::collections::{BTreeMap, HashSet};

/// Groups and deduplicates import statements by root module.
///
/// Analyzes a list of import statements and performs intelligent grouping:
/// - Removes duplicates
/// - Groups imports by root module (std, syn, etc.)
/// - Extracts common path segments
/// - Formats as compact use statements
///
/// # Algorithm
///
/// 1. Deduplicate imports using HashSet
/// 2. Parse each import into root module and path
/// 3. Group by root module using BTreeMap (for sorted output)
/// 4. Find common prefixes within each group
/// 5. Format as `use root::path::{items}` or `use root::path::item`
///
/// # Arguments
///
/// * `imports` - Slice of import statements (e.g., "use std::fs::read;")
///
/// # Returns
///
/// Vector of grouped and formatted import statements
///
/// # Performance
///
/// - O(n log n) for deduplication and grouping
/// - Pre-allocates result capacity based on unique count
/// - Minimizes string allocations by reusing buffers
///
/// # Examples
///
/// ```
/// use cargo_quality::differ::display::grouping::group_imports;
///
/// let imports = vec![
///     "use std::fs::write;",
///     "use std::fs::write;", // duplicate
///     "use std::io::read;",
/// ];
///
/// let grouped = group_imports(&imports);
/// assert_eq!(grouped.len(), 1);
/// assert!(grouped[0].contains("std::{"));
/// ```
///
/// ```
/// use cargo_quality::differ::display::grouping::group_imports;
///
/// let imports = vec!["use syn::visit::visit_file;", "use syn::visit::visit_expr;"];
///
/// let grouped = group_imports(&imports);
/// assert_eq!(grouped.len(), 1);
/// assert!(grouped[0].contains("syn::visit::{"));
/// ```
pub fn group_imports(imports: &[&str]) -> Vec<String> {
    if imports.is_empty() {
        return Vec::new();
    }

    let unique: HashSet<&str> = imports.iter().copied().collect();
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for import in unique {
        let import_str = import
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .trim();

        if let Some(double_colon_pos) = import_str.find("::") {
            let root = import_str[..double_colon_pos].to_string();
            let path = import_str[double_colon_pos + 2..].to_string();
            grouped.entry(root).or_default().push(path);
        } else {
            grouped
                .entry(import_str.to_string())
                .or_default()
                .push(String::new());
        }
    }

    let mut result = Vec::with_capacity(grouped.len());

    for (root, mut paths) in grouped {
        if paths.len() == 1 && !paths[0].is_empty() {
            result.push(format!("use {}::{};", root, paths[0]));
        } else if paths.len() == 1 && paths[0].is_empty() {
            result.push(format!("use {};", root));
        } else {
            paths.sort_unstable();

            let common_prefix = find_common_prefix(&paths);

            if !common_prefix.is_empty() {
                let prefix_with_sep = if paths[0].starts_with(&format!("{}::", common_prefix)) {
                    format!("{}::", common_prefix)
                } else {
                    common_prefix.clone()
                };

                let suffixes: Vec<String> = paths
                    .iter()
                    .map(|p| {
                        p.strip_prefix(&prefix_with_sep)
                            .unwrap_or(p.as_str())
                            .to_string()
                    })
                    .collect();

                if prefix_with_sep.ends_with("::") {
                    result.push(format!(
                        "use {}::{}::{{{}}};",
                        root,
                        common_prefix,
                        suffixes.join(", ")
                    ));
                } else {
                    result.push(format!("use {}::{{{}}};", root, suffixes.join(", ")));
                }
            } else {
                result.push(format!("use {}::{{{}}};", root, paths.join(", ")));
            }
        }
    }

    result
}

/// Finds longest common prefix among import paths.
///
/// Uses component-wise comparison to find shared segments in import paths.
/// Returns the longest common prefix that appears in all paths.
///
/// # Algorithm
///
/// 1. Split each path by "::" into components
/// 2. Find minimum component count across all paths
/// 3. Compare components at each position
/// 4. Stop at first mismatch
/// 5. Join common components back with "::"
///
/// # Arguments
///
/// * `paths` - Slice of import paths (without "use" keyword)
///
/// # Returns
///
/// Common prefix string, or empty if no common prefix
///
/// # Performance
///
/// - O(n × m) where n is path count, m is average component count
/// - Early termination on first mismatch
/// - Pre-allocates component vectors
///
/// # Examples
///
/// ```
/// use cargo_quality::differ::display::grouping::find_common_prefix;
///
/// let paths = vec![
///     "visit::visit_file".to_string(),
///     "visit::visit_expr".to_string(),
/// ];
///
/// let prefix = find_common_prefix(&paths);
/// assert_eq!(prefix, "visit");
/// ```
///
/// ```
/// use cargo_quality::differ::display::grouping::find_common_prefix;
///
/// let paths = vec!["fs::read".to_string(), "io::write".to_string()];
///
/// let prefix = find_common_prefix(&paths);
/// assert!(prefix.is_empty());
/// ```
pub fn find_common_prefix(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }

    if paths.len() == 1 {
        return String::new();
    }

    let parts: Vec<Vec<&str>> = paths.iter().map(|p| p.split("::").collect()).collect();

    let min_len = parts.iter().map(|p| p.len()).min().unwrap_or(0);

    let mut common = Vec::with_capacity(min_len);

    for i in 0..min_len {
        let first = parts[0][i];
        if parts.iter().all(|p| p[i] == first) {
            common.push(first);
        } else {
            break;
        }
    }

    if !common.is_empty() {
        common.join("::")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_imports_deduplication() {
        let imports = vec!["use std::fs::write;", "use std::fs::write;"];
        let result = group_imports(&imports);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_group_imports_same_root() {
        let imports = vec!["use std::fs::write;", "use std::io::read;"];
        let result = group_imports(&imports);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("std::{"));
    }

    #[test]
    fn test_group_imports_common_path() {
        let imports = vec!["use syn::visit::visit_file;", "use syn::visit::visit_expr;"];
        let result = group_imports(&imports);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("syn::visit::{"));
    }

    #[test]
    fn test_group_imports_empty() {
        let imports: Vec<&str> = vec![];
        let result = group_imports(&imports);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_common_prefix_same() {
        let paths = vec![
            "visit::visit_file".to_string(),
            "visit::visit_expr".to_string(),
        ];
        let prefix = find_common_prefix(&paths);
        assert_eq!(prefix, "visit");
    }

    #[test]
    fn test_find_common_prefix_different() {
        let paths = vec!["fs::read".to_string(), "io::write".to_string()];
        let prefix = find_common_prefix(&paths);
        assert!(prefix.is_empty());
    }

    #[test]
    fn test_find_common_prefix_empty() {
        let paths: Vec<String> = vec![];
        let prefix = find_common_prefix(&paths);
        assert!(prefix.is_empty());
    }

    #[test]
    fn test_find_common_prefix_single() {
        let paths = vec!["test::path".to_string()];
        let prefix = find_common_prefix(&paths);
        assert!(prefix.is_empty());
    }

    #[test]
    fn test_group_imports_multiple_roots() {
        let imports = vec!["use std::fs::write;", "use syn::visit::visit_file;"];
        let result = group_imports(&imports);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_group_imports_single_item() {
        let imports = vec!["use std::fs::write;"];
        let result = group_imports(&imports);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "use std::fs::write;");
    }

    #[test]
    fn test_find_common_prefix_partial() {
        let paths = vec![
            "a::b::c".to_string(),
            "a::b::d".to_string(),
            "a::b::e".to_string(),
        ];
        let prefix = find_common_prefix(&paths);
        assert_eq!(prefix, "a::b");
    }
}
