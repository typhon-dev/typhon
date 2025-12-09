//! Tests for identifier parsing.

use std::sync::Arc;

use typhon_ast::nodes::{AnyNode, NodeKind};
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

fn create_parser(source: &'_ str) -> Parser<'_> {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());

    Parser::new(source, file_id, Arc::new(source_manager))
}

// ============================================================================
// Simple Identifier Tests
// ============================================================================

#[test]
fn test_simple_identifier() {
    let mut parser = create_parser("x\n");
    let expr_id = parser.parse_expression().expect("Failed to parse simple identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    // Variable references are expressions, not identifier declarations
    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

#[test]
fn test_underscore_identifier() {
    let mut parser = create_parser("_value\n");
    let expr_id = parser.parse_expression().expect("Failed to parse underscore identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

#[test]
fn test_camel_case_identifier() {
    let mut parser = create_parser("myVariable\n");
    let expr_id = parser.parse_expression().expect("Failed to parse camelCase identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

#[test]
fn test_snake_case_identifier() {
    let mut parser = create_parser("my_variable\n");
    let expr_id = parser.parse_expression().expect("Failed to parse snake_case identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

#[test]
fn test_identifier_with_numbers() {
    let mut parser = create_parser("var123\n");
    let expr_id = parser.parse_expression().expect("Failed to parse identifier with numbers");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

// ============================================================================
// Special Identifier Tests
// ============================================================================

#[test]
fn test_dunder_identifier() {
    let mut parser = create_parser("__init__\n");
    let expr_id = parser.parse_expression().expect("Failed to parse dunder identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

#[test]
fn test_private_identifier() {
    let mut parser = create_parser("_private\n");
    let expr_id = parser.parse_expression().expect("Failed to parse private identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

#[test]
fn test_mangled_identifier() {
    let mut parser = create_parser("__mangled\n");
    let expr_id = parser.parse_expression().expect("Failed to parse mangled identifier");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::VariableExpr(_)));
}

// ============================================================================
// Identifier in Context Tests
// ============================================================================

#[test]
fn test_identifier_in_assignment() {
    let mut parser = create_parser("x = 42\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse identifier in assignment");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssignmentStmt(_)));
}

#[test]
fn test_identifier_in_function_param() {
    let source = "def foo(param):\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse identifier in function param");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_identifier_in_for_loop() {
    let source = "for item in items:\n    pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse identifier in for loop");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ForStmt(_)));
}

// ============================================================================
// Class and Constant Identifier Tests
// ============================================================================

#[test]
fn test_class_name_identifier() {
    let source = "class MyClass:\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class name identifier");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_constant_identifier() {
    let mut parser = create_parser("MAX_SIZE = 100\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse constant identifier");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssignmentStmt(_)));
}

// ============================================================================
// Identifier Expression Tests
// ============================================================================

#[test]
fn test_identifier_in_binary_op() {
    let mut parser = create_parser("x + y\n");
    let expr_id = parser.parse_expression().expect("Failed to parse identifiers in binary op");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_identifier_in_call() {
    let mut parser = create_parser("func()\n");
    let expr_id = parser.parse_expression().expect("Failed to parse identifier in call");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::CallExpr(_)));
}

#[test]
fn test_identifier_with_attribute_access() {
    let mut parser = create_parser("obj.attr\n");
    let expr_id = parser.parse_expression().expect("Failed to parse identifier with attribute");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::AttributeExpr(_)));
}

#[test]
fn test_identifier_with_subscript() {
    let mut parser = create_parser("arr[0]\n");
    let expr_id = parser.parse_expression().expect("Failed to parse identifier with subscript");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::SubscriptionExpr(_)));
}
