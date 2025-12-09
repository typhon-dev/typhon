//! Scope management for symbol tables.

use std::fmt;

use rustc_hash::FxHashMap;

use super::types::Symbol;
use crate::error::SemanticError;

/// Unique identifier for a scope.
///
/// `ScopeID` is a newtype wrapper around `u32` that uniquely identifies
/// a scope within a symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeID(u32);

impl ScopeID {
    /// Creates a new `ScopeID` with the given value.
    #[must_use]
    pub const fn new(id: u32) -> Self { Self(id) }

    /// Returns the inner value of the `ScopeID`.
    #[must_use]
    pub const fn value(self) -> u32 { self.0 }
}

impl fmt::Display for ScopeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "scope:{}", self.0) }
}

/// The kind of scope.
///
/// Different scope kinds have different visibility and lifetime rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Block scope (for control flow blocks like if, while, for).
    Block,
    /// Class scope (for class definitions).
    Class,
    /// Comprehension scope (for list/dict/set comprehensions).
    Comprehension,
    /// Function scope (for function and method definitions).
    Function,
    /// Lambda scope (for lambda expressions).
    Lambda,
    /// Module-level scope (global scope for a module).
    Module,
}

/// Represents a lexical scope in the program.
///
/// A scope contains symbols defined at that level and maintains parent-child
/// relationships with other scopes to form the scope hierarchy.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique identifier for this scope.
    pub id: ScopeID,
    /// The kind of scope.
    pub kind: ScopeKind,
    /// Parent scope ID (None for module scope).
    pub parent: Option<ScopeID>,
    /// Symbols defined in this scope.
    pub symbols: FxHashMap<String, Symbol>,
    /// Child scope IDs.
    pub children: Vec<ScopeID>,
}

impl Scope {
    /// Creates a new scope with the given ID, kind, and parent.
    #[must_use]
    pub fn new(id: ScopeID, kind: ScopeKind, parent: Option<ScopeID>) -> Self {
        Self { id, kind, parent, symbols: FxHashMap::default(), children: Vec::new() }
    }

    /// Returns the child scope IDs.
    #[must_use]
    pub fn children(&self) -> &[ScopeID] { &self.children }

    /// Gets a symbol from this scope by name.
    ///
    /// This only searches the current scope, not parent scopes.
    #[must_use]
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> { self.symbols.get(name) }

    /// Gets a mutable reference to a symbol from this scope by name.
    ///
    /// This only searches the current scope, not parent scopes.
    pub fn get_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }

    /// Inserts a symbol into this scope.
    ///
    /// ## Errors
    ///
    /// Returns [`SemanticError::DuplicateSymbol`] if a symbol with the same name
    /// already exists in this scope.
    pub fn insert_symbol(&mut self, name: String, symbol: Symbol) -> Result<(), SemanticError> {
        if let Some(existing) = self.symbols.get(&name) {
            return Err(SemanticError::DuplicateSymbol {
                name,
                original_span: existing.span,
                duplicate_span: symbol.span,
            });
        }

        drop(self.symbols.insert(name, symbol));

        Ok(())
    }

    /// Returns the parent scope ID, if any.
    #[must_use]
    pub const fn parent(&self) -> Option<ScopeID> { self.parent }
}
