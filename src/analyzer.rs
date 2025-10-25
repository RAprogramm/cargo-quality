use masterror::AppResult;
use syn::File;

/// Analysis issue found in code
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    /// Line number where issue was found
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Issue description
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>
}

/// Result of code analysis
#[derive(Debug, Default)]
pub struct AnalysisResult {
    /// Issues found
    pub issues: Vec<Issue>,
    /// Number of fixable issues
    pub fixable_count: usize
}

/// Trait for code analyzers
pub trait Analyzer {
    /// Analyzer name
    fn name(&self) -> &'static str;

    /// Analyze syntax tree
    fn analyze(&self, ast: &File) -> AppResult<AnalysisResult>;

    /// Apply fixes to syntax tree
    fn fix(&self, ast: &mut File) -> AppResult<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_creation() {
        let issue = Issue {
            line: 42,
            column: 10,
            message: "Test issue".to_string(),
            suggestion: Some("Fix suggestion".to_string())
        };

        assert_eq!(issue.line, 42);
        assert_eq!(issue.column, 10);
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn test_analysis_result_default() {
        let result = AnalysisResult::default();
        assert_eq!(result.issues.len(), 0);
        assert_eq!(result.fixable_count, 0);
    }
}
