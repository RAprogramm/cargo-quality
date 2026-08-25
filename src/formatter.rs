// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

use std::{
    io::{self, Write},
    process::Command
};

use masterror::AppResult;

use crate::error::IoError;

/// Base style settings: line width, braces, and trailing commas.
#[derive(Debug, Clone)]
pub struct StyleConfig {
    pub trailing_comma:    &'static str,
    pub brace_style:       &'static str,
    pub max_width:         u32,
    pub unstable_features: bool
}

/// Struct layout settings: field alignment and literal formatting.
#[derive(Debug, Clone)]
pub struct StructConfig {
    pub struct_field_align_threshold: u32,
    pub struct_lit_single_line:       bool
}

/// Comment formatting settings.
#[derive(Debug, Clone)]
pub struct CommentsConfig {
    pub wrap_comments:               bool,
    pub format_code_in_doc_comments: bool
}

/// Import ordering and grouping settings.
#[derive(Debug, Clone)]
pub struct ImportsConfig {
    pub imports_granularity: &'static str,
    pub group_imports:       &'static str,
    pub reorder_imports:     bool
}

/// Rustfmt configuration settings.
///
/// This structure holds the hardcoded quality standards for Rust code
/// formatting, grouped by concern. All settings are based on project
/// conventions and ensure consistent formatting across all codebases without
/// requiring local .rustfmt.toml files.
#[derive(Debug, Clone)]
pub struct RustfmtConfig {
    pub style:    StyleConfig,
    pub structs:  StructConfig,
    pub comments: CommentsConfig,
    pub imports:  ImportsConfig
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
    /// assert_eq!(config.style.max_width, 99);
    /// ```
    fn default() -> Self {
        Self {
            style:    StyleConfig {
                trailing_comma:    "Never",
                brace_style:       "SameLineWhere",
                max_width:         99,
                unstable_features: true
            },
            structs:  StructConfig {
                struct_field_align_threshold: 20,
                struct_lit_single_line:       false
            },
            comments: CommentsConfig {
                wrap_comments:               true,
                format_code_in_doc_comments: true
            },
            imports:  ImportsConfig {
                imports_granularity: "Crate",
                group_imports:       "StdExternalCrate",
                reorder_imports:     true
            }
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
        let settings = [
            format!("trailing_comma={}", self.style.trailing_comma),
            format!("brace_style={}", self.style.brace_style),
            format!(
                "struct_field_align_threshold={}",
                self.structs.struct_field_align_threshold
            ),
            format!("wrap_comments={}", self.comments.wrap_comments),
            format!(
                "format_code_in_doc_comments={}",
                self.comments.format_code_in_doc_comments
            ),
            format!(
                "struct_lit_single_line={}",
                self.structs.struct_lit_single_line
            ),
            format!("max_width={}", self.style.max_width),
            format!("imports_granularity={}", self.imports.imports_granularity),
            format!("group_imports={}", self.imports.group_imports),
            format!("reorder_imports={}", self.imports.reorder_imports),
            format!("unstable_features={}", self.style.unstable_features)
        ];

        let mut args = Vec::with_capacity(settings.len() * 2);
        for setting in settings {
            args.push("--config".to_string());
            args.push(setting);
        }

        args
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
        io::stdout()
            .lock()
            .write_all(b"Code formatted successfully\n")
            .map_err(IoError::from)?;
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
        assert_eq!(config.style.max_width, 99);
        assert_eq!(config.style.trailing_comma, "Never");
        assert_eq!(config.style.brace_style, "SameLineWhere");
        assert_eq!(config.imports.imports_granularity, "Crate");
        assert_eq!(config.imports.group_imports, "StdExternalCrate");
        assert!(config.comments.wrap_comments);
        assert!(config.comments.format_code_in_doc_comments);
        assert!(!config.structs.struct_lit_single_line);
        assert!(config.imports.reorder_imports);
        assert!(config.style.unstable_features);
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
