// Copyright (c) 2024 The Typhon Project
// SPDX-License-Identifier: Apache-2.0
//! Error types for code generation.

use std::error::Error;
use std::fmt;

use crate::frontend::lexer::token::TokenSpan;

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
        span: Option<TokenSpan>,
    },

    /// Error during code generation
    CodeGenError {
        /// Description of the error
        message: String,
        /// Source location
        span: Option<TokenSpan>,
    },

    /// Unsupported language feature
    UnsupportedFeature {
        /// Description of the feature
        feature: String,
        /// Source location
        span: Option<TokenSpan>,
    },
}

impl CodeGenError {
    /// Creates a new LLVM setup error.
    pub fn llvm_setup_error(message: impl Into<String>) -> Self {
        CodeGenError::LLVMSetupError(message.into())
    }

    /// Creates a new type conversion error.
    pub fn type_conversion_error(message: impl Into<String>, span: Option<TokenSpan>) -> Self {
        CodeGenError::TypeConversionError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new code generation error.
    pub fn code_gen_error(message: impl Into<String>, span: Option<TokenSpan>) -> Self {
        CodeGenError::CodeGenError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new unsupported feature error.
    pub fn unsupported_feature(feature: impl Into<String>, span: Option<TokenSpan>) -> Self {
        CodeGenError::UnsupportedFeature {
            feature: feature.into(),
            span,
        }
    }
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::LLVMSetupError(message) => {
                write!(f, "LLVM setup error: {}", message)
            }
            CodeGenError::TypeConversionError { message, span } => {
                write!(f, "Type conversion error: {}", message)?;
                if let Some(span) = span {
                    write!(f, " at {}:{}", span.line, span.column)?;
                }
                Ok(())
            }
            CodeGenError::CodeGenError { message, span } => {
                write!(f, "Code generation error: {}", message)?;
                if let Some(span) = span {
                    write!(f, " at {}:{}", span.line, span.column)?;
                }
                Ok(())
            }
            CodeGenError::UnsupportedFeature { feature, span } => {
                write!(f, "Unsupported feature: {}", feature)?;
                if let Some(span) = span {
                    write!(f, " at {}:{}", span.line, span.column)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for CodeGenError {}

/// Result type for code generation operations.
pub type CodeGenResult<T> = Result<T, CodeGenError>;
