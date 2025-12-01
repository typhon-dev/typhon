//! Type system for the Typhon programming language.
//!
//! This module contains the implementation of Typhon's static type system, including
//! type definitions, type checking, type inference, and error handling.

/// Type definitions and utilities.
pub mod types;

/// Type checking and inference.
pub mod checker;

/// Type system errors.
pub mod error;

/// Tests for the type system.
#[cfg(test)]
mod tests;

// Re-exports for commonly used components
pub use self::checker::{
    TypeCheckResult,
    TypeChecker,
};
pub use self::error::{
    TypeError,
    TypeErrorKind,
    TypeErrorReport,
};
pub use self::types::{
    ClassType,
    FunctionType,
    GenericInstance,
    GenericParam,
    ListType,
    PrimitiveType,
    TupleType,
    Type,
    TypeCompatibility,
    TypeEnv,
    TypeId,
    TypeVar,
    UnionType,
};

/// Result type for type checking operations.
pub type Result<T> = std::result::Result<T, TypeError>;
