//! Tests for pattern parsing (match statement patterns).

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
// Literal Pattern Tests
// ============================================================================

#[test]
fn test_literal_int_pattern() {
    let source = "match x:\n    case 42:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with int literal");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_literal_string_pattern() {
    let source = "match x:\n    case \"hello\":\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with string literal");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Identifier Pattern Tests
// ============================================================================

#[test]
fn test_identifier_pattern() {
    let source = "match x:\n    case value:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with identifier");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Wildcard Pattern Tests
// ============================================================================

#[test]
fn test_wildcard_pattern() {
    let source = "match x:\n    case _:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with wildcard");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Sequence Pattern Tests
// ============================================================================

#[test]
fn test_list_pattern() {
    let source = "match x:\n    case [a, b, c]:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with list pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_list_pattern_with_star() {
    let source = "match x:\n    case [a, *rest, b]:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id =
        parser.parse_statement().expect("Failed to parse match with list pattern and star");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_tuple_pattern() {
    let source = "match x:\n    case (a, b):\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with tuple pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Mapping Pattern Tests
// ============================================================================

#[test]
fn test_dict_pattern() {
    let source = "match x:\n    case {\"key\": value}:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with dict pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_dict_pattern_with_rest() {
    let source = "match x:\n    case {\"key\": value, **rest}:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id =
        parser.parse_statement().expect("Failed to parse match with dict pattern and rest");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Class Pattern Tests
// ============================================================================

#[test]
fn test_class_pattern() {
    let source = "match x:\n    case Point(x=0, y=0):\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with class pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_class_pattern_positional() {
    let source = "match x:\n    case Point(0, 0):\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id =
        parser.parse_statement().expect("Failed to parse match with positional class pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// OR Pattern Tests
// ============================================================================

#[test]
fn test_or_pattern() {
    let source = "match x:\n    case 1 | 2 | 3:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with or pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// AS Pattern Tests
// ============================================================================

#[test]
fn test_as_pattern() {
    let source = "match x:\n    case [1, 2] as pair:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with as pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

// ============================================================================
// Complex Pattern Tests
// ============================================================================

#[test]
fn test_nested_pattern() {
    let source = "match x:\n    case [[a, b], [c, d]]:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with nested pattern");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_multiple_cases() {
    let source = "match x:\n    case 1:\n        pass\n    case 2:\n        pass\n    case _:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with multiple cases");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}

#[test]
fn test_pattern_with_guard() {
    let source = "match x:\n    case n if n > 0:\n        pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse match with guard");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Statement);
    assert!(matches!(node.data, AnyNode::MatchStmt(_)));
}
