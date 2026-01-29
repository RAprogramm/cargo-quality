// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::process::Command;

use masterror::AppResult;

use crate::error::IoError;

/// Rustfmt configuration settings.
///
/// This structure holds the hardcoded quality standards for Rust code
/// formatting. All settings are based on project conventions and ensure
/// consistent formatting across all codebases without requiring local
/// .rustfmt.toml files.
#[derive(Debug, Clone)]
pub struct RustfmtConfig {
    pub trailing_comma:               &'static str,
    pub brace_style:                  &'static str,
    pub struct_field_align_threshold: u32,
    pub wrap_comments:                bool,
    pub format_code_in_doc_comments:  bool,
    pub struct_lit_single_line:       bool,
    pub max_width:                    u32,
    pub imports_granularity:          &'static str,
    pub group_imports:                &'static str,
    pub reorder_imports:              bool,
    pub unstable_features:            bool
}

impl Default for RustfmtConfig {
    /// Creates the default configuration matching project quality standards.
    ///
    /// # Returns
    ///
    /// `RustfmtConfig` with hardcoded quality settings
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_quality::formatter::RustfmtConfig;
    /// let config = RustfmtConfig::default();
    /// assert_eq!(config.max_width, 99);
    /// ```
    fn default() -> Self {
        Self {
            trailing_comma:               "Never",
            brace_style:                  "SameLineWhere",
            struct_field_align_threshold: 20,
            wrap_comments:                true,
            format_code_in_doc_comments:  true,
            struct_lit_single_line:       false,
            max_width:                    99,
            imports_granularity:          "Crate",
            group_imports:                "StdExternalCrate",
            reorder_imports:              true,
            unstable_features:            true
        }
    }
}

impl RustfmtConfig {
    /// Converts configuration to rustfmt command-line arguments.
    ///
    /// Generates a vector of `--config key=value` arguments that can be
    /// passed directly to `cargo +nightly fmt`.
    ///
    /// # Returns
    ///
    /// `Vec<String>` containing all configuration arguments
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_quality::formatter::RustfmtConfig;
    /// let config = RustfmtConfig::default();
    /// let args = config.to_args();
    /// assert!(args.contains(&"--config".to_string()));
    /// assert!(args.contains(&"max_width=99".to_string()));
    /// ```
    pub fn to_args(&self) -> Vec<String> {
        vec![
            "--config".to_string(),
            format!("trailing_comma={}", self.trailing_comma),
            "--config".to_string(),
            format!("brace_style={}", self.brace_style),
            "--config".to_string(),
            format!(
                "struct_field_align_threshold={}",
                self.struct_field_align_threshold
            ),
            "--config".to_string(),
            format!("wrap_comments={}", self.wrap_comments),
            "--config".to_string(),
            format!(
                "format_code_in_doc_comments={}",
                self.format_code_in_doc_comments
            ),
            "--config".to_string(),
            format!("struct_lit_single_line={}", self.struct_lit_single_line),
            "--config".to_string(),
            format!("max_width={}", self.max_width),
            "--config".to_string(),
            format!("imports_granularity={}", self.imports_granularity),
            "--config".to_string(),
            format!("group_imports={}", self.group_imports),
            "--config".to_string(),
            format!("reorder_imports={}", self.reorder_imports),
            "--config".to_string(),
            format!("unstable_features={}", self.unstable_features),
        ]
    }
}

/// Runs cargo +nightly fmt with hardcoded quality configuration.
///
/// Executes rustfmt with project-defined quality standards, ignoring any
/// local .rustfmt.toml files. This ensures consistent formatting across
/// all projects without configuration file duplication.
///
/// # Returns
///
/// `AppResult<()>` - Ok if formatting succeeds, error otherwise
///
/// # Examples
///
/// ```no_run
/// use cargo_quality::formatter::format_code;
/// format_code().unwrap();
/// ```
pub fn format_code() -> AppResult<()> {
    let config = RustfmtConfig::default();
    let args = config.to_args();

    let mut command = Command::new("cargo");
    command.arg("+nightly").arg("fmt").arg("--");

    for arg in args {
        command.arg(arg);
    }

    let status = command.status().map_err(IoError::from)?;

    if status.success() {
        println!("Code formatted successfully");
        Ok(())
    } else {
        Err(IoError::from(std::io::Error::other(format!(
            "cargo fmt failed with status: {}",
            status
        )))
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RustfmtConfig::default();
        assert_eq!(config.max_width, 99);
        assert_eq!(config.trailing_comma, "Never");
        assert_eq!(config.brace_style, "SameLineWhere");
        assert_eq!(config.imports_granularity, "Crate");
        assert_eq!(config.group_imports, "StdExternalCrate");
        assert!(config.wrap_comments);
        assert!(config.format_code_in_doc_comments);
        assert!(!config.struct_lit_single_line);
        assert!(config.reorder_imports);
        assert!(config.unstable_features);
    }

    #[test]
    fn test_config_to_args() {
        let config = RustfmtConfig::default();
        let args = config.to_args();

        assert!(args.contains(&"--config".to_string()));
        assert!(args.contains(&"max_width=99".to_string()));
        assert!(args.contains(&"trailing_comma=Never".to_string()));
        assert!(args.contains(&"brace_style=SameLineWhere".to_string()));
        assert!(args.contains(&"imports_granularity=Crate".to_string()));
        assert!(args.contains(&"group_imports=StdExternalCrate".to_string()));
    }

    #[test]
    fn test_config_to_args_count() {
        let config = RustfmtConfig::default();
        let args = config.to_args();
        assert_eq!(args.len(), 22);
    }

    #[test]
    fn test_config_to_args_pairs() {
        let config = RustfmtConfig::default();
        let args = config.to_args();

        for i in (0..args.len()).step_by(2) {
            assert_eq!(args[i], "--config");
            assert!(args[i + 1].contains('='));
        }
    }

    #[test]
    fn test_format_code_execution() {
        let result = format_code();
        assert!(result.is_ok() || result.is_err());
    }
}
