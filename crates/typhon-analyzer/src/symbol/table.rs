//! Symbol table implementation.

use rustc_hash::FxHashMap;
use typhon_ast::nodes::NodeID;
use typhon_source::types::Span;

use super::BUILTINS;
use super::scope::{Scope, ScopeID, ScopeKind};
use super::types::{Symbol, SymbolKind};
use crate::error::SemanticError;

/// The main symbol table managing all scopes and symbols.
///
/// The symbol table maintains a hierarchical structure of scopes and provides
/// methods for scope management, symbol definition, and name resolution.
#[derive(Debug)]
pub struct SymbolTable {
    /// All scopes indexed by ID.
    scopes: Vec<Scope>,
    /// Current scope stack (for traversal).
    scope_stack: Vec<ScopeID>,
    /// Map from AST node ID to scope ID.
    node_to_scope: FxHashMap<NodeID, ScopeID>,
    /// Next scope ID to allocate.
    next_scope_id: u32,
}

impl SymbolTable {
    /// Creates a new symbol table with a module scope.
    #[must_use]
    pub fn new() -> Self {
        let mut table = Self {
            scopes: Vec::new(),
            scope_stack: Vec::new(),
            node_to_scope: FxHashMap::default(),
            next_scope_id: 0,
        };

        // Create the module scope
        let module_scope_id = table.create_scope(ScopeKind::Module, None);
        table.scope_stack.push(module_scope_id);

        // Register Python builtins in module scope
        table.register_builtins();

        table
    }

    /// Associates an AST node with a scope.
    pub fn associate_node_with_scope(&mut self, node_id: NodeID, scope_id: ScopeID) {
        let _ = self.node_to_scope.insert(node_id, scope_id);
    }

    /// Creates a new scope with the given kind and parent.
    ///
    /// Returns the ID of the newly created scope.
    pub fn create_scope(&mut self, kind: ScopeKind, parent: Option<ScopeID>) -> ScopeID {
        let id = ScopeID::new(self.next_scope_id);
        self.next_scope_id += 1;

        let scope = Scope::new(id, kind, parent);

        // Add to parent's children
        if let Some(parent_id) = parent
            && let Some(parent_scope) = self.scopes.get_mut(parent_id.value() as usize)
        {
            parent_scope.children.push(id);
        }

        self.scopes.push(scope);
        id
    }

    /// Gets the current scope ID.
    #[must_use]
    pub fn current_scope(&self) -> Option<ScopeID> { self.scope_stack.last().copied() }

    /// Defines a symbol in the current scope.
    ///
    /// ## Errors
    ///
    /// Returns [`SemanticError::NoActiveScope`] if there is no active scope.
    /// Returns [`SemanticError::DuplicateSymbol`] if a symbol with the same name
    /// already exists in the current scope.
    pub fn define_symbol(&mut self, name: String, symbol: Symbol) -> Result<(), SemanticError> {
        let scope_id = self.current_scope().ok_or(SemanticError::NoActiveScope)?;
        let scope =
            self.scopes.get_mut(scope_id.value() as usize).ok_or(SemanticError::NoActiveScope)?;

        scope.insert_symbol(name, symbol)
    }

    /// Enters a scope by pushing it onto the scope stack.
    pub fn enter_scope(&mut self, scope_id: ScopeID) { self.scope_stack.push(scope_id); }

    /// Exits the current scope by popping it from the scope stack.
    ///
    /// Returns the ID of the exited scope, or None if already at module scope.
    pub fn exit_scope(&mut self) -> Option<ScopeID> {
        // Don't pop the module scope
        if self.scope_stack.len() > 1 { self.scope_stack.pop() } else { None }
    }

    /// Gets the scope associated with an AST node.
    #[must_use]
    pub fn get_node_scope(&self, node_id: NodeID) -> Option<ScopeID> {
        self.node_to_scope.get(&node_id).copied()
    }

    /// Returns the root scope.
    #[must_use]
    pub fn get_root_scope(&self) -> Option<&Scope> { self.scopes.first() }

    /// Gets a reference to a scope by ID.
    #[must_use]
    pub fn get_scope(&self, scope_id: ScopeID) -> Option<&Scope> {
        self.scopes.get(scope_id.value() as usize)
    }

    /// Gets a mutable reference to a scope by ID.
    pub fn get_scope_mut(&mut self, scope_id: ScopeID) -> Option<&mut Scope> {
        self.scopes.get_mut(scope_id.value() as usize)
    }

    /// Looks up a symbol by searching up the scope chain.
    ///
    /// Starts from the current scope and searches parent scopes until
    /// the symbol is found or the module scope is reached.
    #[must_use]
    pub fn lookup_in_scope_chain(&self, name: &str) -> Option<&Symbol> {
        for &scope_id in self.scope_stack.iter().rev() {
            if let Some(scope) = self.scopes.get(scope_id.value() as usize)
                && let Some(symbol) = scope.get_symbol(name)
            {
                return Some(symbol);
            }
        }
        None
    }

    /// Looks up a symbol by name, searching all scopes.
    ///
    /// This searches through all scopes in the table, which is useful for testing
    /// and introspection. For normal name resolution, use [`lookup_in_scope_chain`].
    #[must_use]
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.scopes.iter().find_map(|scope| scope.get_symbol(name))
    }

    /// Returns the module (root) scope ID.
    #[must_use]
    pub fn module_scope(&self) -> Option<ScopeID> { self.scopes.first().map(|scope| scope.id) }

    /// Returns the total number of scopes in the table.
    #[must_use]
    pub const fn scope_count(&self) -> usize { self.scopes.len() }

    /// Returns an iterator over all scopes and their IDs.
    ///
    /// This is primarily useful for testing and debugging.
    pub fn scopes(&self) -> impl Iterator<Item = (ScopeID, &Scope)> {
        self.scopes.iter().map(|scope| (scope.id, scope))
    }

    /// Registers Python builtin functions and types in the module scope.
    fn register_builtins(&mut self) {
        // Use placeholder NodeID and empty span for builtins since they're not from source
        let builtin_node_id = NodeID::placeholder();
        let builtin_span = Span::default();

        if let Some(module_scope_id) = self.module_scope() {
            for &builtin_name in BUILTINS {
                let symbol = Symbol::new(
                    builtin_name.to_string(),
                    SymbolKind::Builtin,
                    builtin_node_id,
                    builtin_span,
                    module_scope_id,
                );

                // Directly insert into module scope, ignoring duplicate errors
                // (should never happen since we're initializing)
                if let Some(scope) = self.scopes.get_mut(module_scope_id.value() as usize) {
                    drop(scope.insert_symbol(builtin_name.to_string(), symbol));
                }
            }
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self { Self::new() }
}
