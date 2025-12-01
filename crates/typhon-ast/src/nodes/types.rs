//! Type annotation node types
//!
//! This module provides type annotation types used for static typing in the AST.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Type Annotation Types
// ============================================================================

/// Callable type (e.g. `Callable[[int, str], bool]`)
///
/// This type wraps the functionality of `FunctionType` but uses the
/// appropriate visitor method for callable types.
#[derive(Debug, Clone)]
pub struct CallableType {
    /// The IDs of the parameter types
    pub param_ids: Vec<NodeID>,
    /// The ID of the return type
    pub return_type_id: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl CallableType {
    /// Creates a new callable type
    #[must_use]
    pub const fn new(
        param_ids: Vec<NodeID>,
        return_type_id: NodeID,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { param_ids, return_type_id, id, parent: None, span }
    }
}

impl ASTNode for CallableType {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Type }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = self.param_ids.clone();
        children.push(self.return_type_id);
        children
    }
}

impl_visitable!(CallableType, visit_callable_type);

impl fmt::Display for CallableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "CallableType") }
}

/// Generic type (e.g. `List[int]`)
#[derive(Debug, Clone)]
pub struct GenericType {
    /// The ID of the base type
    pub base_id: NodeID,
    /// The IDs of the type arguments
    pub arg_ids: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl GenericType {
    /// Creates a new generic type
    #[must_use]
    pub const fn new(base_id: NodeID, arg_ids: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { base_id, arg_ids, id, parent: None, span }
    }
}

impl ASTNode for GenericType {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Type }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();
        children.push(self.base_id);
        children.extend(&self.arg_ids);
        children
    }
}

impl_visitable!(GenericType, visit_generic_type);

impl fmt::Display for GenericType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "GenericType") }
}

/// Literal type (e.g. `Literal["red", "green", "blue"]`)
#[derive(Debug, Clone)]
pub struct LiteralType {
    /// The IDs of the literal values
    pub value_ids: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl LiteralType {
    /// Creates a new literal type
    #[must_use]
    pub const fn new(value_ids: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { value_ids, id, parent: None, span }
    }
}

impl ASTNode for LiteralType {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Type }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.value_ids.clone() }
}

impl_visitable!(LiteralType, visit_literal_type);

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "LiteralType") }
}

/// Tuple type (e.g. `tuple[int, str, float]`)
#[derive(Debug, Clone)]
pub struct TupleType {
    /// The IDs of the element types
    pub element_type_ids: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl TupleType {
    /// Creates a new tuple type
    #[must_use]
    pub const fn new(element_type_ids: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { element_type_ids, id, parent: None, span }
    }
}

impl ASTNode for TupleType {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Type }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.element_type_ids.clone() }
}

impl_visitable!(TupleType, visit_tuple_type);

impl fmt::Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "TupleType") }
}

/// Union type (e.g. `int | str`)
#[derive(Debug, Clone)]
pub struct UnionType {
    /// The IDs of the types in the union
    pub type_ids: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl UnionType {
    /// Creates a new union type
    #[must_use]
    pub const fn new(type_ids: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { type_ids, id, parent: None, span }
    }
}

impl ASTNode for UnionType {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Type }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.type_ids.clone() }
}

impl_visitable!(UnionType, visit_union_type);

impl fmt::Display for UnionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "UnionType") }
}
