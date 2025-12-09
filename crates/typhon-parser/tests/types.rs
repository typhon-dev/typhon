//! Tests for type annotation parsing.

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
// Simple Type Tests
// ============================================================================

#[test]
fn test_simple_type() {
    let source = "x: int\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse simple type annotation");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_string_type() {
    let source = "name: str\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse str type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

// ============================================================================
// Generic Type Tests
// ============================================================================

#[test]
fn test_list_type() {
    let source = "items: list[int]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse list type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_dict_type() {
    let source = "mapping: dict[str, int]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse dict type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_nested_generic_type() {
    let source = "matrix: list[list[float]]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse nested generic type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

// ============================================================================
// Union Type Tests
// ============================================================================

#[test]
fn test_union_type() {
    let source = "value: int | str\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse union type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_optional_type() {
    let source = "value: int | None\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse optional type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_multi_union_type() {
    let source = "value: int | str | float | None\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse multi-union type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

// ============================================================================
// Tuple Type Tests
// ============================================================================

#[test]
fn test_tuple_type() {
    let source = "coord: tuple[int, int]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse tuple type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_variable_length_tuple_type() {
    let source = "values: tuple[int, ...]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse variable length tuple type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

// ============================================================================
// Callable Type Tests
// ============================================================================

#[test]
fn test_callable_type() {
    let source = "func: Callable[[int, str], bool]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse callable type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_callable_no_args() {
    let source = "func: Callable[[], None]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse callable with no args");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

// ============================================================================
// Literal Type Tests
// ============================================================================

#[test]
fn test_literal_type() {
    let source = "mode: Literal[\"read\", \"write\"]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse literal type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

// ============================================================================
// Function Type Annotation Tests
// ============================================================================

#[test]
fn test_function_return_type() {
    let source = "def add(a: int, b: int) -> int:\n    return a + b\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with return type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_no_return() {
    let source = "def print_msg(msg: str) -> None:\n    print(msg)\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with None return");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_complex_types() {
    let source = "def process(data: list[dict[str, int]]) -> tuple[int, str]:\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with complex types");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

// ============================================================================
// Complex Type Tests
// ============================================================================

#[test]
fn test_generic_union_type() {
    let source = "value: list[int | str]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse generic union type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_nested_callable_type() {
    let source = "factory: Callable[[], Callable[[int], str]]\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse nested callable type");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}
