//! Error handling for the Typhon runtime.

use std::{error, fmt};

/// Runtime error types for the Typhon VM.
#[derive(Debug)]
pub enum RuntimeError {
    /// Error when a value has an unexpected type.
    TypeError { expected: String, found: String, message: String },
    /// Error when accessing an invalid index.
    IndexError { message: String },
    /// Error when accessing a non-existent key.
    KeyError { message: String },
    /// Error during I/O operations.
    IOError { message: String },
    /// Error when a value is invalid.
    ValueError { message: String },
    /// Error when a name is not found in scope.
    NameError { name: String },
    /// Generic runtime error.
    Generic { message: String },
}

impl RuntimeError {
    /// Create a new type error.
    pub fn type_error(
        expected: impl Into<String>,
        found: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::TypeError { expected: expected.into(), found: found.into(), message: message.into() }
    }

    /// Create a new index error.
    pub fn index_error(message: impl Into<String>) -> Self {
        Self::IndexError { message: message.into() }
    }

    /// Create a new key error.
    pub fn key_error(message: impl Into<String>) -> Self {
        Self::KeyError { message: message.into() }
    }

    /// Create a new I/O error.
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IOError { message: message.into() }
    }

    /// Create a new value error.
    pub fn value_error(message: impl Into<String>) -> Self {
        Self::ValueError { message: message.into() }
    }

    /// Create a new name error.
    pub fn name_error(name: impl Into<String>) -> Self { Self::NameError { name: name.into() } }

    /// Create a new generic runtime error.
    pub fn generic(message: impl Into<String>) -> Self { Self::Generic { message: message.into() } }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError { expected, found, message } => {
                write!(f, "TypeError: expected {expected}, found {found}. {message}")
            }
            Self::IndexError { message } => write!(f, "IndexError: {message}"),
            Self::KeyError { message } => write!(f, "KeyError: {message}"),
            Self::IOError { message } => write!(f, "IOError: {message}"),
            Self::ValueError { message } => write!(f, "ValueError: {message}"),
            Self::NameError { name } => {
                write!(f, "NameError: name '{name}' is not defined")
            }
            Self::Generic { message } => write!(f, "RuntimeError: {message}"),
        }
    }
}

impl error::Error for RuntimeError {}
