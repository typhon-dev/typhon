//! Tests for declaration parsing.

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
// Function Declaration Tests
// ============================================================================

#[test]
fn test_simple_function() {
    let source = "def foo():\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse simple function");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_parameters() {
    let source = "def add(a, b):\n    return a + b\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with parameters");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_default_params() {
    let source = "def greet(name, greeting='Hello'):\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with default params");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_type_hints() {
    let source = "def add(a: int, b: int) -> int:\n    return a + b\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with type hints");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_args_kwargs() {
    let source = "def func(*args, **kwargs):\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with *args/**kwargs");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_kwonly_params() {
    let source = "def func(a, *, b, c=None):\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with kwonly params");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_decorators() {
    let source = "@decorator\ndef foo():\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with decorator");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

#[test]
fn test_function_with_multiple_decorators() {
    let source = "@decorator1\n@decorator2\ndef foo():\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id =
        parser.parse_declaration().expect("Failed to parse function with multiple decorators");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}

// ============================================================================
// Async Function Declaration Tests
// ============================================================================

#[test]
fn test_async_function() {
    let source = "async def fetch():\n    pass\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse async function");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::AsyncFunctionDecl(_)));
}

#[test]
fn test_async_function_with_await() {
    let source = "async def fetch():\n    result = await get_data()\n    return result\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse async function with await");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::AsyncFunctionDecl(_)));
}

// ============================================================================
// Class Declaration Tests
// ============================================================================

#[test]
fn test_simple_class() {
    let source = "class MyClass:\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse simple class");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_class_with_base() {
    let source = "class MyClass(BaseClass):\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class with base");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_class_with_multiple_bases() {
    let source = "class MyClass(Base1, Base2):\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class with multiple bases");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_class_with_methods() {
    let source = "class MyClass:\n    def method(self):\n        pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class with method");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_class_with_init() {
    let source = "class MyClass:\n    def __init__(self, value):\n        self.value = value\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class with __init__");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_class_with_decorator() {
    let source = "@dataclass\nclass MyClass:\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class with decorator");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

// ============================================================================
// Type Definition Tests
// ============================================================================

#[test]
fn test_type_alias() {
    let source = "type Vector = list[float]\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse type alias");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::TypeDecl(_)));
}

// TODO: Generic type aliases not yet supported by parser
// #[test]
// fn test_generic_type_alias() {
//     let source = "type Matrix[T] = list[list[T]]\n";
//     let mut parser = create_parser(source);
//     let stmt_id = parser.parse_statement().expect("Failed to parse generic type alias");
//     let node = parser.ast().get_node(stmt_id).expect("Node not found");
//
//     assert_eq!(node.kind, NodeKind::Declaration);
//     assert!(matches!(node.data, AnyNode::TypeDecl(_)));
// }

// ============================================================================
// Variable Declaration Tests
// ============================================================================

#[test]
fn test_variable_declaration_with_type() {
    let source = "x: int = 42\n";
    let mut parser = create_parser(source);
    let stmt_id = parser.parse_statement().expect("Failed to parse variable declaration");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_variable_declaration_without_value() {
    let source = "x: int\n";
    let mut parser = create_parser(source);
    let stmt_id =
        parser.parse_statement().expect("Failed to parse variable declaration without value");
    let node = parser.ast().get_node(stmt_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::VariableDecl(_)));
}

#[test]
fn test_class_variable_declaration() {
    let source = "class MyClass:\n    x: int = 42\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class variable declaration");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

// ============================================================================
// Complex Declaration Tests
// ============================================================================

#[test]
fn test_nested_class() {
    let source = "class Outer:\n    class Inner:\n        pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse nested class");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_method_with_decorator() {
    let source = "class MyClass:\n    @staticmethod\n    def method():\n        pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse method with decorator");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_property_decorator() {
    let source =
        "class MyClass:\n    @property\n    def value(self):\n        return self._value\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse property decorator");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_class_with_docstring() {
    let source = "class MyClass:\n    \"\"\"A simple class.\"\"\"\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse class with docstring");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::ClassDecl(_)));
}

#[test]
fn test_function_with_docstring() {
    let source = "def foo():\n    \"\"\"A simple function.\"\"\"\n    pass\n";
    let mut parser = create_parser(source);
    let decl_id = parser.parse_declaration().expect("Failed to parse function with docstring");
    let node = parser.ast().get_node(decl_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Declaration);
    assert!(matches!(node.data, AnyNode::FunctionDecl(_)));
}
