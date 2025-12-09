//! Semantic error types and reporting.
//!
//! This module defines the error types that can occur during semantic analysis,
//! including undefined names, type mismatches, and scope-related errors.

use thiserror::Error;
use typhon_source::types::Span;

use crate::types::Type;

/// Semantic analysis errors.
///
/// Represents various semantic errors that can be detected during the analysis phase,
/// such as undefined variables, type mismatches, and invalid operations.
#[derive(Debug, Error, Clone)]
pub enum SemanticError {
    /// Argument error - wrong number or types of arguments in function call.
    #[error("Argument error: {message}")]
    ArgumentError {
        /// Description of the error
        message: String,
        /// The location of the function call
        span: Span,
    },

    /// Attribute error - type doesn't have the requested attribute.
    #[error("Type {type_name} has no attribute '{attribute}'")]
    AttributeError {
        /// The type being accessed
        type_name: String,
        /// The attribute name
        attribute: String,
        /// The location of the attribute access
        span: Span,
    },

    /// Break statement outside loop
    #[error("'break' statement outside loop")]
    BreakOutsideLoop {
        /// The location of the break statement
        span: Span,
    },

    /// Continue statement outside loop
    #[error("'continue' statement outside loop")]
    ContinueOutsideLoop {
        /// The location of the continue statement
        span: Span,
    },

    /// Duplicate symbol error - attempt to declare a name that already exists in the same scope.
    #[error("Duplicate symbol '{name}'")]
    DuplicateSymbol {
        /// The name that was declared twice
        name: String,
        /// The location of the original declaration
        original_span: Span,
        /// The location of the duplicate declaration
        duplicate_span: Span,
    },

    /// Invalid operator error - operator not supported for the given operand types.
    #[error("Invalid operator '{operator}' for types {left_type} and {right_type}")]
    InvalidOperator {
        /// The operator being used
        operator: String,
        /// The left operand type
        left_type: Box<Type>,
        /// The right operand type
        right_type: Box<Type>,
        /// The location of the operation
        span: Span,
    },

    /// Invalid scope error - operation performed in an invalid scope context.
    #[error("Invalid scope operation")]
    InvalidScope {
        /// Description of what went wrong
        message: String,
        /// The location of the invalid operation
        span: Span,
    },

    /// Function missing return statement
    #[error("Function '{function_name}' missing return statement in some paths")]
    MissingReturn {
        /// The function name
        function_name: String,
        /// The location of the function
        span: Span,
    },

    /// No active scope error - internal error when no scope is available.
    #[error("No active scope (internal error)")]
    NoActiveScope,

    /// Return statement outside function
    #[error("'return' statement outside function")]
    ReturnOutsideFunction {
        /// The location of the return statement
        span: Span,
    },

    /// Return type mismatch - return value doesn't match function signature.
    #[error("Return type mismatch: expected {expected}, found {found}")]
    ReturnTypeMismatch {
        /// The expected return type
        expected: Box<Type>,
        /// The actual return type found
        found: Box<Type>,
        /// The location of the return statement
        span: Span,
    },

    /// Type mismatch error - incompatible types in an operation or assignment.
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        /// The expected type
        expected: Box<Type>,
        /// The actual type found
        found: Box<Type>,
        /// The location of the type mismatch
        span: Span,
    },

    /// Undefined name error - reference to a name that hasn't been declared.
    #[error("Undefined name '{name}'")]
    UndefinedName {
        /// The name that was not found
        name: String,
        /// The location where the undefined name was used
        span: Span,
    },

    /// Unreachable code detected
    #[error("Unreachable code")]
    UnreachableCode {
        /// The location of the unreachable code
        span: Span,
    },

    /// Variable used before assignment
    #[error("Variable '{name}' used before assignment")]
    UseBeforeAssignment {
        /// The variable name
        name: String,
        /// The location of the use
        span: Span,
    },
}

impl SemanticError {
    /// Returns the span associated with this error, if any.
    #[must_use]
    pub const fn span(&self) -> Option<Span> {
        match self {
            Self::ArgumentError { span, .. }
            | Self::AttributeError { span, .. }
            | Self::BreakOutsideLoop { span, .. }
            | Self::ContinueOutsideLoop { span, .. }
            | Self::DuplicateSymbol { duplicate_span: span, .. }
            | Self::InvalidOperator { span, .. }
            | Self::InvalidScope { span, .. }
            | Self::MissingReturn { span, .. }
            | Self::ReturnOutsideFunction { span, .. }
            | Self::ReturnTypeMismatch { span, .. }
            | Self::TypeMismatch { span, .. }
            | Self::UndefinedName { span, .. }
            | Self::UnreachableCode { span, .. }
            | Self::UseBeforeAssignment { span, .. } => Some(*span),
            Self::NoActiveScope => None,
        }
    }
}
