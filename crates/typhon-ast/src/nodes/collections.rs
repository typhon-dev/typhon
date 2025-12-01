//! Collection node types
//!
//! This file contains container types (List, Dict, Set, Tuple) and their
//! comprehension variants, along with shared comprehension structures.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Comprehension Base Types
// ============================================================================

/// Represents a comprehension component: `(for x in y if z)`
#[derive(Debug, Clone)]
pub struct ComprehensionFor {
    /// The target expression for the iteration (e.g. `x` in `for x in y`)
    pub target: NodeID,
    /// The iterable expression (e.g. `y` in `for x in y`)
    pub iter: NodeID,
    /// The condition expressions (e.g. `z` in `for x in y if z`)
    pub ifs: Vec<NodeID>,
    /// The span of this for clause
    pub span: Span,
}

impl ComprehensionFor {
    /// Creates a new for clause for a comprehension
    #[must_use]
    pub const fn new(target: NodeID, iter: NodeID, ifs: Vec<NodeID>, span: Span) -> Self {
        Self { target, iter, ifs, span }
    }

    /// Returns a list of children nodes
    #[must_use]
    pub fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.target, self.iter];
        children.extend_from_slice(&self.ifs);
        children
    }
}

impl fmt::Display for ComprehensionFor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "for _ in _ with {} conditions", self.ifs.len())
    }
}

/// A common trait for all comprehension expressions
pub trait Comprehension: ASTNode {
    /// Returns the element or key-value expressions for the comprehension
    fn element(&self) -> NodeID;

    /// Returns the list of generators (for clauses) in the comprehension
    fn generators(&self) -> &[ComprehensionFor];
}

// ============================================================================
// Dictionary
// ============================================================================

/// Represents a dictionary expression in the AST (e.g. `{key: value, ...}`)
#[derive(Debug, Clone)]
pub struct DictExpr {
    /// The dictionary entries (key-value pairs)
    pub entries: Vec<(NodeID, NodeID)>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl DictExpr {
    /// Creates a new dictionary expression
    #[must_use]
    pub const fn new(entries: Vec<(NodeID, NodeID)>, id: NodeID, span: Span) -> Self {
        Self { entries, id, parent: None, span }
    }
}

impl ASTNode for DictExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::with_capacity(self.entries.len() * 2);
        for (key, value) in &self.entries {
            children.push(*key);
            children.push(*value);
        }
        children
    }
}

impl_visitable!(DictExpr, visit_dict_expr);

impl fmt::Display for DictExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dict(entries: {})", self.entries.len())
    }
}

// ============================================================================
// Dictionary Comprehension
// ============================================================================

/// Represents a dictionary comprehension expression in the AST
/// (e.g. `{k: v for k, v in zip(keys, values)}`)
#[derive(Debug, Clone)]
pub struct DictComprehensionExpr {
    /// The key expression (e.g. `k` in `{k: v for k, v in zip(keys, values)}`)
    pub key: NodeID,
    /// The value expression (e.g. `v` in `{k: v for k, v in zip(keys, values)}`)
    pub value: NodeID,
    /// The generators (for clauses) in the comprehension
    pub generators: Vec<ComprehensionFor>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl DictComprehensionExpr {
    /// Creates a new dictionary comprehension expression
    #[must_use]
    pub const fn new(
        key: NodeID,
        value: NodeID,
        generators: Vec<ComprehensionFor>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { key, value, generators, id, parent: None, span }
    }
}

impl ASTNode for DictComprehensionExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.key, self.value];
        for generator in &self.generators {
            children.append(&mut generator.children());
        }
        children
    }
}

impl Comprehension for DictComprehensionExpr {
    fn element(&self) -> NodeID { self.key }

    fn generators(&self) -> &[ComprehensionFor] { &self.generators }
}

impl_visitable!(DictComprehensionExpr, visit_dict_comprehension_expr);

impl fmt::Display for DictComprehensionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DictComprehension(generators: {})", self.generators.len())
    }
}

// ============================================================================
// Generator Expression
// ============================================================================

/// Represents a generator expression in the AST
/// (e.g. `(x for x in range(10))`)
#[derive(Debug, Clone)]
pub struct GeneratorExpr {
    /// The element expression (e.g. `x` in `(x for x in range(10))`)
    pub element: NodeID,
    /// The generators (for clauses) in the comprehension
    pub generators: Vec<ComprehensionFor>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl GeneratorExpr {
    /// Creates a new generator expression
    #[must_use]
    pub const fn new(
        element: NodeID,
        generators: Vec<ComprehensionFor>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { element, generators, id, parent: None, span }
    }
}

impl ASTNode for GeneratorExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.element];
        for generator in &self.generators {
            children.append(&mut generator.children());
        }
        children
    }
}

