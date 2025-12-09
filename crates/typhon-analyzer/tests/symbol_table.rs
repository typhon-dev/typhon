//! Tests for symbol table functionality.

use typhon_analyzer::context::SemanticContext;
use typhon_analyzer::symbol::{ScopeKind, Symbol, SymbolKind, SymbolTable};
use typhon_ast::nodes::NodeID;
use typhon_source::types::Span;

#[test]
fn test_symbol_table_creation() {
    let table = SymbolTable::new();
    assert_eq!(table.scope_count(), 1, "Should have module scope");
    assert!(table.current_scope().is_some(), "Should have current scope");
}

#[test]
fn test_scope_creation() {
    let mut table = SymbolTable::new();
    let module_scope = table.current_scope().unwrap();

    // Create a function scope
    let func_scope = table.create_scope(ScopeKind::Function, Some(module_scope));
    assert_eq!(table.scope_count(), 2, "Should have 2 scopes");

    let scope = table.get_scope(func_scope).unwrap();
    assert_eq!(scope.kind, ScopeKind::Function);
    assert_eq!(scope.parent(), Some(module_scope));
}

#[test]
fn test_scope_hierarchy() {
    let mut table = SymbolTable::new();
    let module_scope = table.current_scope().unwrap();

    // Create nested scopes: module -> class -> function -> block
    let class_scope = table.create_scope(ScopeKind::Class, Some(module_scope));
    let func_scope = table.create_scope(ScopeKind::Function, Some(class_scope));
    let block_scope = table.create_scope(ScopeKind::Block, Some(func_scope));

    // Verify hierarchy
    let block = table.get_scope(block_scope).unwrap();
    assert_eq!(block.parent(), Some(func_scope));

    let func = table.get_scope(func_scope).unwrap();
    assert_eq!(func.parent(), Some(class_scope));

    let class = table.get_scope(class_scope).unwrap();
    assert_eq!(class.parent(), Some(module_scope));
}

#[test]
fn test_enter_exit_scope() {
    let mut table = SymbolTable::new();
    let module_scope = table.current_scope().unwrap();

    // Create and enter a function scope
    let func_scope = table.create_scope(ScopeKind::Function, Some(module_scope));
    table.enter_scope(func_scope);
    assert_eq!(table.current_scope(), Some(func_scope));

    // Exit back to module scope
    assert!(table.exit_scope().is_some());
    assert_eq!(table.current_scope(), Some(module_scope));
}

#[test]
fn test_exit_scope_from_module_returns_none() {
    let mut table = SymbolTable::new();
    // Cannot exit from module scope - returns None
    assert_eq!(table.exit_scope(), None);
}

#[test]
fn test_symbol_definition() {
    let mut table = SymbolTable::new();
    let span = Span::new(0, 10);

    let symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable,
        NodeID::new(1, 0),
        span,
        table.current_scope().unwrap(),
    );

    assert!(table.define_symbol("x".to_string(), symbol).is_ok());
}

#[test]
fn test_duplicate_symbol_error() {
    let mut table = SymbolTable::new();
    let span = Span::new(0, 10);
    let scope = table.current_scope().unwrap();

    let symbol1 =
        Symbol::new("x".to_string(), SymbolKind::Variable, NodeID::new(1, 0), span, scope);

    let symbol2 =
        Symbol::new("x".to_string(), SymbolKind::Variable, NodeID::new(2, 0), span, scope);

    assert!(table.define_symbol("x".to_string(), symbol1).is_ok());
    assert!(table.define_symbol("x".to_string(), symbol2).is_err());
}

#[test]
fn test_symbol_lookup() {
    let mut table = SymbolTable::new();
    let span = Span::new(0, 10);
    let scope = table.current_scope().unwrap();

    let symbol = Symbol::new("x".to_string(), SymbolKind::Variable, NodeID::new(1, 0), span, scope);

    table.define_symbol("x".to_string(), symbol).unwrap();

    // Lookup in current scope
    let found = table.lookup_symbol("x");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "x");

    // Lookup non-existent symbol
    assert!(table.lookup_symbol("y").is_none());
}

#[test]
fn test_symbol_lookup_in_scope_chain() {
    let mut table = SymbolTable::new();
    let span = Span::new(0, 10);
    let module_scope = table.current_scope().unwrap();

    // Define symbol in module scope
    let mut module_symbol = Symbol::new(
        "global_var".to_string(),
        SymbolKind::Variable,
        NodeID::new(1, 0),
        span,
        module_scope,
    );
    module_symbol.set_global(true);
    table.define_symbol("global_var".to_string(), module_symbol).unwrap();

    // Create and enter function scope
    let func_scope = table.create_scope(ScopeKind::Function, Some(module_scope));
    table.enter_scope(func_scope);

    // Define local symbol
    let local_symbol = Symbol::new(
        "local_var".to_string(),
        SymbolKind::Variable,
        NodeID::new(2, 0),
        span,
        func_scope,
    );
    table.define_symbol("local_var".to_string(), local_symbol).unwrap();

    // Lookup should find both via scope chain
    assert!(table.lookup_in_scope_chain("local_var").is_some());
    assert!(table.lookup_in_scope_chain("global_var").is_some());

    // Exit to module scope
    let _ = table.exit_scope();

    // Now local_var should not be found in scope chain
    assert!(table.lookup_in_scope_chain("local_var").is_none());
    assert!(table.lookup_in_scope_chain("global_var").is_some());
}

#[test]
fn test_symbol_flags() {
    let span = Span::new(0, 10);
    let mut symbol = Symbol::new(
        "x".to_string(),
        SymbolKind::Variable,
        NodeID::new(1, 0),
        span,
        typhon_analyzer::symbol::ScopeID::new(0),
    );

    assert!(!symbol.is_mutable());
    assert!(!symbol.is_global());
    assert!(!symbol.is_used());

    symbol.set_mutable(true);
    assert!(symbol.is_mutable());

    symbol.mark_used();
    assert!(symbol.is_used());

    symbol.mark_defined();
    assert!(symbol.is_defined());
}

#[test]
fn test_node_scope_association() {
    let mut table = SymbolTable::new();
    let module_scope = table.current_scope().unwrap();
    let node_id = NodeID::new(42, 0);

    table.associate_node_with_scope(node_id, module_scope);
    assert_eq!(table.get_node_scope(node_id), Some(module_scope));
}

#[test]
fn test_semantic_context_integration() {
    let mut context = SemanticContext::new();
    let span = Span::new(0, 10);

    // Access symbol table through context
    let scope = context.symbol_table().current_scope().unwrap();

    let symbol =
        Symbol::new("test".to_string(), SymbolKind::Variable, NodeID::new(1, 0), span, scope);

    context.symbol_table_mut().define_symbol("test".to_string(), symbol).unwrap();

    let found = context.symbol_table().lookup_symbol("test");
    assert!(found.is_some());
}
