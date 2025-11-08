//! Error handling functionality for the Typhon language.

use std::fmt;

/// A custom error type for Typhon programs.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

/// Different kinds of errors that can occur in Typhon programs.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error that occurs during runtime.
    Runtime,
    /// An error related to type checking.
    Type,
    /// An error related to I/O operations.
    IO,
    /// A value error (e.g., invalid argument).
    Value,
    /// An index error (e.g., out of bounds).
    Index,
    /// A key error (e.g., missing dictionary key).
    Key,
    /// A custom error type.
    Custom,
}

impl Error {
    /// Create a new runtime error with the given message.
    pub fn runtime(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: message.into(),
        }
    }

    /// Create a new type error with the given message.
    pub fn type_error(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Type,
            message: message.into(),
        }
    }

    /// Create a new I/O error with the given message.
    pub fn io(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::IO,
            message: message.into(),
        }
    }

    /// Create a new value error with the given message.
    pub fn value(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Value,
            message: message.into(),
        }
    }

    /// Create a new index error with the given message.
    pub fn index(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Index,
            message: message.into(),
        }
    }

    /// Create a new key error with the given message.
    pub fn key(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Key,
            message: message.into(),
        }
    }

    /// Create a new custom error with the given message.
    pub fn custom(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Custom,
            message: message.into(),
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the error kind.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}
