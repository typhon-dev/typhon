//! Control flow statement node types for the AST.
//!
//! This module contains statement types that control program execution flow,
//! including loops (for, while, async for), conditionals (if/elif/else),
//! and flow control keywords (break, continue, return).

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Async For Statement
// ============================================================================

/// Async for loop statement (e.g. `async for i in async_iterable: ...`).
#[derive(Debug, Clone)]
pub struct AsyncForStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
    /// The target being assigned to in each iteration
    pub target: NodeID,
    /// The async iterable being looped over
    pub iter: NodeID,
    /// The body of the loop
    pub body: Vec<NodeID>,
    /// The optional else body of the loop (executed when the loop completes normally)
    pub else_body: Option<Vec<NodeID>>,
}

impl AsyncForStmt {
    /// Creates a new async for loop statement
    #[must_use]
    pub const fn new(
        id: NodeID,
        target: NodeID,
        iter: NodeID,
        body: Vec<NodeID>,
        span: Span,
    ) -> Self {
        Self { id, parent: None, target, iter, body, else_body: None, span }
    }

    /// Sets the else body of this async for loop statement
    #[must_use]
    pub fn with_else_body(mut self, else_body: Vec<NodeID>) -> Self {
        self.else_body = Some(else_body);
        self
    }
}

impl ASTNode for AsyncForStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.target, self.iter];
        children.extend(&self.body);
        if let Some(else_body) = &self.else_body {
            children.extend(else_body);
        }
        children
    }
}

impl_visitable!(AsyncForStmt, visit_async_for_stmt);

impl fmt::Display for AsyncForStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "AsyncFor") }
}

// ============================================================================
// Break Statement
// ============================================================================

/// Break statement (e.g. `break`).
#[derive(Debug, Clone, Copy)]
pub struct BreakStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl BreakStmt {
    /// Creates a new break statement
    #[must_use]
    pub const fn new(id: NodeID, span: Span) -> Self { Self { id, parent: None, span } }
}

impl ASTNode for BreakStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { Vec::new() }
}

impl_visitable!(BreakStmt, visit_break_stmt);

impl fmt::Display for BreakStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Break") }
}

// ============================================================================
// Continue Statement
// ============================================================================

/// Continue statement (e.g. `continue`).
#[derive(Debug, Clone, Copy)]
pub struct ContinueStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ContinueStmt {
    /// Creates a new continue statement
    #[must_use]
    pub const fn new(id: NodeID, span: Span) -> Self { Self { id, parent: None, span } }
}

impl ASTNode for ContinueStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { Vec::new() }
}

impl_visitable!(ContinueStmt, visit_continue_stmt);

impl fmt::Display for ContinueStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Continue") }
}

// ============================================================================
// For Statement
// ============================================================================

/// For loop statement (e.g. `for i in range(10): ...`).
#[derive(Debug, Clone)]
pub struct ForStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
    /// The target being assigned to in each iteration
    pub target: NodeID,
    /// The iterable being looped over
    pub iter: NodeID,
    /// The body of the loop
    pub body: Vec<NodeID>,
    /// The optional else body of the loop (executed when the loop completes normally)
    pub else_body: Option<Vec<NodeID>>,
}

impl ForStmt {
    /// Creates a new for loop statement
    #[must_use]
    pub const fn new(
        id: NodeID,
        target: NodeID,
        iter: NodeID,
        body: Vec<NodeID>,
        span: Span,
    ) -> Self {
        Self { id, parent: None, target, iter, body, else_body: None, span }
    }

    /// Sets the else body of this for loop statement
    #[must_use]
    pub fn with_else_body(mut self, else_body: Vec<NodeID>) -> Self {
        self.else_body = Some(else_body);
        self
    }
}

impl ASTNode for ForStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.target, self.iter];
        children.extend(&self.body);
        if let Some(else_body) = &self.else_body {
            children.extend(else_body);
        }
        children
    }
}

impl_visitable!(ForStmt, visit_for_stmt);

impl fmt::Display for ForStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "For") }
}

// ============================================================================
// If Statement
// ============================================================================

/// If statement (e.g. `if condition: ... elif condition: ... else: ...`).
#[derive(Debug, Clone)]
pub struct IfStmt {
    /// The condition expression
    pub condition: NodeID,
    /// The body statements to execute if the condition is true
    pub body: Vec<NodeID>,
    /// Optional elif branches (condition, body pairs)
    pub elif_branches: Vec<(NodeID, Vec<NodeID>)>,
    /// Optional else body statements
    pub else_body: Option<Vec<NodeID>>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl IfStmt {
    /// Creates a new if statement
    #[must_use]
    pub const fn new(
        condition: NodeID,
        body: Vec<NodeID>,
        elif_branches: Vec<(NodeID, Vec<NodeID>)>,
        else_body: Option<Vec<NodeID>>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { condition, body, elif_branches, else_body, id, parent: None, span }
    }
}

impl ASTNode for IfStmt {
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
        children.push(self.condition);
        children.extend(&self.body);

        for (cond, stmts) in &self.elif_branches {
            children.push(*cond);
            children.extend(stmts);
        }

        if let Some(else_body) = &self.else_body {
            children.extend(else_body);
        }

        children
    }
}

impl_visitable!(IfStmt, visit_if_stmt);

impl fmt::Display for IfStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "If(elif_branches: {}, has_else: {})",
            self.elif_branches.len(),
            self.else_body.is_some()
        )
    }
}

// ============================================================================
// Return Statement
// ============================================================================

/// Return statement (e.g. `return [expr]`).
#[derive(Debug, Clone, Copy)]
pub struct ReturnStmt {
    /// The optional value to return
    pub value: Option<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ReturnStmt {
    /// Creates a new return statement
    #[must_use]
    pub const fn new(value: Option<NodeID>, id: NodeID, span: Span) -> Self {
        Self { value, id, parent: None, span }
    }
}

impl ASTNode for ReturnStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.value.map_or_else(Vec::new, |value| vec![value]) }
}

impl_visitable!(ReturnStmt, visit_return_stmt);

impl fmt::Display for ReturnStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Return(has_value: {})", self.value.is_some())
    }
}

// ============================================================================
// While Statement
// ============================================================================

/// While loop statement (e.g. `while condition: ...`).
#[derive(Debug, Clone)]
pub struct WhileStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
    /// The test condition of the while loop
    pub test: NodeID,
    /// The body of the while loop
    pub body: Vec<NodeID>,
    /// The optional else body of the while loop (executed when the loop completes normally)
    pub else_body: Option<Vec<NodeID>>,
}

impl WhileStmt {
    /// Creates a new while loop statement
    #[must_use]
    pub const fn new(id: NodeID, test: NodeID, body: Vec<NodeID>, span: Span) -> Self {
        Self { id, parent: None, test, body, else_body: None, span }
    }

    /// Sets the else body of this while loop statement
    #[must_use]
    pub fn with_else_body(mut self, else_body: Vec<NodeID>) -> Self {
        self.else_body = Some(else_body);
        self
    }
}

impl ASTNode for WhileStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.test];
        children.extend(&self.body);

        if let Some(else_body) = &self.else_body {
            children.extend(else_body);
        }

        children
    }
}

impl_visitable!(WhileStmt, visit_while_stmt);

impl fmt::Display for WhileStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "While") }
}
