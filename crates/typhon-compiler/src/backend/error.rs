// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/error.rs
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
//! Error types for code generation.

use std::error::Error;
use std::fmt;

use crate::common::SourceInfo;

/// Error type for code generation errors.
#[derive(Debug)]
pub enum CodeGenError {
    /// Error during LLVM setup
    LLVMSetupError(String),

    /// Error during type conversion
    TypeConversionError {
        /// Description of the error
        message: String,
        /// Source location
        source_info: Option<SourceInfo>,
    },

    /// Error during code generation
    CodeGenError {
        /// Description of the error
        message: String,
        /// Source location
        source_info: Option<SourceInfo>,
    },

    /// Unsupported language feature
    UnsupportedFeature {
        /// Description of the feature
        feature: String,
        /// Source location
        source_info: Option<SourceInfo>,
    },
}

impl CodeGenError {
    /// Creates a new LLVM setup error.
    pub fn llvm_setup_error(message: impl Into<String>) -> Self {
        CodeGenError::LLVMSetupError(message.into())
    }

    /// Creates a new type conversion error.
    pub fn type_conversion_error(
        message: impl Into<String>,
        source_info: Option<SourceInfo>,
    ) -> Self {
        CodeGenError::TypeConversionError {
            message: message.into(),
            source_info,
        }
    }

    /// Creates a new code generation error.
    pub fn code_gen_error(message: impl Into<String>, source_info: Option<SourceInfo>) -> Self {
        CodeGenError::CodeGenError {
            message: message.into(),
            source_info,
        }
    }

    /// Creates a new unsupported feature error.
    pub fn unsupported_feature(
        feature: impl Into<String>,
        source_info: Option<SourceInfo>,
    ) -> Self {
        CodeGenError::UnsupportedFeature {
            feature: feature.into(),
            source_info,
        }
    }
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::LLVMSetupError(message) => {
                write!(f, "LLVM setup error: {message}")
            }
            CodeGenError::TypeConversionError {
                message,
                source_info,
            } => {
                write!(f, "Type conversion error: {message}")?;
                if let Some(source_info) = source_info {
                    write!(f, " at {}:{}", source_info.line, source_info.column)?;
                }
                Ok(())
            }
            CodeGenError::CodeGenError {
                message,
                source_info,
            } => {
                write!(f, "Code generation error: {message}")?;
                if let Some(source_info) = source_info {
                    write!(f, " at {}:{}", source_info.line, source_info.column)?;
                }
                Ok(())
            }
            CodeGenError::UnsupportedFeature {
                feature,
                source_info,
            } => {
                write!(f, "Unsupported feature: {feature}")?;
                if let Some(source_info) = source_info {
                    write!(f, " at {}:{}", source_info.line, source_info.column)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for CodeGenError {}

/// Result type for code generation operations.
pub type CodeGenResult<T> = Result<T, CodeGenError>;
