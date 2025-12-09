//! Tests for statement parsing.

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
// Simple Statement Tests
// ============================================================================

#[test]
fn test_pass_statement() {
    let mut parser = create_parser("pass\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse pass statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::PassStmt(_)));
}

#[test]
fn test_break_statement() {
    let mut parser = create_parser("break\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse break statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::BreakStmt(_)));
}

#[test]
fn test_continue_statement() {
    let mut parser = create_parser("continue\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse continue statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ContinueStmt(_)));
}

// ============================================================================
// Assignment Statement Tests
// ============================================================================

#[test]
fn test_simple_assignment() {
    let mut parser = create_parser("x = 42\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse simple assignment");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssignmentStmt(_)));
}

#[test]
fn test_tuple_unpacking_assignment() {
    let mut parser = create_parser("a, b = 1, 2\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse tuple unpacking assignment");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssignmentStmt(_)));
}

#[test]
fn test_subscript_assignment() {
    let mut parser = create_parser("arr[0] = value\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse subscript assignment");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssignmentStmt(_)));
}

#[test]
fn test_dict_subscript_assignment() {
    let mut parser = create_parser("schema['key'] = 'value'\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse dict subscript assignment");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssignmentStmt(_)));
}

#[test]
fn test_augmented_assignment() {
    let mut parser = create_parser("x += 5\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse augmented assignment");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AugmentedAssignmentStmt(_)));
}

// ============================================================================
// Control Flow Statement Tests
// ============================================================================

#[test]
fn test_if_statement() {
    let source = "if x > 0:\n    pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse if statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::IfStmt(_)));
}

#[test]
fn test_if_else_statement() {
    let source = "if x > 0:\n    pass\nelse:\n    pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse if-else statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::IfStmt(_)));
}

#[test]
fn test_if_elif_else_statement() {
    let source = "if x > 0:\n    pass\nelif x < 0:\n    pass\nelse:\n    pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse if-elif-else statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::IfStmt(_)));
}

#[test]
fn test_while_statement() {
    let source = "while x > 0:\n    x -= 1\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse while statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::WhileStmt(_)));
}

#[test]
fn test_for_statement() {
    let source = "for item in items:\n    process(item)\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse for statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ForStmt(_)));
}

#[test]
fn test_async_for_statement() {
    let source = "async for item in async_items:\n    await process(item)\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse async for statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AsyncForStmt(_)));
}

// ============================================================================
// Return Statement Tests
// ============================================================================

#[test]
fn test_return_no_value() {
    let mut parser = create_parser("return\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse return with no value");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ReturnStmt(_)));
}

#[test]
fn test_return_with_value() {
    let mut parser = create_parser("return 42\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse return with value");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ReturnStmt(_)));
}

// ============================================================================
// Exception Handling Tests (Try Statement)
// ============================================================================

#[test]
fn test_try_except() {
    let source = "try:\n    risky()\nexcept Exception:\n    handle()\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse try-except");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::TryStmt(_)));
}

#[test]
fn test_try_except_as() {
    let source = "try:\n    risky()\nexcept Exception as e:\n    handle(e)\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse try-except-as");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::TryStmt(_)));
}

#[test]
fn test_try_except_finally() {
    let source = "try:\n    risky()\nexcept Exception:\n    handle()\nfinally:\n    cleanup()\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse try-except-finally");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::TryStmt(_)));
}

#[test]
fn test_try_except_else_finally() {
    let source = "try:\n    risky()\nexcept Exception:\n    handle()\nelse:\n    success()\nfinally:\n    cleanup()\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse try-except-else-finally");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::TryStmt(_)));
}

#[test]
fn test_multiple_except_handlers() {
    let source = "try:\n    risky()\nexcept ValueError:\n    handle_value()\nexcept TypeError:\n    handle_type()\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse multiple except handlers");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::TryStmt(_)));
}

// ============================================================================
// Raise Statement Tests
// ============================================================================

#[test]
fn test_raise_no_exception() {
    let mut parser = create_parser("raise\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse bare raise");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::RaiseStmt(_)));
}

#[test]
fn test_raise_exception() {
    let mut parser = create_parser("raise ValueError()\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse raise with exception");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::RaiseStmt(_)));
}

#[test]
fn test_raise_from() {
    let mut parser = create_parser("raise ValueError() from cause\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse raise from");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::RaiseStmt(_)));
}

// ============================================================================
// Assert Statement Tests
// ============================================================================

#[test]
fn test_assert_simple() {
    let mut parser = create_parser("assert True\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse simple assert");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssertStmt(_)));
}

#[test]
fn test_assert_with_message() {
    let mut parser = create_parser("assert x > 0, \"x must be positive\"\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse assert with message");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AssertStmt(_)));
}

// ============================================================================
// With Statement Tests
// ============================================================================

#[test]
fn test_with_statement() {
    let source = "with open('file.txt') as f:\n    data = f.read()\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse with statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::WithStmt(_)));
}

#[test]
fn test_async_with_statement() {
    let source = "async with async_open('file.txt') as f:\n    data = await f.read()\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse async with statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::AsyncWithStmt(_)));
}

#[test]
fn test_with_multiple_contexts() {
    let source = "with open('file1.txt') as f1, open('file2.txt') as f2:\n    pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse with multiple contexts");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::WithStmt(_)));
}

// ============================================================================
// Match Statement Tests
// ============================================================================

#[test]
fn test_match_statement() {
    let source = "match value:\n    case 1:\n        pass\n    case 2:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_match_with_wildcard() {
    let source = "match value:\n    case 1:\n        pass\n    case _:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with wildcard");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Delete Statement Tests
// ============================================================================

#[test]
fn test_delete_statement() {
    let mut parser = create_parser("del x\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse delete statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::DeleteStmt(_)));
}

#[test]
fn test_delete_multiple() {
    let mut parser = create_parser("del x, y, z\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse delete multiple");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::DeleteStmt(_)));
}

// ============================================================================
// Global and Nonlocal Statement Tests
// ============================================================================

#[test]
fn test_global_statement() {
    let mut parser = create_parser("global x\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse global statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::GlobalStmt(_)));
}

#[test]
fn test_nonlocal_statement() {
    let mut parser = create_parser("nonlocal x\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse nonlocal statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::NonlocalStmt(_)));
}

// ============================================================================
// Import Statement Tests
// ============================================================================

#[test]
fn test_import_statement() {
    let mut parser = create_parser("import os\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse import statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ImportStmt(_)));
}

#[test]
fn test_from_import_statement() {
    let mut parser = create_parser("from os import path\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse from import statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::FromImportStmt(_)));
}

#[test]
fn test_import_as() {
    let mut parser = create_parser("import numpy as np\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse import as");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ImportStmt(_)));
}

// ============================================================================
// Expression Statement Tests
// ============================================================================

#[test]
fn test_expression_statement() {
    let mut parser = create_parser("func()\n");
    let stmt_id = parser.parse_statement().expect("Failed to parse expression statement");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::ExpressionStmt(_)));
}
