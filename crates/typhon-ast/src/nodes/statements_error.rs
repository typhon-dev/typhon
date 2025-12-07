//! Error handling statement node types for the AST.
//!
//! This module contains statement types for exception handling,
//! including try/except/else/finally blocks, raise statements,
//! and context managers (with/async with).

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Async With Statement
// ============================================================================

/// Async with statement (e.g. `async with aiofiles.open('file.txt') as f: ...`).
#[derive(Debug, Clone)]
pub struct AsyncWithStmt {
    /// The context managers for the async with statement.
    ///
    /// Each item is a tuple of (`context_expr`, `optional_var`) where:
    /// - `context_expr` is the expression that evaluates to an async context manager
    /// - `optional_var` is an optional variable to assign the result of __aenter__ to
    pub items: Vec<(NodeID, Option<NodeID>)>,
    /// The body of the async with statement
    pub body: Vec<NodeID>,
    /// The unique ID for this node
    pub id: NodeID,
    /// The parent node ID
    pub parent: Option<NodeID>,
    /// The span of source code for this statement
    pub span: Span,
}

impl AsyncWithStmt {
    /// Creates a new async with statement
    #[must_use]
    pub const fn new(
        items: Vec<(NodeID, Option<NodeID>)>,
        body: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { items, body, id, parent: None, span }
    }
}

impl ASTNode for AsyncWithStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::with_capacity(self.items.len() * 2 + self.body.len());

        for (context_expr, optional_var) in &self.items {
            children.push(*context_expr);
            if let Some(var) = optional_var {
                children.push(*var);
            }
        }

        children.extend_from_slice(&self.body);
        children
    }
}

impl_visitable!(AsyncWithStmt, visit_async_with_stmt);

impl fmt::Display for AsyncWithStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "async with ")?;

        for (i, (expr, optional_var)) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{expr}")?;

            if let Some(var) = optional_var {
                write!(f, " as {var}")?;
            }
        }

        write!(f, ":\n    ...")
    }
}

// ============================================================================
// Except Handler
// ============================================================================

/// Except handler (e.g. `except Exception as e: ...`).
#[derive(Debug, Clone)]
pub struct ExceptHandler {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
    /// The type of exception to catch (None for bare except:)
    pub exception_type: Option<NodeID>,
    /// The name to bind the exception to (as e)
    pub name: Option<NodeID>,
    /// The body of the except handler
    pub body: Vec<NodeID>,
}

impl ExceptHandler {
    /// Creates a new except handler
    #[must_use]
    pub const fn new(id: NodeID, body: Vec<NodeID>, span: Span) -> Self {
        Self { id, parent: None, exception_type: None, name: None, body, span }
    }

    /// Sets the exception type of this except handler
    #[must_use]
    pub const fn with_exception_type(mut self, exception_type: NodeID) -> Self {
        self.exception_type = Some(exception_type);
        self
    }

    /// Sets the name of this except handler
    #[must_use]
    pub const fn with_name(mut self, name: NodeID) -> Self {
        self.name = Some(name);
        self
    }
}

impl ASTNode for ExceptHandler {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();

        if let Some(exception_type) = self.exception_type {
            children.push(exception_type);
        }

        if let Some(name) = self.name {
            children.push(name);
        }

        children.extend(&self.body);
        children
    }
}

impl_visitable!(ExceptHandler, visit_except_handler);

impl fmt::Display for ExceptHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "ExceptHandler") }
}

// ============================================================================
// Raise Statement
// ============================================================================

/// Raise statement (e.g. `raise Exception`).
#[derive(Debug, Clone, Copy)]
pub struct RaiseStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
    /// The exception expression to raise (None for bare raise to re-raise)
    pub exception: Option<NodeID>,
    /// The cause of the exception (from clause)
    pub cause: Option<NodeID>,
}

impl RaiseStmt {
    /// Creates a new raise statement
    #[must_use]
    pub const fn new(id: NodeID, span: Span) -> Self {
        Self { id, parent: None, exception: None, cause: None, span }
    }

