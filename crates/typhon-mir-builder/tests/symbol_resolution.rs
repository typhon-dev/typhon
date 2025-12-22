//! Tests for symbol resolution integration with MIR builder.

use typhon_analyzer::context::AnalysisContext;
use typhon_analyzer::symbol::{ScopeID, Symbol, SymbolKind, SymbolTable};
use typhon_ast::ast::AST;
use typhon_ast::nodes::NodeID;
use typhon_mir_builder::context::LoweringContext;
use typhon_mir_builder::symbol_resolution::{
    NameClassification,
    classify_name,
    is_builtin,
    is_global,
    resolve_base_class,
    resolve_name,
};
use typhon_source::types::Span;

/// Helper to create a test symbol
fn create_symbol(name: &str, kind: SymbolKind) -> Symbol {
    Symbol::new(name.to_string(), kind, NodeID::placeholder(), Span::default(), ScopeID::new(0))
}

#[test]
fn test_name_classification_builtin() {
    let symbol = create_symbol("len", SymbolKind::Builtin);
    let classification = classify_name(&symbol, false, false);

    assert_eq!(classification, Some(NameClassification::Builtin));
}

#[test]
fn test_name_classification_local() {
    let symbol = create_symbol("x", SymbolKind::Variable);
    let classification = classify_name(&symbol, false, true);

    assert_eq!(classification, Some(NameClassification::Local));
}

#[test]
fn test_name_classification_captured() {
    let symbol = create_symbol("x", SymbolKind::Variable);
    let classification = classify_name(&symbol, true, false);

    assert_eq!(classification, Some(NameClassification::Captured));
}

#[test]
fn test_name_classification_global() {
    let mut symbol = create_symbol("x", SymbolKind::Variable);

    symbol.set_global(true);

    let classification = classify_name(&symbol, false, false);

    assert_eq!(classification, Some(NameClassification::Global));
}

#[test]
fn test_is_builtin_function() {
    let symbol = create_symbol("len", SymbolKind::Builtin);

    assert!(is_builtin(&symbol));

    let symbol = create_symbol("print", SymbolKind::Builtin);

    assert!(is_builtin(&symbol));
}

#[test]
fn test_is_builtin_variable() {
    let symbol = create_symbol("x", SymbolKind::Variable);

    assert!(!is_builtin(&symbol));
}

#[test]
fn test_is_global_variable() {
    let mut symbol = create_symbol("x", SymbolKind::Variable);

    assert!(!is_global(&symbol));

    symbol.set_global(true);

    assert!(is_global(&symbol));
}

#[test]
fn test_resolve_name_from_symbol_table() {
    let mut table = SymbolTable::new();
    let symbol = create_symbol("test_var", SymbolKind::Variable);

    // Define symbol in current scope
    table.define_symbol("test_var".to_string(), symbol).unwrap();

    // Resolve it
    let result = resolve_name(&table, "test_var").unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "test_var");
}

#[test]
fn test_resolve_name_not_found() {
    let table = SymbolTable::new();
    let result = resolve_name(&table, "nonexistent").unwrap();

    assert!(result.is_none());
}

#[test]
fn test_resolve_name_builtin() {
    let table = SymbolTable::new();

    // Builtins are registered automatically
    let result = resolve_name(&table, "len").unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().kind, SymbolKind::Builtin);
}

#[test]
fn test_resolve_base_class_found() {
    let mut table = SymbolTable::new();
    let mut symbol = create_symbol("BaseClass", SymbolKind::Class);

    symbol.type_id = Some(42);

    table.define_symbol("BaseClass".to_string(), symbol).unwrap();

    let type_id = resolve_base_class(&table, "BaseClass");

    assert_eq!(type_id, Some(42));
}

#[test]
fn test_resolve_base_class_not_found() {
    let table = SymbolTable::new();
    let type_id = resolve_base_class(&table, "NonexistentClass");

    assert!(type_id.is_none());
}

#[test]
fn test_resolve_base_class_no_type_id() {
    let mut table = SymbolTable::new();
    let symbol = create_symbol("IncompleteClass", SymbolKind::Class);

    table.define_symbol("IncompleteClass".to_string(), symbol).unwrap();

    let type_id = resolve_base_class(&table, "IncompleteClass");

    assert!(type_id.is_none());
}

#[test]
fn test_lowering_context_classify_name() {
    let ast = AST::new();
    let ctx = LoweringContext::new(&ast, "test_module".to_string());

    // Without semantic context, classification should return None
    let classification = ctx.classify_name("any_var");

    assert!(classification.is_none());
}

#[test]
fn test_lowering_context_resolve_symbol() {
    let ast = AST::new();
    let ctx = LoweringContext::new(&ast, "test_module".to_string());

    // Without semantic context, resolution should return None
    let symbol = ctx.resolve_symbol("any_var");

    assert!(symbol.is_none());
}

#[test]
fn test_lowering_context_resolve_base_class() {
    let ast = AST::new();
    let ctx = LoweringContext::new(&ast, "test_module".to_string());

    // Without semantic context, resolution should return None
    let type_id = ctx.resolve_base_class("BaseClass");

    assert!(type_id.is_none());
}

#[test]
fn test_lowering_context_with_semantics() {
    let ast = AST::new();
    let analysis_ctx = AnalysisContext::new();
    let ctx = LoweringContext::new_with_semantics(&ast, "test_module".to_string(), &analysis_ctx);

    // With semantic context, classify_name should work
    // But since we haven't defined any symbols, it still returns None
    let classification = ctx.classify_name("len"); // builtin

    // Returns None because len is not in locals map
    assert!(classification.is_some());
}

#[test]
fn test_classification_priority_local_over_captured() {
    let symbol = create_symbol("x", SymbolKind::Variable);

    // When a variable is both in closure and local, prefer Local
    let classification = classify_name(&symbol, true, true);

    assert_eq!(classification, Some(NameClassification::Local));
}

#[test]
fn test_classification_builtin_takes_precedence() {
    let symbol = create_symbol("len", SymbolKind::Builtin);

    // Builtin should be classified as builtin regardless of other flags
    let classification = classify_name(&symbol, false, true);

    assert_eq!(classification, Some(NameClassification::Builtin));
}

#[test]
fn test_multiple_symbol_kinds() {
    let kinds = vec![
        SymbolKind::Builtin,
        SymbolKind::Class,
        SymbolKind::Function,
        SymbolKind::Import,
        SymbolKind::Module,
        SymbolKind::Parameter,
        SymbolKind::TypeParameter,
        SymbolKind::Variable,
    ];

    for kind in kinds {
        let symbol = create_symbol("test", kind);

        // Only Builtin should be detected as builtin
        assert_eq!(is_builtin(&symbol), kind == SymbolKind::Builtin);
    }
}

#[test]
fn test_symbol_table_scope_chain() {
    let mut table = SymbolTable::new();

    // Create a nested scope
    let parent_scope = table.current_scope().unwrap();
    let child_scope =
        table.create_scope(typhon_analyzer::symbol::ScopeKind::Function, Some(parent_scope));

    // Define a symbol in parent scope
    let symbol = create_symbol("parent_var", SymbolKind::Variable);

    table.define_symbol("parent_var".to_string(), symbol).unwrap();

    // Enter child scope
    table.enter_scope(child_scope);

    // Should be able to resolve parent_var from child scope
    let result = resolve_name(&table, "parent_var").unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "parent_var");
}
