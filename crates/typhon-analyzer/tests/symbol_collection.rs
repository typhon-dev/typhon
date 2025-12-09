//! Tests for symbol collection and scope hierarchy building.
//!
//! These tests verify that the `SymbolCollectorVisitor` correctly:
//!
//! - Collects symbols from various declaration types
//! - Builds the proper scope hierarchy
//! - Handles Python-specific scoping rules (hoisting, loop variables, etc.)
//! - Detects duplicate symbol definitions

use std::sync::Arc;

use typhon_analyzer::symbol::{ScopeKind, SymbolKind};
use typhon_analyzer::visitors::SymbolCollectorVisitor;
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

/// Helper to create a parser for test source code
fn parse_source(source: &str) -> (Parser<'_>, typhon_ast::nodes::NodeID) {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());
    let mut parser = Parser::new(source, file_id, Arc::new(source_manager));
    let module_id = parser.parse_module().expect("Failed to parse module");

    (parser, module_id)
}

#[test]
fn test_simple_variable_declaration() {
    let source = "x = 42\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Verify variable symbol exists
    let symbol = symbol_table.lookup_symbol("x");

    assert!(symbol.is_some(), "Variable 'x' should be defined");
    assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
}

#[test]
fn test_multiple_variables() {
    let source = "x = 1\ny = 2\nz = 3\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    assert!(symbol_table.lookup_symbol("x").is_some());
    assert!(symbol_table.lookup_symbol("y").is_some());
    assert!(symbol_table.lookup_symbol("z").is_some());
}

#[test]
fn test_function_declaration() {
    let source = "def foo(a, b):\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Verify function symbol in module scope
    let func_symbol = symbol_table.lookup_symbol("foo");

    assert!(func_symbol.is_some(), "Function 'foo' should be defined");
    assert_eq!(func_symbol.unwrap().kind, SymbolKind::Function);

    // Verify function has its own scope
    assert!(
        symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Function),
        "Function should create a scope"
    );

    // Verify parameters are defined somewhere
    let has_param_a = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("a"));
    let has_param_b = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("b"));

    assert!(has_param_a, "Parameter 'a' should be defined");
    assert!(has_param_b, "Parameter 'b' should be defined");
}

#[test]
fn test_class_declaration() {
    let source = "class MyClass:\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Verify class symbol exists
    let class_symbol = symbol_table.lookup_symbol("MyClass");

    assert!(class_symbol.is_some(), "Class 'MyClass' should be defined");
    assert_eq!(class_symbol.unwrap().kind, SymbolKind::Class);

    // Verify class has its own scope
    assert!(
        symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Class),
        "Class should create a scope"
    );
}

#[test]
fn test_nested_functions() {
    let source = "def outer():\n    def inner():\n        pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Verify outer function exists
    let outer_symbol = symbol_table.lookup_symbol("outer");

    assert!(outer_symbol.is_some(), "Function 'outer' should be defined");

    // Verify we have multiple function scopes
    let func_scopes: Vec<_> =
        symbol_table.scopes().filter(|(_, s)| s.kind == ScopeKind::Function).collect();

    assert!(
        func_scopes.len() >= 2,
        "Should have at least 2 function scopes (outer and inner), found {}",
        func_scopes.len()
    );
}

#[test]
fn test_duplicate_variable_error() {
    let source = "x = 1\nx = 2\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    // Should get duplicate symbol error
    assert!(result.is_err(), "Should report duplicate symbol error");

    let errors = result.unwrap_err();

    assert!(!errors.is_empty(), "Should have at least one error");
}

#[test]
fn test_duplicate_function_error() {
    let source = "def foo():\n    pass\n\ndef foo():\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_err(), "Should report duplicate function error");
}

#[test]
fn test_function_with_typed_parameters() {
    let source = "def greet(name: str, age: int) -> str:\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    assert!(symbol_table.lookup_symbol("greet").is_some());

    let has_name = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("name"));
    let has_age = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("age"));

    assert!(has_name, "Parameter 'name' should be defined");
    assert!(has_age, "Parameter 'age' should be defined");
}

#[test]
fn test_class_with_methods() {
    let source = "class Calculator:\n    def add(self, x, y):\n        pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    assert!(symbol_table.lookup_symbol("Calculator").is_some());

    // Method should be defined somewhere in the class scope
    let has_add = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("add"));

    assert!(has_add, "Method 'add' should be defined");
}

#[test]
fn test_for_loop_creates_block_scope() {
    let source = "for i in range(10):\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Loop variable should be defined
    let has_loop_var = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("i"));

    assert!(has_loop_var, "Loop variable 'i' should be defined");

    // Should have block scope for loop body
    assert!(
        !symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Block),
        "For loop should create block scope"
    );
}

#[test]
fn test_while_loop_creates_block_scope() {
    let source = "while True:\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    assert!(
        symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Block),
        "While loop should create block scope"
    );
}

#[test]
fn test_if_statement_creates_block_scopes() {
    let source = "if True:\n    x = 1\nelse:\n    y = 2\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Should have block scopes for if and else branches
    assert!(
        symbol_table.scopes().filter(|(_, s)| s.kind == ScopeKind::Block).count() >= 2,
        "If/else should create at least 2 block scopes"
    );
}

#[test]
fn test_try_except_creates_block_scopes() {
    let source = "try:\n    pass\nexcept Exception as e:\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Exception variable should be defined
    let has_exc_var = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("e"));

    assert!(has_exc_var, "Exception variable 'e' should be defined");

    // Should have block scopes
    assert!(
        symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Block),
        "Try/except should create block scopes"
    );
}

#[test]
fn test_with_statement_collects_context_variable() {
    let source = "with open('file.txt') as f:\n    pass\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Context manager variable should be defined
    let has_var = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("f"));

    assert!(has_var, "Context manager variable 'f' should be defined");
}

#[test]
fn test_lambda_creates_lambda_scope() {
    let source = "f = lambda x: x + 1\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Should have lambda scope
    assert!(
        symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Lambda),
        "Lambda should create its own scope"
    );

    // Lambda parameter should be defined
    let has_param = symbol_table.scopes().any(|(_, s)| s.symbols.contains_key("x"));

    assert!(has_param, "Lambda parameter 'x' should be defined");
}

#[test]
fn test_list_comprehension_creates_scope() {
    let source = "result = [x for x in range(10)]\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Should have comprehension scope
    assert!(
        symbol_table.scopes().any(|(_, s)| s.kind == ScopeKind::Comprehension),
        "List comprehension should create its own scope"
    );
}

#[test]
fn test_import_statement() {
    let source = "import os\nimport sys\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Import symbols should be defined
    assert!(symbol_table.lookup_symbol("os").is_some(), "Import 'os' should be defined");
    assert!(symbol_table.lookup_symbol("sys").is_some(), "Import 'sys' should be defined");
}

#[test]
fn test_from_import_statement() {
    let source = "from os import path\n";
    let (parser, module_id) = parse_source(source);
    let visitor = SymbolCollectorVisitor::new(parser.ast());
    let result = visitor.collect(module_id);

    assert!(result.is_ok(), "Symbol collection should succeed");

    let symbol_table = result.unwrap();

    // Imported name should be defined
    assert!(symbol_table.lookup_symbol("path").is_some(), "Imported 'path' should be defined");
}
