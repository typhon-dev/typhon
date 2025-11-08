// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-runtime/src/errors.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! Error handling for the Typhon runtime.

use std::fmt;

/// Runtime error types for the Typhon VM.
#[derive(Debug)]
pub enum RuntimeError {
    /// Error when a value has an unexpected type.
    TypeError {
        expected: String,
        found: String,
        message: String,
    },
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
        RuntimeError::TypeError {
            expected: expected.into(),
            found: found.into(),
            message: message.into(),
        }
    }

    /// Create a new index error.
    pub fn index_error(message: impl Into<String>) -> Self {
        RuntimeError::IndexError {
            message: message.into(),
        }
    }

    /// Create a new key error.
    pub fn key_error(message: impl Into<String>) -> Self {
        RuntimeError::KeyError {
            message: message.into(),
        }
    }

    /// Create a new I/O error.
    pub fn io_error(message: impl Into<String>) -> Self {
        RuntimeError::IOError {
            message: message.into(),
        }
    }

    /// Create a new value error.
    pub fn value_error(message: impl Into<String>) -> Self {
        RuntimeError::ValueError {
            message: message.into(),
        }
    }

    /// Create a new name error.
    pub fn name_error(name: impl Into<String>) -> Self {
        RuntimeError::NameError { name: name.into() }
    }

    /// Create a new generic runtime error.
    pub fn generic(message: impl Into<String>) -> Self {
        RuntimeError::Generic {
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeError {
                expected,
                found,
                message,
            } => {
                write!(
                    f,
                    "TypeError: expected {expected}, found {found}. {message}"
                )
            }
            RuntimeError::IndexError { message } => write!(f, "IndexError: {message}"),
            RuntimeError::KeyError { message } => write!(f, "KeyError: {message}"),
            RuntimeError::IOError { message } => write!(f, "IOError: {message}"),
            RuntimeError::ValueError { message } => write!(f, "ValueError: {message}"),
            RuntimeError::NameError { name } => {
                write!(f, "NameError: name '{name}' is not defined")
            }
            RuntimeError::Generic { message } => write!(f, "RuntimeError: {message}"),
        }
    }
}

impl std::error::Error for RuntimeError {}
