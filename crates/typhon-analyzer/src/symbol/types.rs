//! Symbol definitions and metadata.

use bitflags::bitflags;
use typhon_ast::nodes::NodeID;
use typhon_source::types::Span;

use super::scope::ScopeID;

/// The kind of symbol.
///
/// Identifies what type of declaration the symbol represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// A Python builtin function or type.
    Builtin,
    /// A class declaration.
    Class,
    /// A function or method declaration.
    Function,
    /// An imported name.
    Import,
    /// A module.
    Module,
    /// A function/method parameter.
    Parameter,
    /// A type parameter (generic).
    TypeParameter,
    /// A variable declaration.
    Variable,
}

bitflags! {
    /// Flags indicating properties of a symbol.
    ///
    /// These flags track various properties like mutability, scope visibility,
    /// and usage patterns.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SymbolFlags: u32 {
        /// Symbol can be modified after declaration.
        const MUTABLE = 1 << 0;
        /// Symbol is declared at global (module) scope.
        const GLOBAL = 1 << 1;
        /// Symbol references a nonlocal variable from an enclosing scope.
        const NONLOCAL = 1 << 2;
        /// Symbol has been used/referenced.
        const USED = 1 << 3;
        /// Symbol has been defined/assigned.
        const DEFINED = 1 << 4;
    }
}

/// Represents a symbol in the program.
///
/// A symbol corresponds to a declared name (variable, function, class, etc.)
/// and tracks its type, location, and various properties.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// The symbol's name.
    pub name: String,
    /// The kind of symbol.
    pub kind: SymbolKind,
    /// Flags indicating symbol properties.
    pub flags: SymbolFlags,
    /// The type ID of this symbol (if known).
    pub type_id: Option<usize>,
    /// The AST node that defines this symbol.
    pub definition_node: NodeID,
    /// The span of the symbol's definition.
    pub span: Span,
    /// The scope where this symbol was declared.
    pub scope_id: ScopeID,
    /// All AST nodes that reference this symbol.
    pub references: Vec<NodeID>,
    /// Scopes that capture this variable (for closure analysis).
    pub captured_by: Vec<ScopeID>,
}

impl Symbol {
    /// Creates a new symbol with the given properties.
    #[must_use]
    pub const fn new(
        name: String,
        kind: SymbolKind,
        definition_node: NodeID,
        span: Span,
        scope_id: ScopeID,
    ) -> Self {
        Self {
            name,
            kind,
            flags: SymbolFlags::empty(),
            type_id: None,
            definition_node,
            span,
            scope_id,
            references: Vec::new(),
            captured_by: Vec::new(),
        }
    }

    /// Adds a scope that captures this variable.
    pub fn add_capture(&mut self, scope_id: ScopeID) {
        if !self.captured_by.contains(&scope_id) {
            self.captured_by.push(scope_id);
        }
    }

    /// Adds a reference to this symbol.
    pub fn add_reference(&mut self, node_id: NodeID) { self.references.push(node_id); }

    /// Returns true if this symbol is captured by any closure.
    #[must_use]
    pub const fn is_captured(&self) -> bool { !self.captured_by.is_empty() }

    /// Returns true if this symbol has been defined.
    #[must_use]
    pub const fn is_defined(&self) -> bool { self.flags.contains(SymbolFlags::DEFINED) }

    /// Returns true if this symbol is global.
    #[must_use]
    pub const fn is_global(&self) -> bool { self.flags.contains(SymbolFlags::GLOBAL) }

    /// Returns true if this symbol is mutable.
    #[must_use]
    pub const fn is_mutable(&self) -> bool { self.flags.contains(SymbolFlags::MUTABLE) }

    /// Returns true if this symbol is nonlocal.
    #[must_use]
    pub const fn is_nonlocal(&self) -> bool { self.flags.contains(SymbolFlags::NONLOCAL) }

    /// Returns true if this symbol has been used.
    #[must_use]
    pub const fn is_used(&self) -> bool { self.flags.contains(SymbolFlags::USED) }

    /// Marks this symbol as defined.
    pub fn mark_defined(&mut self) { self.flags.insert(SymbolFlags::DEFINED); }

    /// Marks this symbol as used.
    pub fn mark_used(&mut self) { self.flags.insert(SymbolFlags::USED); }

    /// Sets the global flag.
    pub fn set_global(&mut self, global: bool) {
        if global {
            self.flags.insert(SymbolFlags::GLOBAL);
        } else {
            self.flags.remove(SymbolFlags::GLOBAL);
        }
    }

    /// Sets the mutability flag.
    pub fn set_mutable(&mut self, mutable: bool) {
        if mutable {
            self.flags.insert(SymbolFlags::MUTABLE);
        } else {
            self.flags.remove(SymbolFlags::MUTABLE);
        }
    }

    /// Sets the nonlocal flag.
    pub fn set_nonlocal(&mut self, nonlocal: bool) {
        if nonlocal {
            self.flags.insert(SymbolFlags::NONLOCAL);
        } else {
            self.flags.remove(SymbolFlags::NONLOCAL);
        }
    }
}
