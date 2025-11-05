// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

pub mod empty_lines;
pub mod format_args;
pub mod inline_comments;
pub mod path_import;

pub use empty_lines::EmptyLinesAnalyzer;
pub use format_args::FormatArgsAnalyzer;
pub use inline_comments::InlineCommentsAnalyzer;
pub use path_import::PathImportAnalyzer;

use crate::analyzer::Analyzer;

/// Get all available analyzers
pub fn get_analyzers() -> Vec<Box<dyn Analyzer>> {
    vec![
        Box::new(PathImportAnalyzer::new()),
        Box::new(FormatArgsAnalyzer::new()),
        Box::new(EmptyLinesAnalyzer::new()),
        Box::new(InlineCommentsAnalyzer::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_analyzers() {
        let analyzers = get_analyzers();
        assert_eq!(analyzers.len(), 4);
    }

    #[test]
    fn test_analyzer_names() {
        let analyzers = get_analyzers();
        let names: Vec<&str> = analyzers.iter().map(|a| a.name()).collect();

        assert!(names.contains(&"path_import"));
        assert!(names.contains(&"format_args"));
        assert!(names.contains(&"empty_lines"));
        assert!(names.contains(&"inline_comments"));
    }
}
