// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Pre-rendered file diff block with calculated dimensions.
///
/// Stores all output lines for a single file along with the maximum visual
/// width required for display. Used by grid layout system to calculate optimal
/// column arrangements.
///
/// # Memory Layout
///
/// Lines are stored with ANSI color codes embedded. Width represents the
/// visual character count excluding escape sequences.
///
/// # Examples
///
/// ```
/// use cargo_quality::differ::display::types::RenderedFile;
///
/// let rendered = RenderedFile {
///     lines: vec![
///         "File: test.rs".to_string(),
///         "────────────".to_string(),
///     ],
///     width: 40,
/// };
///
/// assert_eq!(rendered.line_count(), 2);
/// assert_eq!(rendered.width, 40);
/// ```
#[derive(Debug, Clone)]
pub struct RenderedFile {
    /// Output lines with embedded ANSI color codes
    pub lines: Vec<String>,
    /// Maximum visual width in characters (excluding ANSI codes)
    pub width: usize
}

impl RenderedFile {
    /// Returns the number of lines in the rendered output.
    ///
    /// # Returns
    ///
    /// Line count
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_quality::differ::display::types::RenderedFile;
    ///
    /// let mut rendered = RenderedFile {
    ///     lines: vec!["line1".to_string(), "line2".to_string()],
    ///     width: 40
    /// };
    ///
    /// assert_eq!(rendered.line_count(), 2);
    /// ```
    #[inline]
    pub const fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_count() {
        let mut rendered = RenderedFile {
            lines: Vec::new(),
            width: 40
        };
        assert_eq!(rendered.line_count(), 0);

        rendered.lines.push("line1".to_string());
        assert_eq!(rendered.line_count(), 1);

        rendered.lines.push("line2".to_string());
        assert_eq!(rendered.line_count(), 2);
    }

    #[test]
    fn test_clone() {
        let original = RenderedFile {
            lines: vec!["test".to_string()],
            width: 50
        };

        let cloned = original.clone();
        assert_eq!(cloned.line_count(), 1);
        assert_eq!(cloned.width, 50);
    }
}