impl Comprehension for GeneratorExpr {
    fn element(&self) -> NodeID { self.element }

    fn generators(&self) -> &[ComprehensionFor] { &self.generators }
}

impl_visitable!(GeneratorExpr, visit_generator_expr);

impl fmt::Display for GeneratorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Generator(generators: {})", self.generators.len())
    }
}

// ============================================================================
// List
// ============================================================================

/// Represents a list expression in the AST (e.g. `[1, 2, 3]`)
#[derive(Debug, Clone)]
pub struct ListExpr {
    /// The elements of the list
    pub elements: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ListExpr {
    /// Creates a new list expression
    #[must_use]
    pub const fn new(elements: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { elements, id, parent: None, span }
    }
}

impl ASTNode for ListExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.elements.clone() }
}

impl_visitable!(ListExpr, visit_list_expr);

impl fmt::Display for ListExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List(elements: {})", self.elements.len())
    }
}

// ============================================================================
// List Comprehension
// ============================================================================

/// Represents a list comprehension expression in the AST
/// (e.g. `[x for x in range(10) if x % 2 == 0]`)
#[derive(Debug, Clone)]
pub struct ListComprehensionExpr {
    /// The element expression (e.g. `x` in `[x for x in range(10)]`)
    pub element: NodeID,
    /// The generators (for clauses) in the comprehension
    pub generators: Vec<ComprehensionFor>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ListComprehensionExpr {
    /// Creates a new list comprehension expression
    #[must_use]
    pub const fn new(
        element: NodeID,
        generators: Vec<ComprehensionFor>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { element, generators, id, parent: None, span }
    }
}

impl ASTNode for ListComprehensionExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.element];
        for generator in &self.generators {
            children.append(&mut generator.children());
        }
        children
    }
}

impl Comprehension for ListComprehensionExpr {
    fn element(&self) -> NodeID { self.element }

    fn generators(&self) -> &[ComprehensionFor] { &self.generators }
}

impl_visitable!(ListComprehensionExpr, visit_list_comprehension_expr);

impl fmt::Display for ListComprehensionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListComprehension(generators: {})", self.generators.len())
    }
}

// ============================================================================
// Set
// ============================================================================

/// Represents a set expression in the AST (e.g. `{1, 2, 3}`)
#[derive(Debug, Clone)]
pub struct SetExpr {
    /// The elements of the set
    pub elements: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl SetExpr {
    /// Creates a new set expression
    #[must_use]
    pub const fn new(elements: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { elements, id, parent: None, span }
    }
}

impl ASTNode for SetExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.elements.clone() }
}

impl_visitable!(SetExpr, visit_set_expr);

impl fmt::Display for SetExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Set(elements: {})", self.elements.len())
    }
}

// ============================================================================
// Set Comprehension
// ============================================================================

/// Represents a set comprehension expression in the AST
/// (e.g. `{x*2 for x in range(5)}`)
#[derive(Debug, Clone)]
pub struct SetComprehensionExpr {
    /// The element expression (e.g. `x*2` in `{x*2 for x in range(5)}`)
    pub element: NodeID,
    /// The generators (for clauses) in the comprehension
    pub generators: Vec<ComprehensionFor>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl SetComprehensionExpr {
    /// Creates a new set comprehension expression
    #[must_use]
    pub const fn new(
        element: NodeID,
        generators: Vec<ComprehensionFor>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { element, generators, id, parent: None, span }
    }
}

impl ASTNode for SetComprehensionExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.element];
        for generator in &self.generators {
            children.append(&mut generator.children());
        }
        children
    }
}

impl Comprehension for SetComprehensionExpr {
    fn element(&self) -> NodeID { self.element }

    fn generators(&self) -> &[ComprehensionFor] { &self.generators }
}

impl_visitable!(SetComprehensionExpr, visit_set_comprehension_expr);

impl fmt::Display for SetComprehensionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SetComprehension(generators: {})", self.generators.len())
    }
}

// ============================================================================
// Tuple
// ============================================================================

/// Represents a tuple expression in the AST (e.g. `(1, 2, 3)`)
#[derive(Debug, Clone)]
pub struct TupleExpr {
    /// The elements of the tuple
    pub elements: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl TupleExpr {
    /// Creates a new tuple expression
    #[must_use]
    pub const fn new(elements: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { elements, id, parent: None, span }
    }
}

impl ASTNode for TupleExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.elements.clone() }
}

impl_visitable!(TupleExpr, visit_tuple_expr);

impl fmt::Display for TupleExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tuple(elements: {})", self.elements.len())
    }
}
