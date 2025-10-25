use std::io;

use masterror::AppError;

/// IO operation failed
#[derive(Debug)]
pub struct IoError {
    source: io::Error
}

impl From<IoError> for AppError {
    fn from(err: IoError) -> Self {
        AppError::internal(format!("IO error: {}", err.source))
    }
}

/// Syntax parsing failed
#[derive(Debug)]
pub struct ParseError {
    source: syn::Error
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self {
        AppError::bad_request(format!("Parse error: {}", err.source))
    }
}

/// Invalid configuration
#[derive(Debug)]
pub struct InvalidConfigError {
    message: String
}

impl From<InvalidConfigError> for AppError {
    fn from(err: InvalidConfigError) -> Self {
        AppError::bad_request(format!("Invalid configuration: {}", err.message))
    }
}

/// File not found
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
        Self { source }
    }
}

impl From<syn::Error> for ParseError {
    fn from(source: syn::Error) -> Self {
        Self { source }
    }
}

impl InvalidConfigError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl FileNotFoundError {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}
