//! Error types for code generation.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

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

    /// Undefined variable error
    UndefinedVariable {
        /// Name of the variable
        name: String,
        /// Source location
        source_info: Option<SourceInfo>,
    },

    /// Immutable assignment error
    ImmutableAssignment {
        /// Name of the variable
        name: String,
        /// Source location
        source_info: Option<SourceInfo>,
    },

    /// Type mismatch error
    TypeMismatch {
        /// Expected type
        expected: String,
        /// Actual type found
        found: String,
        /// Source location
        source_info: Option<SourceInfo>,
    },

    /// Unsupported operation error
    UnsupportedOperation {
        /// Operation name
        op: String,
        /// Type the operation was attempted on
        ty: String,
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

    /// Creates an error for an undefined variable.
    pub fn undefined_variable(name: &str, source_info: Option<SourceInfo>) -> Self {
        CodeGenError::UndefinedVariable {
            name: name.to_string(),
            source_info,
        }
    }

    /// Creates an error for assignment to an immutable variable.
    pub fn immutable_assignment(name: &str, source_info: Option<SourceInfo>) -> Self {
        CodeGenError::ImmutableAssignment {
            name: name.to_string(),
            source_info,
        }
    }

    /// Creates an error for type mismatch.
    pub fn type_mismatch(expected: &str, found: &str, source_info: Option<SourceInfo>) -> Self {
        CodeGenError::TypeMismatch {
            expected: expected.to_string(),
            found: found.to_string(),
            source_info,
        }
    }

    /// Creates an error for an unsupported operation on a specific type.
    pub fn unsupported_operation(op: &str, ty: &str, source_info: Option<SourceInfo>) -> Self {
        CodeGenError::UnsupportedOperation {
            op: op.to_string(),
            ty: ty.to_string(),
            source_info,
        }
    }
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
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
            CodeGenError::UndefinedVariable { name, source_info } => {
                write!(f, "Undefined variable: {name}")?;
                if let Some(source_info) = source_info {
                    write!(f, " at {}:{}", source_info.line, source_info.column)?;
                }
                Ok(())
            }
            CodeGenError::ImmutableAssignment { name, source_info } => {
                write!(f, "Cannot assign to immutable variable: {name}")?;
                if let Some(source_info) = source_info {
                    write!(f, " at {}:{}", source_info.line, source_info.column)?;
                }
                Ok(())
            }
            CodeGenError::TypeMismatch {
                expected,
                found,
                source_info,
            } => {
                write!(f, "Type mismatch: expected {expected}, found {found}")?;
                if let Some(source_info) = source_info {
                    write!(f, " at {}:{}", source_info.line, source_info.column)?;
                }
                Ok(())
            }
            CodeGenError::UnsupportedOperation {
                op,
                ty,
                source_info,
            } => {
                write!(f, "Unsupported {op} operation for {ty}")?;
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
