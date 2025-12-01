use std::collections::HashMap;
use std::rc::Rc;

use inkwell::values::BasicValueEnum;

use crate::typesystem::types::Type;

/// A symbol entry in the symbol table.
#[derive(Debug)]
pub struct SymbolEntry {
    /// The LLVM value representing the variable.
    pub value: BasicValueEnum,
    /// The type of the variable.
    pub ty: Rc<Type>,
    /// Whether the variable is mutable.
    pub mutable: bool,
}

/// A symbol table for tracking variables in scope.
#[derive(Debug)]
pub struct SymbolTable {
    /// Nested scopes, with the last one being the current scope.
    scopes: Vec<HashMap<String, SymbolEntry>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// Create a new symbol table.
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());
        SymbolTable { scopes }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add a symbol to the current scope.
    pub fn add_symbol(&mut self, name: String, entry: SymbolEntry) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, entry);
        }
    }

    /// Look up a symbol in the current scope chain.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        // Look in scopes from inner to outer
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }
}
