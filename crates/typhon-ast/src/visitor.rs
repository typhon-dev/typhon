//! ## AST visitor pattern implementation
//!
//! This module provides a comprehensive visitor pattern for traversing and analyzing the AST.
//! The implementation is designed for flexibility, type safety, and performance.
//!
//! ## Architecture
//!
//! The visitor pattern follows an arena-based approach:
//! 1. The AST owns all nodes in a contiguous arena
//! 2. Visitors receive `NodeID` references rather than direct pointers
//! 3. Generation counters prevent use-after-free bugs
//!
//! This design avoids trait object overhead and simplifies lifetime management.
//!
//! ## Core Traits
//!
//! ### `Visitable`
//!
//! Implemented by all AST node types, enabling double dispatch to appropriate
//! visitor methods based on runtime type.
//!
//! ### `Visitor<T>`
//!
//! Immutable visitor with generic return type `T`. Provides specialized visit
//! methods for each node type, all returning `VisitorResult<T>` for error handling.
//!
//! ### `MutVisitor<T>`
//!
//! Mutable visitor that can modify visitor state during traversal. Mirrors the
//! `Visitor<T>` interface but takes `&mut self`.
//!
//! ## Helper Methods
//!
//! Both visitor traits provide convenience methods:
//! - `try_visit()` - Visit with Result error handling
//! - `try_visit_opt()` - Visit optional nodes
//! - `visit_list()` - Visit multiple nodes, collecting results
//! - `visit_list_stop_on_error()` - Visit until first error
//!
//! ## Usage Examples
//!
//! ### Basic Visitor
//!
//! ```ignore
//! use typhon_ast::node::NodeID;
//! use typhon_ast::visitor::{Visitor, VisitorResult};
//!
//! struct TypeChecker {
//!     errors: Vec<String>,
//! }
//!
//! impl Visitor<String> for TypeChecker {
//!     fn visit_binary_op(&mut self, node_id: NodeID) -> VisitorResult<String> {
//!         Ok("int".to_string())
//!     }
//! }
//! ```
//!
//! ### Mutable Visitor with State
//!
//! ```ignore
//! use typhon_ast::visitor::{MutVisitor, VisitorResult};
//!
//! struct SymbolTable {
//!     symbols: HashMap<String, Type>,
//! }
//!
//! impl MutVisitor<()> for SymbolTable {
//!     fn visit_variable_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
//!         // Add symbol to table
//!         Ok(())
//!     }
//! }
//! ```

use std::fmt::{self, Display, Formatter};

use crate::nodes::NodeID;

/// Trait for AST nodes that can be visited
///
/// This trait defines the interface for nodes that can be visited by a visitor.
/// It includes an `accept<T>` method that dispatches to the appropriate visitor method
/// based on the node's type.
pub trait Visitable {
    /// Accept a visitor and dispatch to the appropriate visit method
    ///
    /// ## Arguments
    ///
    /// * `visitor` - A mutable reference to a visitor that implements the `Visitor<T>` trait
    /// * `node_id` - The ID of the node to visit
    ///
    /// ## Returns
    ///
    /// A result containing the value returned by the visitor, or an error if the visit failed
    ///
    /// ## Errors
    ///
    /// This may return a `VisitorError` if:
    /// - `NodeNotFound`: The node with the given ID does not exist in the AST arena
    /// - `TypeMismatch`: The node exists but is not of the expected type for this visitor method
    /// - `Custom`: A custom error occurred during the visit operation, with details in the error message
    fn accept<T>(&self, visitor: &mut dyn Visitor<T>, node_id: NodeID) -> VisitorResult<T>;

    /// Accept a mutable visitor and dispatch to the appropriate visit method
    ///
    /// ## Arguments
    ///
    /// * `visitor` - A mutable reference to a visitor that implements the `MutVisitor<T>` trait
    /// * `node_id` - The ID of the node to visit
    ///
    /// ## Returns
    ///
    /// A result containing the value returned by the visitor, or an error if the visit failed
    ///
    /// ## Errors
    ///
    /// This may return a `VisitorError` if:
    /// - `NodeNotFound`: The node with the given ID does not exist in the AST arena
    /// - `TypeMismatch`: The node exists but is not of the expected type for this visitor method
    /// - `Custom`: A custom error occurred during the visit operation, with details in the error message
    fn accept_mut<T>(&self, visitor: &mut dyn MutVisitor<T>, node_id: NodeID) -> VisitorResult<T>;
}

