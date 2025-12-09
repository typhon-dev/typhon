//! Tests for module-level parsing.

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
// Empty Module Tests
// ============================================================================

#[test]
fn test_empty_module() {
    let mut parser = create_parser("");
    let module_id = parser.parse_module().expect("Failed to parse empty module");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_only_whitespace() {
    let mut parser = create_parser("\n\n\n");
    let module_id = parser.parse_module().expect("Failed to parse whitespace-only module");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Import Statement Tests
// ============================================================================

#[test]
fn test_module_with_import() {
    let source = "import os\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with import");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_from_import() {
    let source = "from os import path\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with from import");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_multiple_imports() {
    let source = "import os\nimport sys\nfrom pathlib import Path\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with multiple imports");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Function Definition Tests
// ============================================================================

#[test]
fn test_module_with_function() {
    let source = "def hello():\n    print('Hello, world!')\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with function");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_multiple_functions() {
    let source = "def foo():\n    pass\n\ndef bar():\n    pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with multiple functions");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Class Definition Tests
// ============================================================================

#[test]
fn test_module_with_class() {
    let source = "class MyClass:\n    pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with class");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_multiple_classes() {
    let source = "class Foo:\n    pass\n\nclass Bar:\n    pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with multiple classes");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Variable Declaration Tests
// ============================================================================

#[test]
fn test_module_with_variable() {
    let source = "x = 42\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with variable");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_typed_variable() {
    let source = "x: int = 42\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with typed variable");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_constants() {
    let source = "MAX_SIZE = 100\nMIN_SIZE = 10\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with constants");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Type Definition Tests
// ============================================================================

#[test]
fn test_module_with_type_alias() {
    let source = "type Vector = list[float]\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with type alias");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Mixed Content Tests
// ============================================================================

#[test]
fn test_module_with_mixed_content() {
    let source =
        "import os\n\nMAX_SIZE = 100\n\ndef process():\n    pass\n\nclass Handler:\n    pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with mixed content");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_docstring() {
    let source = "\"\"\"Module docstring.\"\"\"\n\ndef foo():\n    pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with docstring");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Complex Module Tests
// ============================================================================

#[test]
fn test_module_with_nested_definitions() {
    let source = "class Outer:\n    class Inner:\n        def method(self):\n            pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with nested definitions");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_decorators() {
    let source = "@decorator\ndef foo():\n    pass\n\n@dataclass\nclass Bar:\n    pass\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with decorators");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

#[test]
fn test_module_with_if_name_main() {
    let source = "def main():\n    pass\n\nif __name__ == '__main__':\n    main()\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with if __name__");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Comment Tests
// ============================================================================

#[test]
fn test_module_with_comments() {
    let source = "# This is a comment\nx = 42  # inline comment\n# Another comment\n";
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse module with comments");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}

// ============================================================================
// Real-World Module Tests
// ============================================================================

#[test]
fn test_complete_module() {
    let source = r#"
"""A complete module example."""

import sys
from typing import List, Optional

# Constants
MAX_RETRIES = 3
DEFAULT_TIMEOUT = 30

type ConnectionPool = list[Connection]

class Connection:
    """Database connection."""

    def __init__(self, url: str):
        self.url = url

    def connect(self) -> bool:
        """Connect to database."""
        return True

def create_connection(url: str) -> Optional[Connection]:
    """Create a new connection."""
    try:
        conn = Connection(url)
        if conn.connect():
            return conn
    except Exception:
        return None

if __name__ == '__main__':
    conn = create_connection(sys.argv[1])
"#;
    let mut parser = create_parser(source);
    let module_id = parser.parse_module().expect("Failed to parse complete module");
    let node = parser.ast().get_node(module_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Module);
    assert!(matches!(node.data, AnyNode::Module(_)));
}
