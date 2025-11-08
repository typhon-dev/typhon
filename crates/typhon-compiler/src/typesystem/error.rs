//! Error types and utilities for the Typhon type system.
//!
//! This module defines the error types used in the type checking process,
//! including type errors, error reporting, and error formatting.

use std::fmt;

use crate::frontend::ast::SourceInfo;
use crate::frontend::lexer::token::TokenSpan;
use crate::typesystem::types::Type;

/// A type error.
#[derive(Debug, Clone)]
pub struct TypeError {
    /// The kind of error.
    pub kind: TypeErrorKind,
    /// Source information for the error.
    pub source_info: Option<SourceInfo>,
}

impl TypeError {
    /// Creates a new type error.
    pub fn new(kind: TypeErrorKind, source_info: Option<SourceInfo>) -> Self {
        Self { kind, source_info }
    }

    /// Returns the error message.
    pub fn message(&self) -> String {
        self.kind.message()
    }

    /// Returns the source span for the error, if available.
    pub fn span(&self) -> Option<TokenSpan> {
        self.source_info.map(|info| info.span)
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(source_info) = self.source_info {
            write!(f, "Type error at {:?}: {}", source_info.span, self.kind)
        } else {
            write!(f, "Type error: {}", self.kind)
        }
    }
}

impl std::error::Error for TypeError {}

/// The kind of type error.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeErrorKind {
    /// Type mismatch.
    TypeMismatch {
        /// Expected type.
        expected: String,
        /// Actual type.
        actual: String,
    },
    /// Undefined variable.
    UndefinedVariable {
        /// Variable name.
        name: String,
    },
    /// Undefined attribute.
    UndefinedAttribute {
        /// Base type.
        base: String,
        /// Attribute name.
        name: String,
    },
    /// Not callable.
    NotCallable {
        /// Type that was attempted to be called.
        ty: String,
    },
    /// Incorrect number of arguments.
    IncorrectArgumentCount {
        /// Expected number of arguments.
        expected: usize,
        /// Actual number of arguments.
        actual: usize,
    },
    /// Undefined type.
    UndefinedType {
        /// Type name.
        name: String,
    },
    /// Recursive type definition.
    RecursiveType {
        /// Type name.
        name: String,
    },
    /// Invalid type for operation.
    InvalidOperationType {
        /// Operation name.
        operation: String,
        /// Invalid type.
        ty: String,
    },
    /// Invalid operand types for binary operation.
    InvalidBinaryOperandTypes {
        /// Operation name.
        operation: String,
        /// Left operand type.
        left: String,
        /// Right operand type.
        right: String,
    },
    /// Invalid operand type for unary operation.
    InvalidUnaryOperandType {
        /// Operation name.
        operation: String,
        /// Operand type.
        ty: String,
    },
    /// Invalid assignment target.
    InvalidAssignmentTarget {
        /// Target type.
        ty: String,
    },
    /// Invalid return type.
    InvalidReturnType {
        /// Expected type.
        expected: String,
        /// Actual type.
        actual: String,
    },
    /// Missing return statement.
    MissingReturn {
        /// Function name.
        function: String,
        /// Expected return type.
        expected: String,
    },
    /// Invalid type annotation.
    InvalidTypeAnnotation {
        /// Variable name.
        name: String,
        /// Annotation.
        annotation: String,
    },
    /// Circular inheritance.
    CircularInheritance {
        /// Class name.
        name: String,
    },
    /// Generic error.
    Generic {
        /// Error message.
        message: String,
    },
}

impl TypeErrorKind {
    /// Returns the error message for the error kind.
    pub fn message(&self) -> String {
        match self {
            TypeErrorKind::TypeMismatch { expected, actual } => {
                format!("Type mismatch: expected {}, got {}", expected, actual)
            }
            TypeErrorKind::UndefinedVariable { name } => {
                format!("Undefined variable: '{}'", name)
            }
            TypeErrorKind::UndefinedAttribute { base, name } => {
                format!("Undefined attribute '{}' for type {}", name, base)
            }
            TypeErrorKind::NotCallable { ty } => {
                format!("Type {} is not callable", ty)
            }
            TypeErrorKind::IncorrectArgumentCount { expected, actual } => {
                format!(
                    "Incorrect argument count: expected {}, got {}",
                    expected, actual
                )
            }
            TypeErrorKind::UndefinedType { name } => {
                format!("Undefined type: '{}'", name)
            }
            TypeErrorKind::RecursiveType { name } => {
                format!("Recursive type definition: '{}'", name)
            }
            TypeErrorKind::InvalidOperationType { operation, ty } => {
                format!("Invalid type for operation '{}': {}", operation, ty)
            }
            TypeErrorKind::InvalidBinaryOperandTypes {
                operation,
                left,
                right,
            } => {
                format!(
                    "Invalid operand types for binary operation '{}': {} and {}",
                    operation, left, right
                )
            }
            TypeErrorKind::InvalidUnaryOperandType { operation, ty } => {
                format!(
                    "Invalid operand type for unary operation '{}': {}",
                    operation, ty
                )
            }
            TypeErrorKind::InvalidAssignmentTarget { ty } => {
                format!("Invalid assignment target: {}", ty)
            }
            TypeErrorKind::InvalidReturnType { expected, actual } => {
                format!("Invalid return type: expected {}, got {}", expected, actual)
            }
            TypeErrorKind::MissingReturn { function, expected } => {
                format!(
                    "Missing return statement in function '{}' with return type {}",
                    function, expected
                )
            }
            TypeErrorKind::InvalidTypeAnnotation { name, annotation } => {
                format!("Invalid type annotation for '{}': {}", name, annotation)
            }
            TypeErrorKind::CircularInheritance { name } => {
                format!("Circular inheritance in class '{}'", name)
            }
            TypeErrorKind::Generic { message } => message.clone(),
        }
    }
}

/// A type error report, containing multiple errors.
#[derive(Debug, Clone)]
pub struct TypeErrorReport {
    /// The errors in the report.
    pub errors: Vec<TypeError>,
}

impl TypeErrorReport {
    /// Creates a new type error report.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Adds an error to the report.
    pub fn add_error(&mut self, error: TypeError) {
        self.errors.push(error);
    }

    /// Returns whether the report has errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the number of errors in the report.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Formats the error report as a string.
    pub fn format(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            "Type error report ({} errors):\n",
            self.error_count()
        ));

        for (i, error) in self.errors.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, error));
        }

        result
    }
}

impl Default for TypeErrorReport {
    fn default() -> Self {
        Self::new()
    }
}
