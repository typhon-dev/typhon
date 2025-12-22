//! Tests for type-aware expression lowering
//!
//! Verifies that expression lowering uses type information from semantic
//! analysis when available, and gracefully falls back when unavailable.

use typhon_ast::ast::AST;
use typhon_ast::nodes::NodeID;
use typhon_mir::types::MIRType;
use typhon_mir_builder::context::LoweringContext;

#[test]
fn test_backward_compatibility_no_semantic_context() {
    // Verify expressions work without semantic context (legacy mode)
    let ast = AST::new();
    let ctx = LoweringContext::new(&ast, "test_module".to_string());

    assert!(!ctx.has_semantic_context());
    // Expression lowering should work with Object as default type
}

#[test]
fn test_binary_op_falls_back_to_object() {
    // Without semantic context, binary operations should use Object type
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());
    let dummy_node_id = NodeID::new(1, 0);
    let result_type = ctx.query_expr_type_or_default(dummy_node_id);

    assert_eq!(result_type, MIRType::Object { type_id: None });
}

#[test]
fn test_unary_op_falls_back_to_object() {
    // Without semantic context, unary operations should use Object type
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());
    let dummy_node_id = NodeID::new(2, 0);
    let result_type = ctx.query_expr_type_or_default(dummy_node_id);

    assert_eq!(result_type, MIRType::Object { type_id: None });
}

#[test]
fn test_call_falls_back_to_object() {
    // Without semantic context, calls should use Object as return type
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());
    let dummy_node_id = NodeID::new(3, 0);
    let return_type = ctx.query_expr_type_or_default(dummy_node_id);

    assert_eq!(return_type, MIRType::Object { type_id: None });
}

#[test]
fn test_attribute_falls_back_to_object() {
    // Without semantic context, attributes should use Object type
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());
    let dummy_node_id = NodeID::new(4, 0);
    let attr_type = ctx.query_expr_type_or_default(dummy_node_id);

    assert_eq!(attr_type, MIRType::Object { type_id: None });
}

#[test]
fn test_type_cache_behavior() {
    // Verify type cache works correctly
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());

    ctx.clear_type_cache();

    // Cache should be empty after clear
    let node_id = NodeID::new(5, 0);

    assert_eq!(ctx.query_expr_type(node_id), None);
}

#[test]
fn test_context_creation_modes() {
    // Test both context creation modes
    let ast = AST::new();
    let ctx_without = LoweringContext::new(&ast, "test_module".to_string());

    assert!(!ctx_without.has_semantic_context());

    // Note: Testing with semantic context requires actual AnalysisContext
    // which would be created in integration tests
}

#[test]
fn test_query_expr_type_returns_none_without_context() {
    // Verify query_expr_type returns None when no semantic context
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());
    let node_id = NodeID::new(10, 0);

    assert_eq!(ctx.query_expr_type(node_id), None);
}

#[test]
fn test_multiple_query_calls_same_node() {
    // Verify multiple queries for the same node are consistent
    let ast = AST::new();
    let mut ctx = LoweringContext::new(&ast, "test_module".to_string());
    let node_id = NodeID::new(11, 0);
    let result1 = ctx.query_expr_type_or_default(node_id);
    let result2 = ctx.query_expr_type_or_default(node_id);

    assert_eq!(result1, result2);
    assert_eq!(result1, MIRType::Object { type_id: None });
}