    /// Sets the cause of this raise statement
    #[must_use]
    pub const fn with_cause(mut self, cause: Option<NodeID>) -> Self {
        self.cause = cause;
        self
    }

    /// Sets the exception of this raise statement
    #[must_use]
    pub const fn with_exception(mut self, exception: Option<NodeID>) -> Self {
        self.exception = exception;
        self
    }
}

impl ASTNode for RaiseStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();

        if let Some(exception) = self.exception {
            children.push(exception);
        }

        if let Some(cause) = self.cause {
            children.push(cause);
        }

        children
    }
}

impl_visitable!(RaiseStmt, visit_raise_stmt);

impl fmt::Display for RaiseStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Raise") }
}

// ============================================================================
// Try Statement
// ============================================================================

/// Try statement (e.g. `try: ... except: ... else: ... finally: ...`).
#[derive(Debug, Clone)]
pub struct TryStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
    /// The body of the try block
    pub body: Vec<NodeID>,
    /// The except handlers of the try statement
    pub handlers: Vec<NodeID>,
    /// The optional else body of the try statement (executed when no exception is raised)
    pub else_body: Option<Vec<NodeID>>,
    /// The optional finally body of the try statement (executed regardless of whether an exception is raised)
    pub finally_body: Option<Vec<NodeID>>,
}

impl TryStmt {
    /// Creates a new try statement
    #[must_use]
    pub const fn new(id: NodeID, body: Vec<NodeID>, handlers: Vec<NodeID>, span: Span) -> Self {
        Self { id, parent: None, body, handlers, else_body: None, finally_body: None, span }
    }

    /// Sets the else body of this try statement
    #[must_use]
    pub fn with_else_body(mut self, else_body: Vec<NodeID>) -> Self {
        self.else_body = Some(else_body);
        self
    }

    /// Sets the finally body of this try statement
    #[must_use]
    pub fn with_finally_body(mut self, finally_body: Vec<NodeID>) -> Self {
        self.finally_body = Some(finally_body);
        self
    }
}

impl ASTNode for TryStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();

        children.extend(&self.body);
        children.extend(&self.handlers);

        if let Some(else_body) = &self.else_body {
            children.extend(else_body);
        }

        if let Some(finally_body) = &self.finally_body {
            children.extend(finally_body);
        }

        children
    }
}

impl_visitable!(TryStmt, visit_try_stmt);

impl fmt::Display for TryStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Try") }
}

// ============================================================================
// With Statement
// ============================================================================

/// With statement (e.g. `with open('file.txt') as f: ...`).
#[derive(Debug, Clone)]
pub struct WithStmt {
    /// The context managers for the with statement.
    ///
    /// Each item is a tuple of (`context_expr`, `optional_var`) where:
    /// - `context_expr` is the expression that evaluates to a context manager
    /// - `optional_var` is an optional variable to assign the result of __enter__ to
    pub items: Vec<(NodeID, Option<NodeID>)>,
    /// The body of the with statement
    pub body: Vec<NodeID>,
    /// The unique ID for this node
    pub id: NodeID,
    /// The parent node ID
    pub parent: Option<NodeID>,
    /// The span of source code for this statement
    pub span: Span,
}

impl WithStmt {
    /// Creates a new with statement
    #[must_use]
    pub const fn new(
        items: Vec<(NodeID, Option<NodeID>)>,
        body: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { items, body, id, parent: None, span }
    }
}

impl ASTNode for WithStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::with_capacity(self.items.len() * 2 + self.body.len());

        for (context_expr, optional_var) in &self.items {
            children.push(*context_expr);
            if let Some(var) = optional_var {
                children.push(*var);
            }
        }

        children.extend_from_slice(&self.body);
        children
    }
}

impl_visitable!(WithStmt, visit_with_stmt);

impl fmt::Display for WithStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "with ")?;

        for (i, (expr, optional_var)) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{expr}")?;

            if let Some(var) = optional_var {
                write!(f, " as {var}")?;
            }
        }

        write!(f, ":\n    ...")
    }
}
