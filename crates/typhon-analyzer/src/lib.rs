//! Semantic analysis for the Typhon programming language.
//!
//! This crate provides semantic analysis capabilities for the Typhon compiler, including:
//!
//! - Symbol table construction and management
//! - Name resolution across scopes
//! - Type checking and type inference
//! - Semantic validation and error reporting
//!
//! ## Architecture
//!
//! The semantic analysis phase consists of several key components:
//!
//! - **Symbol Management** ([`symbol`]): Manages symbols, scopes, and name resolution
//! - **Type System** ([`types`]): Type representation, checking, and inference
//! - **Error Handling** ([`error`]): Semantic error types and reporting
//! - **Context** ([`context`]): Main semantic analysis context
//!
//! ## Example
//!
//! ```rust,ignore
//! use typhon_analyzer::context::SemanticContext;
//! use typhon_ast::ast::AST;
//! use std::sync::Arc;
//!
//! // Create AST from parsing...
//! let ast = AST::new();
//! let mut context = SemanticContext::new(Arc::new(ast));
//!
//! // Perform semantic analysis
//! if let Err(errors) = context.analyze(root_node) {
//!     for error in errors {
//!         eprintln!("Semantic error: {}", error);
//!     }
//! }
//! ```

pub mod analysis;
pub mod context;
pub mod error;
pub mod symbol;
pub mod types;
pub mod visitors;

use context::SemanticContext;
use error::SemanticError;
use typhon_ast::ast::AST;
use typhon_ast::nodes::NodeID;

/// Analyzes a module and returns a semantic context with collected symbols and resolved names.
///
/// This is a convenience function that creates a semantic context and performs
/// the complete semantic analysis pipeline:
///
/// 1. Symbol collection
/// 2. Name resolution
/// 3. Type checking
/// 4. Semantic validation
///
/// ## Errors
///
/// Returns semantic errors if any were encountered during analysis.
pub fn analyze_module(ast: &AST, module_id: NodeID) -> Result<SemanticContext, Vec<SemanticError>> {
    let mut context = SemanticContext::new();
    context.collect_symbols(ast, module_id)?;
    context.resolve_names(ast, module_id)?;
    context.check_types(ast, module_id)?;
    context.validate_semantics(ast, module_id)?;

    Ok(context)
}
