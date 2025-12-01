//! Declaration node types
//!
//! This file contains all declaration types: functions, classes, variables, and type aliases.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// AsyncFunctionDef
// ============================================================================

/// Async function definition in the AST (e.g. `async def func(params): body`).
#[derive(Debug, Clone)]
pub struct AsyncFunctionDecl {
    /// The function name
    pub name: String,
    /// The function parameters
    pub parameters: Vec<NodeID>,
    /// The function body statements
    pub body: Vec<NodeID>,
    /// Optional return type annotation
    pub return_type: Option<NodeID>,
    /// Optional function decorators
    pub decorators: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AsyncFunctionDecl {
    /// Creates a new async function definition
    #[must_use]
    pub const fn new(
        name: String,
        parameters: Vec<NodeID>,
        body: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            return_type: None,
            decorators: Vec::new(),
            id,
            parent: None,
            span,
        }
    }

    /// Sets the return type of this async function
    #[must_use]
    pub const fn with_return_type(mut self, return_type: NodeID) -> Self {
        self.return_type = Some(return_type);
        self
    }

    /// Adds decorators to this async function
    #[must_use]
    pub fn with_decorators(mut self, decorators: Vec<NodeID>) -> Self {
        self.decorators = decorators;
        self
    }
}

impl ASTNode for AsyncFunctionDecl {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Declaration }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();
        children.extend(&self.decorators);
        children.extend(&self.parameters);
        if let Some(return_type) = self.return_type {
            children.push(return_type);
        }
        children.extend(&self.body);
        children
    }
}

impl_visitable!(AsyncFunctionDecl, visit_async_function_decl);

impl fmt::Display for AsyncFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "async def {}(...)", self.name)
    }
}

// ============================================================================
// ClassDef
// ============================================================================

/// Class definition in the AST (e.g. `class Name[(bases])]: body`).
#[derive(Debug, Clone)]
pub struct ClassDecl {
    /// The class name
    pub name: String,
    /// The base classes (if any)
    pub bases: Vec<NodeID>,
    /// The class body statements (methods, class variables, etc.)
    pub body: Vec<NodeID>,
    /// Optional class decorators
    pub decorators: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ClassDecl {
    /// Creates a new class definition
    #[must_use]
    pub const fn new(
        name: String,
        bases: Vec<NodeID>,
        body: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { name, bases, body, decorators: Vec::new(), id, parent: None, span }
    }

    /// Adds decorators to this class
    #[must_use]
    pub fn with_decorators(mut self, decorators: Vec<NodeID>) -> Self {
        self.decorators = decorators;
        self
    }
}

impl ASTNode for ClassDecl {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Declaration }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();
        children.extend(&self.decorators);
        children.extend(&self.bases);
        children.extend(&self.body);
        children
    }
}

impl_visitable!(ClassDecl, visit_class_decl);

impl fmt::Display for ClassDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "class {}(...)", self.name)
    }
}

// ============================================================================
// FunctionDef
// ============================================================================

/// Function definition in the AST (e.g. `def func(params): ...`).
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    /// The function name
    pub name: String,
    /// The function parameters
    pub parameters: Vec<NodeID>,
    /// The function body statements
    pub body: Vec<NodeID>,
    /// Optional return type annotation
    pub return_type: Option<NodeID>,
    /// Optional function decorators
    pub decorators: Vec<NodeID>,
    /// Whether the function is async
    pub is_async: bool,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl FunctionDecl {
    /// Creates a new function definition
    #[must_use]
    pub const fn new(
        name: String,
        parameters: Vec<NodeID>,
        body: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            return_type: None,
            decorators: Vec::new(),
            is_async: false,
            id,
            parent: None,
            span,
        }
    }

    /// Sets the return type of this function
    #[must_use]
    pub const fn with_return_type(mut self, return_type: NodeID) -> Self {
        self.return_type = Some(return_type);
        self
    }

    /// Adds decorators to this function
    #[must_use]
    pub fn with_decorators(mut self, decorators: Vec<NodeID>) -> Self {
        self.decorators = decorators;
        self
    }

    /// Marks this function as async
    #[must_use]
    pub const fn as_async(mut self) -> Self {
        self.is_async = true;
        self
    }
}

impl ASTNode for FunctionDecl {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Declaration }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();
        children.extend(&self.decorators);
        children.extend(&self.parameters);
        if let Some(return_type) = self.return_type {
            children.push(return_type);
        }
        children.extend(&self.body);
        children
    }
}

impl_visitable!(FunctionDecl, visit_function_decl);

impl fmt::Display for FunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let async_prefix = if self.is_async { "async " } else { "" };
        write!(f, "{}def {}(...)", async_prefix, self.name)
    }
}

// ============================================================================
// TypeDef
// ============================================================================

/// Type definition in the AST (e.g. `type Alias = OriginalType`).
#[derive(Debug, Clone)]
pub struct TypeDecl {
    /// The type alias name
    pub name: String,
    /// The original type expression
    pub original_type: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl TypeDecl {
    /// Creates a new type definition
    #[must_use]
    pub const fn new(name: String, original_type: NodeID, id: NodeID, span: Span) -> Self {
        Self { name, original_type, id, parent: None, span }
    }
}

impl ASTNode for TypeDecl {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Declaration }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.original_type] }
}

impl_visitable!(TypeDecl, visit_type_decl);

impl fmt::Display for TypeDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = <type>", self.name)
    }
}

// ============================================================================
// VariableDecl
// ============================================================================

/// Variable definition in the AST (e.g. `x: int = 5`).
#[derive(Debug, Clone)]
pub struct VariableDecl {
    /// The variable name
    pub name: String,
    /// Optional type annotation
    pub type_annotation: Option<NodeID>,
    /// Optional initial value
    pub value: Option<NodeID>,
    /// Whether the variable is final (const)
    pub is_final: bool,
    /// Whether the variable is private
    pub is_private: bool,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl VariableDecl {
    /// Creates a new variable definition
    #[must_use]
    pub const fn new(name: String, id: NodeID, span: Span) -> Self {
        Self {
            name,
            type_annotation: None,
            value: None,
            is_private: false,
            is_final: false,
            id,
            parent: None,
            span,
        }
    }

    /// Sets the type annotation of this variable
    #[must_use]
    pub const fn with_type(mut self, type_annotation: NodeID) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }

    /// Sets the initial value of this variable
    #[must_use]
    pub const fn with_value(mut self, value: NodeID) -> Self {
        self.value = Some(value);
        self
    }

    /// Marks this variable as final (const)
    #[must_use]
    pub const fn as_final(mut self) -> Self {
        self.is_final = true;
        self
    }

    /// Marks this variable as private
    #[must_use]
    pub const fn as_private(mut self) -> Self {
        self.is_private = true;
        self
    }
}

impl ASTNode for VariableDecl {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Declaration }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();
        if let Some(type_id) = self.type_annotation {
            children.push(type_id);
        }
        if let Some(value_id) = self.value {
            children.push(value_id);
        }
        children
    }
}

impl_visitable!(VariableDecl, visit_variable_decl);

impl fmt::Display for VariableDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = if self.is_final { "const " } else { "" };
        write!(f, "{}{}", prefix, self.name)?;

        if self.type_annotation.is_some() {
            write!(f, ": <type>")?;
        }

        if self.value.is_some() {
            write!(f, " = <value>")?;
        }

        Ok(())
    }
}
