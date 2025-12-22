//! Symbol resolution using the analyzer's symbol table.
//!
//! This module handles name resolution and symbol lookups during MIR lowering.

use typhon_analyzer::symbol::{Symbol, SymbolKind, SymbolTable};

use crate::error::LoweringResult;

/// Classification of a name's scope and binding.
///
/// Used to determine the correct MIR instruction for variable access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameClassification {
    /// A built-in function or constant (e.g., `len`, `True`)
    Builtin,
    /// A captured variable from an enclosing scope (closure)
    Captured,
    /// A global/module-level variable
    Global,
    /// A local variable in the current function
    Local,
}

/// Classifies a name based on symbol information.
///
/// Determines whether a name refers to a local, captured, global, or builtin variable.
///
/// ## Arguments
///
/// - `symbol` - The symbol information from the symbol table
/// - `in_closure` - Whether we're currently inside a closure
/// - `is_local` - Whether the name exists in the local variable map
///
/// ## Returns
///
/// The classification of the name, or `None` if it cannot be classified.
#[must_use]
pub fn classify_name(
    symbol: &Symbol,
    in_closure: bool,
    is_local: bool,
) -> Option<NameClassification> {
    // Check if it's a builtin
    if symbol.kind == SymbolKind::Builtin {
        return Some(NameClassification::Builtin);
    }

    // Check if it's a local variable
    if is_local {
        return Some(NameClassification::Local);
    }

    // Check if it's captured (in closure and not in current scope)
    if in_closure && !is_local {
        return Some(NameClassification::Captured);
    }

    // Check if it's a global
    if symbol.is_global() {
        return Some(NameClassification::Global);
    }

    // If none of the above, it's likely a global
    Some(NameClassification::Global)
}

/// Checks if a symbol represents a builtin.
///
/// ## Arguments
///
/// - `symbol` - The symbol to check
///
/// ## Returns
///
/// `true` if the symbol is a builtin, `false` otherwise.
#[must_use]
pub const fn is_builtin(symbol: &Symbol) -> bool { matches!(symbol.kind, SymbolKind::Builtin) }

/// Checks if a symbol represents a global variable.
///
/// ## Arguments
///
/// - `symbol` - The symbol to check
///
/// ## Returns
///
/// `true` if the symbol is global, `false` otherwise.
#[must_use]
pub const fn is_global(symbol: &Symbol) -> bool { symbol.is_global() }

/// Resolves a base class name to its type ID.
///
/// Looks up the class name in the symbol table and extracts its type ID.
///
/// ## Arguments
///
/// - `symbol_table` - The symbol table to query
/// - `name` - The base class name
///
/// ## Returns
///
/// The type ID of the base class if found, `None` otherwise.
#[must_use]
pub fn resolve_base_class(symbol_table: &SymbolTable, name: &str) -> Option<usize> {
    let symbol = symbol_table.lookup_in_scope_chain(name)?;
    symbol.type_id
}

/// Resolves a name to a symbol using the symbol table.
///
/// Looks up the name in the symbol table's scope chain and returns the symbol if found.
///
/// ## Arguments
///
/// - `symbol_table` - The symbol table to query
/// - `name` - The name to resolve
///
/// ## Errors
///
/// Returns [`LoweringError`](crate::error::LoweringError) if symbol resolution fails.
pub fn resolve_name<'sym>(
    symbol_table: &'sym SymbolTable,
    name: &'sym str,
) -> LoweringResult<Option<&'sym Symbol>> {
    // Look up the symbol in the scope chain
    Ok(symbol_table.lookup_in_scope_chain(name))
}

#[cfg(test)]
mod tests {
    use typhon_analyzer::symbol::ScopeID;
    use typhon_ast::nodes::NodeID;
    use typhon_source::types::Span;

    use super::*;

    fn create_test_symbol(name: &str, kind: SymbolKind) -> Symbol {
        Symbol::new(name.to_string(), kind, NodeID::placeholder(), Span::default(), ScopeID::new(0))
    }

    #[test]
    fn test_classify_builtin() {
        let symbol = create_test_symbol("len", SymbolKind::Builtin);
        let classification = classify_name(&symbol, false, false);

        assert_eq!(classification, Some(NameClassification::Builtin));
    }

    #[test]
    fn test_classify_local() {
        let symbol = create_test_symbol("x", SymbolKind::Variable);
        let classification = classify_name(&symbol, false, true);

        assert_eq!(classification, Some(NameClassification::Local));
    }

    #[test]
    fn test_classify_captured() {
        let symbol = create_test_symbol("x", SymbolKind::Variable);
        let classification = classify_name(&symbol, true, false);

        assert_eq!(classification, Some(NameClassification::Captured));
    }

    #[test]
    fn test_classify_global() {
        let mut symbol = create_test_symbol("x", SymbolKind::Variable);

        symbol.set_global(true);

        let classification = classify_name(&symbol, false, false);

        assert_eq!(classification, Some(NameClassification::Global));
    }

    #[test]
    fn test_is_builtin() {
        let symbol = create_test_symbol("len", SymbolKind::Builtin);

        assert!(is_builtin(&symbol));

        let symbol = create_test_symbol("x", SymbolKind::Variable);

        assert!(!is_builtin(&symbol));
    }

    #[test]
    fn test_is_global() {
        let mut symbol = create_test_symbol("x", SymbolKind::Variable);

        assert!(!is_global(&symbol));

        symbol.set_global(true);

        assert!(is_global(&symbol));
    }
}
