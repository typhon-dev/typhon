//! Lowering error types
//!
//! This module defines error types that can occur during AST to MIR lowering.

use thiserror::Error;
use typhon_ast::nodes::NodeID;
use typhon_source::types::Span;

/// Result type for lowering operations
pub type LoweringResult<T> = Result<T, LoweringError>;

/// Errors that can occur during AST to MIR lowering
#[derive(Debug, Error, Clone)]
pub enum LoweringError {
    /// Break statement outside of loop
    #[error("break statement outside of loop")]
    BreakOutsideLoop { span: Span },
    /// Continue statement outside of loop
    #[error("continue statement outside of loop")]
    ContinueOutsideLoop { span: Span },
    /// Internal compiler error during lowering
    #[error("internal compiler error: {message}")]
    InternalError { message: String, span: Span },
    /// Invalid assignment target
    #[error("invalid assignment target")]
    InvalidAssignmentTarget { span: Span },
    /// Missing symbol information from semantic analysis
    #[error("missing symbol information for node {node_id:?}")]
    MissingSymbolInfo { node_id: NodeID, span: Span },
    /// Missing type information from semantic analysis
    #[error("missing type information for node {node_id:?}")]
    MissingTypeInfo { node_id: NodeID, span: Span },
    /// Return statement without value in non-void function
    #[error("return statement without value in non-void function")]
    ReturnWithoutValueInNonVoidFunction { span: Span },
    /// Return statement with value in void function
    #[error("return statement with value in void function")]
    ReturnWithValueInVoidFunction { span: Span },
    /// Type conversion failed
    #[error("type conversion error: cannot convert from {from} to {to}: {reason}")]
    TypeConversionError { from: String, to: String, reason: String, span: Span },
    /// Type information not available when needed
    #[error("type information unavailable: {message}")]
    TypeInformationUnavailable { message: String, span: Span },
    /// Type lookup failed for expression
    #[error("type lookup failed for expression {expr_id}: {reason}")]
    TypeLookupFailed { expr_id: usize, reason: String, span: Span },
    /// Type mismatch during lowering
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String, span: Span },
    /// Unsupported AST node during lowering
    #[error("unsupported AST node: {node_kind}")]
    UnsupportedNode { node_kind: String, span: Span },
}
