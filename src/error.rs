// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Error types for cargo-quality operations.
//!
//! All errors convert to `masterror::AppError` for consistent error handling.
//! Includes errors for IO operations, parsing, configuration, and file access.

use std::io;

use masterror::AppError;

/// IO operation failed.
///
/// Wraps standard IO errors that occur during file operations.
#[derive(Debug)]
pub struct IoError {
    source: io::Error
}

impl From<IoError> for AppError {
    fn from(err: IoError) -> Self {
        AppError::internal(format!("IO error: {}", err.source))
    }
}

/// Syntax parsing failed.
///
/// Wraps syn parsing errors when processing Rust source code.
#[derive(Debug)]
pub struct ParseError {
    source: syn::Error
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self {
        AppError::bad_request(format!("Parse error: {}", err.source))
    }
}

/// Invalid configuration.
///
/// Indicates configuration validation failure.
#[derive(Debug)]
pub struct InvalidConfigError {
    message: String
}

impl From<InvalidConfigError> for AppError {
    fn from(err: InvalidConfigError) -> Self {
        AppError::bad_request(format!("Invalid configuration: {}", err.message))
    }
}

/// File not found.
///
/// Indicates requested file does not exist.
#[derive(Debug)]
pub struct FileNotFoundError {
    path: String
}

impl From<FileNotFoundError> for AppError {
    fn from(err: FileNotFoundError) -> Self {
        AppError::not_found(format!("File not found: {}", err.path))
    }
}

impl From<io::Error> for IoError {
    fn from(source: io::Error) -> Self {
        Self {
            source
        }
    }
}

impl From<syn::Error> for ParseError {
    fn from(source: syn::Error) -> Self {
        Self {
            source
        }
    }
}

impl InvalidConfigError {
    /// Create new configuration error with message.
    ///
    /// # Arguments
    ///
    /// * `message` - Error description
    pub fn new(message: String) -> Self {
        Self {
            message
        }
    }
}

impl FileNotFoundError {
    /// Create new file not found error with path.
    ///
    /// # Arguments
    ///
    /// * `path` - File path that was not found
    pub fn new(path: String) -> Self {
        Self {
            path
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn test_io_error_from_std_io() {
        let std_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let io_error = IoError::from(std_err);
        let _app_error: AppError = io_error.into();
    }

    #[test]
    fn test_parse_error_from_syn() {
        let syn_err = syn::Error::new(proc_macro2::Span::call_site(), "parse failed");
        let parse_error = ParseError::from(syn_err);
        let _app_error: AppError = parse_error.into();
    }

    #[test]
    fn test_invalid_config_error_new() {
        let config_err = InvalidConfigError::new("invalid setting".to_string());
        let _app_error: AppError = config_err.into();
    }

    #[test]
    fn test_file_not_found_error_new() {
        let not_found_err = FileNotFoundError::new("/path/to/file.rs".to_string());
        let _app_error: AppError = not_found_err.into();
    }
}
