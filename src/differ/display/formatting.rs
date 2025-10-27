// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use console::measure_text_width;

/// Pads text to exact visual width with trailing spaces.
///
/// This function handles ANSI escape sequences correctly, measuring only the
/// visible character width. If the text is already wider than the target,
/// it returns unchanged without truncation.
///
/// # Arguments
///
/// * `text` - Text to pad (may contain ANSI escape codes)
/// * `width` - Target visual width in characters
///
/// # Returns
///
/// Padded string with trailing spaces to reach target width
///
/// # Performance
///
/// Uses `console::measure_text_width` for ANSI-aware measurement. Pre-allocates
/// the exact required capacity to avoid reallocations.
///
/// # Examples
///
/// ```
/// use cargo_quality::differ::display::formatting::pad_to_width;
///
/// let padded = pad_to_width("hello", 10);
/// assert_eq!(padded, "hello     ");
/// assert_eq!(padded.len(), 10);
/// ```
///
/// ```
/// use cargo_quality::differ::display::formatting::pad_to_width;
/// use console::measure_text_width;
/// use owo_colors::OwoColorize;
///
/// let colored = format!("{}", "test".red());
/// let padded = pad_to_width(&colored, 10);
/// assert_eq!(measure_text_width(&padded), 10);
/// ```
#[inline]
pub fn pad_to_width(text: &str, width: usize) -> String {
    let current = measure_text_width(text);

    if current >= width {
        return text.to_string();
    }

    let padding = width - current;
    let total_capacity = text.len() + padding;

    let mut result = String::with_capacity(total_capacity);
    result.push_str(text);
    result.push_str(&" ".repeat(padding));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_to_width_short_text() {
        let result = pad_to_width("hello", 10);
        assert_eq!(result, "hello     ");
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_pad_to_width_exact() {
        let result = pad_to_width("exactly10!", 10);
        assert_eq!(result, "exactly10!");
    }

    #[test]
    fn test_pad_to_width_already_wide() {
        let long = "this is longer than ten";
        let result = pad_to_width(long, 10);
        assert_eq!(result, long);
    }

    #[test]
    fn test_pad_to_width_empty() {
        let result = pad_to_width("", 5);
        assert_eq!(result, "     ");
    }

    #[test]
    fn test_pad_preserves_capacity() {
        let result = pad_to_width("test", 10);
        assert!(result.capacity() >= 10);
    }
}