/// Macro to generate all visit_* methods with the same pattern.
///
/// Each method will:
/// 1. Take a `NodeID` parameter
/// 2. Return a `VisitorResult`<T>
/// 3. Have a default implementation that returns an error with a formatted message
macro_rules! visit_default {
    ($($(#[$meta:meta])* $method:ident),*$(,)?) => {
        $(
            /// Visits a node of the specified type
            ///
            /// ## Errors
            ///
            /// This may return an error if:
            /// - The node with the given ID does not exist in the AST arena
            /// - The node with the given ID is not a node of the specified type
            /// - The visitor implementation does not handle nodes of the specified type
            $(#[$meta])*
            fn $method(&mut self, node_id: NodeID) -> VisitorResult<T> {
                Err(VisitorError::Custom(format!(
                    concat!(stringify!($method), " not implemented for node {}"), node_id,
                )))
            }
        )*
    };
}

/// Generic Visitor trait for AST nodes
///
/// This trait defines the interface for visitors that traverse the AST.
/// It includes a generic `visit` method that takes a `NodeID` and returns an optional value of type `T`.
/// The generic method dispatches to specialized methods based on the node type.
pub trait Visitor<T> {
    /// Helper method to try visiting a node
    ///
    /// This method attempts to visit a node and returns an error result if the visit fails.
    ///
    /// ## Arguments
    ///
    /// * `node_id` - The ID of the node to visit
    ///
    /// ## Returns
    ///
    /// A result containing the value returned by the visitor, or an error if the visit failed
    ///
    /// ## Errors
    ///
    /// This may return a `VisitorError` if:
    /// - `NodeNotFound`: The node with the given ID does not exist in the AST arena
    /// - `TypeMismatch`: The node exists but is not of the expected type for this visitor method
    /// - `Custom`: A custom error occurred during the visit operation, with details in the error message
    fn try_visit(&mut self, node_id: NodeID) -> VisitorResult<T> {
        self.visit(node_id)
            .ok_or_else(|| VisitorError::Custom(format!("Failed to visit node {node_id}")))
    }

    /// Helper method to try visiting an optional node
    ///
    /// This method attempts to visit an optional node and returns Ok(None) if the node is None.
    ///
    /// ## Arguments
    ///
    /// * `node_id_opt` - An optional `NodeID` to visit
    ///
    /// ## Returns
    ///
    /// A result containing the optional value returned by the visitor,
    /// Ok(None) if the node is None, or an error if the visit failed
    ///
    /// ## Errors
    ///
    /// This may return a `VisitorError` if:
    /// - `NodeNotFound`: The node with the given ID does not exist in the AST arena
    /// - `TypeMismatch`: The node exists but is not of the expected type for this visitor method
    /// - `Custom`: A custom error occurred during the visit operation, with details in the error message
    fn try_visit_opt(&mut self, node_id_opt: Option<NodeID>) -> VisitorResult<Option<T>> {
        node_id_opt.map_or_else(|| Ok(None), |node_id| self.try_visit(node_id).map(Some))
    }

    /// Helper method to visit a list of nodes and collect results
    ///
    /// This method visits all nodes in the list and collects their results into a Vec.
    /// If any visit fails, returns an error immediately (fail-fast behavior).
    ///
    /// ## Arguments
    ///
    /// * `node_ids` - A slice of `NodeID`s to visit
    ///
    /// ## Returns
    ///
    /// A result containing a vector of results, or the first error encountered
    fn visit_list(&mut self, node_ids: &[NodeID]) -> VisitorResult<Vec<T>> {
        node_ids.iter().map(|&id| self.try_visit(id)).collect()
    }

    /// Visits an AST node
    ///
    /// This is the generic entry point for visiting any node.
    fn visit(&mut self, node_id: NodeID) -> Option<T>;

    visit_default!(
        visit_argument_expr,
        visit_as_pattern,
        visit_assert_stmt,
        visit_assignment_expr,
        visit_assignment_stmt,
        visit_async_for_stmt,
        visit_async_function_decl,
        visit_async_with_stmt,
        visit_attribute_expr,
        visit_augmented_assignment_stmt,
        visit_await_expr,
        visit_basic_ident,
        visit_binary_op_expr,
        visit_break_stmt,
        visit_call_expr,
        visit_callable_type,
        visit_class_decl,
        visit_class_pattern,
        visit_continue_stmt,
        visit_delete_stmt,
        visit_dict_comprehension_expr,
        visit_dict_expr,
        visit_except_handler,
        visit_expression_stmt,
        visit_fmt_string_expr,
        visit_for_stmt,
        visit_from_import_stmt,
        visit_function_decl,
        visit_generator_expr,
        visit_generic_type,
        visit_global_stmt,
        visit_grouping_expr,
        visit_identifier_pattern,
        visit_if_stmt,
        visit_import_stmt,
        visit_lambda_expr,
        visit_list_comprehension_expr,
        visit_list_expr,
        visit_literal_expr,
        visit_literal_pattern,
        visit_literal_type,
        visit_mapping_pattern,
        visit_match_case,
        visit_match_stmt,
        visit_module,
        visit_nonlocal_stmt,
        visit_or_pattern,
        visit_parameter_ident,
        visit_pass_stmt,
        visit_raise_stmt,
        visit_return_stmt,
        visit_sequence_pattern,
        visit_set_comprehension_expr,
        visit_set_expr,
        visit_slice_expr,
        visit_starred_expr,
        visit_subscription_expr,
        visit_template_string_expr,
        visit_ternary_expr,
        visit_try_stmt,
        visit_tuple_expr,
        visit_tuple_type,
        visit_type_decl,
        visit_unary_op_expr,
        visit_union_type,
        visit_variable_decl,
        visit_variable_expr,
        visit_while_stmt,
        visit_wildcard_pattern,
        visit_with_stmt,
        visit_yield_expr,
        visit_yield_from_expr,
    );
}

/// Mutable Visitor trait for AST nodes
///
/// This trait is identical to `Visitor<T>` but takes `&mut self`,
/// allowing the visitor to maintain and modify mutable state during traversal.
/// Useful for collecting information, building symbol tables, or performing
/// transformations that require stateful tracking.
pub trait MutVisitor<T> {
    /// Helper method to try visiting a node
    fn try_visit(&mut self, node_id: NodeID) -> VisitorResult<T> {
        self.visit(node_id)
            .ok_or_else(|| VisitorError::Custom(format!("Failed to visit node {node_id}")))
    }

    /// Helper method to try visiting an optional node
    fn try_visit_opt(&mut self, node_id_opt: Option<NodeID>) -> VisitorResult<Option<T>> {
        node_id_opt.map_or_else(|| Ok(None), |node_id| self.try_visit(node_id).map(Some))
    }

    /// Helper method to visit a list of nodes and collect results
    fn visit_list(&mut self, node_ids: &[NodeID]) -> VisitorResult<Vec<T>> {
        node_ids.iter().map(|&id| self.try_visit(id)).collect()
    }

    /// Visits an AST node
    fn visit(&mut self, node_id: NodeID) -> Option<T>;

    visit_default!(
        visit_argument_expr,
        visit_as_pattern,
        visit_assert_stmt,
        visit_assignment_expr,
        visit_assignment_stmt,
        visit_async_for_stmt,
        visit_async_function_decl,
        visit_async_with_stmt,
        visit_attribute_expr,
        visit_augmented_assignment_stmt,
        visit_await_expr,
        visit_basic_ident,
        visit_binary_op_expr,
        visit_break_stmt,
        visit_call_expr,
        visit_callable_type,
        visit_class_decl,
        visit_class_pattern,
        visit_continue_stmt,
        visit_delete_stmt,
        visit_dict_comprehension_expr,
        visit_dict_expr,
        visit_except_handler,
        visit_expression_stmt,
        visit_fmt_string_expr,
        visit_for_stmt,
        visit_from_import_stmt,
        visit_function_decl,
        visit_generator_expr,
        visit_generic_type,
        visit_global_stmt,
        visit_grouping_expr,
        visit_identifier_pattern,
        visit_if_stmt,
        visit_import_stmt,
        visit_lambda_expr,
        visit_list_comprehension_expr,
        visit_list_expr,
        visit_literal_expr,
        visit_literal_pattern,
        visit_literal_type,
        visit_mapping_pattern,
        visit_match_case,
        visit_match_stmt,
        visit_module,
        visit_nonlocal_stmt,
        visit_or_pattern,
        visit_parameter_ident,
        visit_pass_stmt,
        visit_raise_stmt,
        visit_return_stmt,
        visit_sequence_pattern,
        visit_set_comprehension_expr,
        visit_set_expr,
        visit_slice_expr,
        visit_starred_expr,
        visit_subscription_expr,
        visit_template_string_expr,
        visit_ternary_expr,
        visit_try_stmt,
        visit_tuple_expr,
        visit_tuple_type,
        visit_type_decl,
        visit_unary_op_expr,
        visit_union_type,
        visit_variable_decl,
        visit_variable_expr,
        visit_while_stmt,
        visit_wildcard_pattern,
        visit_with_stmt,
        visit_yield_expr,
        visit_yield_from_expr,
    );
}

/// Error type for visitor operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisitorError {
    /// Node not found in the AST
    NodeNotFound(NodeID),
    /// Node type mismatch
    TypeMismatch {
        /// The node ID that caused the mismatch
        node_id: NodeID,
        /// Expected node kind
        expected: String,
        /// Actual node kind
        actual: String,
    },
    /// Custom error with message
    Custom(String),
}

impl Display for VisitorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeNotFound(id) => write!(f, "Node not found: {id}"),
            Self::TypeMismatch { node_id, expected, actual } => {
                write!(f, "Type mismatch for node {node_id}: expected {expected}, got {actual}")
            }
            Self::Custom(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for VisitorError {}

/// Result type for visitor operations
pub type VisitorResult<T> = Result<T, VisitorError>;
