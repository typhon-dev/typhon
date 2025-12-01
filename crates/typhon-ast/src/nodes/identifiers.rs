//! Identifier node types
//!
//! This module provides identifier types used throughout the AST for names, variables,
//! parameters, and specialized identifier variants.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Core Identifier Types
// ============================================================================

/// A unified identifier node in the AST.
///
/// Represents any identifier in the source code. Naming convention information
/// (private, constant, mangled) is computed from the name string itself rather
/// than encoded in separate types.
///
/// # Examples
/// - Simple: `variable`, `function_name`
/// - Private: `_private_var`, `_internal_function`
/// - Constant: `MAX_SIZE`, `PI`, `DEFAULT_VALUE`
/// - Private constant: `_MAX_RETRY_COUNT`, `_DEFAULT_TIMEOUT`
/// - Mangled: `__dunder__`, `__special_method__`
#[derive(Debug, Clone)]
pub struct BasicIdent {
    /// The identifier's name as a string
    pub name: String,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl BasicIdent {
    /// Creates a new identifier
    #[must_use]
    pub const fn new(name: String, id: NodeID, span: Span) -> Self {
        Self { name, id, parent: None, span }
    }

    /// Returns true if this is a constant identifier (all uppercase with optional underscores)
    #[must_use]
    pub fn is_const(&self) -> bool {
        !self.name.is_empty()
            && self.name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            && self.name.chars().any(char::is_alphabetic)
    }

    /// Returns true if this is a dunder/magic method (starts and ends with `__`)
    ///
    /// Examples: `__init__`, `__repr__`, `__add__`
    #[must_use]
    pub fn is_dunder(&self) -> bool {
        self.name.starts_with("__") && self.name.ends_with("__") && self.name.len() > 4
    }

    /// Returns true if this is a mangled identifier (starts with `__` but doesn't end with `__`)
    ///
    /// Names starting with `__` (but not ending with `__`) trigger name mangling
    /// in class contexts, becoming `_ClassName__attribute`.
    #[must_use]
    pub fn is_mangled(&self) -> bool { self.name.starts_with("__") && !self.name.ends_with("__") }

    /// Returns true if this is a private identifier (starts with single underscore)
    #[must_use]
    pub fn is_private(&self) -> bool { self.name.starts_with('_') && !self.name.starts_with("__") }

    /// Returns true if this is a private constant (private + constant naming)
    #[must_use]
    pub fn is_private_const(&self) -> bool { self.is_private() && self.is_const() }
}

impl ASTNode for BasicIdent {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Identifier }

    fn span(&self) -> Span { self.span }
}

impl_visitable!(BasicIdent, visit_basic_ident);

impl fmt::Display for BasicIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.name) }
}

/// A function or method parameter
///
/// Represents a function parameter with optional type annotation and default value.
/// This is kept separate from `BasicIdent` due to its unique structure with child nodes.
#[derive(Debug, Clone)]
pub struct ParameterIdent {
    /// The parameter name
    pub name: String,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The type annotation node ID (if any)
    pub type_annotation: Option<NodeID>,
    /// The default value expression node ID (if any)
    pub default_value: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ParameterIdent {
    /// Creates a new parameter
    #[must_use]
    pub const fn new(name: String, id: NodeID, span: Span) -> Self {
        Self { name, id, parent: None, type_annotation: None, default_value: None, span }
    }

    /// Sets the type annotation of this parameter
    #[must_use]
    pub const fn with_type(mut self, type_annotation: NodeID) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }

    /// Sets the default value of this parameter
    #[must_use]
    pub const fn with_default(mut self, default_value: NodeID) -> Self {
        self.default_value = Some(default_value);
        self
    }
}

impl ASTNode for ParameterIdent {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Identifier }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();

        // Add type annotation and default value if present
        if let Some(type_id) = self.type_annotation {
            children.push(type_id);
        }

        if let Some(default_id) = self.default_value {
            children.push(default_id);
        }

        children
    }
}

impl_visitable!(ParameterIdent, visit_parameter_ident);

impl fmt::Display for ParameterIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.type_annotation.is_some() {
            write!(f, ": <type>")?;
        }
        if self.default_value.is_some() {
            write!(f, " = <default>")?;
        }
        Ok(())
    }
}

/// A variable reference in an expression
///
/// Represents a reference to a variable in an expression context.
/// This is kept separate from `BasicIdent` because it has a different `NodeKind`
/// (`Expression` instead of `Identifier`) to distinguish usage context.
#[derive(Debug, Clone)]
pub struct VariableIdent {
    /// The variable name
    pub name: String,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl VariableIdent {
    /// Creates a new variable reference
    #[must_use]
    pub const fn new(name: String, id: NodeID, span: Span) -> Self {
        Self { name, id, parent: None, span }
    }

    /// Returns true if this is a private variable (starts with single underscore)
    #[must_use]
    pub fn is_private(&self) -> bool { self.name.starts_with('_') && !self.name.starts_with("__") }

    /// Returns true if this is a constant variable (all uppercase with optional underscores)
    #[must_use]
    pub fn is_const(&self) -> bool {
        !self.name.is_empty()
            && self.name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            && self.name.chars().any(char::is_alphabetic)
    }

    /// Returns true if this is a mangled variable (starts with `__` but doesn't end with `__`)
    #[must_use]
    pub fn is_mangled(&self) -> bool { self.name.starts_with("__") && !self.name.ends_with("__") }

    /// Returns true if this is a dunder/magic method reference (starts and ends with `__`)
    #[must_use]
    pub fn is_dunder(&self) -> bool {
        self.name.starts_with("__") && self.name.ends_with("__") && self.name.len() > 4
    }
}

impl ASTNode for VariableIdent {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }
}

impl_visitable!(VariableIdent, visit_variable_ident);

impl fmt::Display for VariableIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.name) }
}
