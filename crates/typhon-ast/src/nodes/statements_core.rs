//! Core statement node types for the AST.
//!
//! This module contains fundamental statement types that handle assignments,
//! assertions, deletions, scope declarations, imports, and other core language features.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Assert Statement
// ============================================================================

/// Assert statement (e.g. `assert condition [, message]`).
#[derive(Debug, Clone, Copy)]
pub struct AssertStmt {
    /// The condition to assert
    pub condition: NodeID,
    /// Optional error message
    pub message: Option<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AssertStmt {
    /// Creates a new assert statement
    #[must_use]
    pub const fn new(condition: NodeID, message: Option<NodeID>, id: NodeID, span: Span) -> Self {
        Self { condition, message, id, parent: None, span }
    }
}

impl ASTNode for AssertStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.condition];
        if let Some(message) = self.message {
            children.push(message);
        }
        children
    }
}

impl_visitable!(AssertStmt, visit_assert_stmt);

impl fmt::Display for AssertStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Assert(has_message: {})", self.message.is_some())
    }
}

// ============================================================================
// Assignment Statements
// ============================================================================

/// Assignment statement node (e.g. `target = value`).
#[derive(Debug, Clone, Copy)]
pub struct AssignmentStmt {
    /// The target to assign to
    pub target: NodeID,
    /// The value to assign
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AssignmentStmt {
    /// Creates a new assignment statement
    #[must_use]
    pub const fn new(target: NodeID, value: NodeID, id: NodeID, span: Span) -> Self {
        Self { target, value, id, parent: None, span }
    }
}

impl ASTNode for AssignmentStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.target, self.value] }
}

impl_visitable!(AssignmentStmt, visit_assignment_stmt);

impl fmt::Display for AssignmentStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "AssignmentStmt") }
}

/// Augmented assignment operator (e.g. `+=`, `-=`, `*=`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AugmentedAssignmentOp {
    /// `+=` operator
    Add,
    /// `&=` operator
    BitAnd,
    /// `|=` operator
    BitOr,
    /// `^=` operator
    BitXor,
    /// `/=` operator
    Div,
    /// `//=` operator (floor division)
    FloorDiv,
    /// `<<=` operator
    LShift,
    /// `@=` operator (matrix multiplication)
    MatMul,
    /// `%=` operator
    Mod,
    /// `*=` operator
    Mul,
    /// `**=` operator
    Pow,
    /// `>>=` operator
    RShift,
    /// `-=` operator
    Sub,
}

impl fmt::Display for AugmentedAssignmentOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+="),
            Self::BitAnd => write!(f, "&="),
            Self::BitOr => write!(f, "|="),
            Self::BitXor => write!(f, "^="),
            Self::Div => write!(f, "/="),
            Self::FloorDiv => write!(f, "//="),
            Self::LShift => write!(f, "<<="),
            Self::MatMul => write!(f, "@="),
            Self::Mod => write!(f, "%="),
            Self::Mul => write!(f, "*="),
            Self::Pow => write!(f, "**="),
            Self::RShift => write!(f, ">>="),
            Self::Sub => write!(f, "-="),
        }
    }
}

/// Augmented assignment statement (e.g. `x += 1`, `y *= 2`).
#[derive(Debug, Clone, Copy)]
pub struct AugmentedAssignmentStmt {
    /// The target of the augmented assignment
    pub target: NodeID,
    /// The operator used for the augmented assignment
    pub operator: AugmentedAssignmentOp,
    /// The value to use in the augmented assignment
    pub value: NodeID,
    /// The unique ID for this node
    pub id: NodeID,
    /// The parent node ID
    pub parent: Option<NodeID>,
    /// The span of source code for this statement
    pub span: Span,
}

impl AugmentedAssignmentStmt {
    /// Creates a new augmented assignment statement
    #[must_use]
    pub const fn new(
        target: NodeID,
        operator: AugmentedAssignmentOp,
        value: NodeID,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { target, operator, value, id, parent: None, span }
    }
}

impl ASTNode for AugmentedAssignmentStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.target, self.value] }
}

impl_visitable!(AugmentedAssignmentStmt, visit_augmented_assignment_stmt);

impl fmt::Display for AugmentedAssignmentStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.target, self.operator, self.value)
    }
}

// ============================================================================
// Delete Statement
// ============================================================================

/// Delete statement (e.g. `del x`, `del x, y, z`, `del obj.attr`).
#[derive(Debug, Clone)]
pub struct DeleteStmt {
    /// The targets to be deleted
    pub targets: Vec<NodeID>,
    /// The unique ID for this node
    pub id: NodeID,
    /// The parent node ID
    pub parent: Option<NodeID>,
    /// The span of source code for this statement
    pub span: Span,
}

impl DeleteStmt {
    /// Creates a new delete statement
    #[must_use]
    pub const fn new(targets: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { targets, id, parent: None, span }
    }
}

impl ASTNode for DeleteStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.targets.clone() }
}

impl_visitable!(DeleteStmt, visit_delete_stmt);

impl fmt::Display for DeleteStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "del ")?;
        for (i, target_id) in self.targets.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{target_id}")?;
        }
        Ok(())
    }
}

// ============================================================================
// Expression Statement
// ============================================================================

/// Expression statement (an expression used as a statement).
#[derive(Debug, Clone, Copy)]
pub struct ExpressionStmt {
    /// The expression being used as a statement
    pub expression: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ExpressionStmt {
    /// Creates a new expression statement
    #[must_use]
    pub const fn new(expression: NodeID, id: NodeID, span: Span) -> Self {
        Self { expression, id, parent: None, span }
    }
}

impl ASTNode for ExpressionStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.expression] }
}

impl_visitable!(ExpressionStmt, visit_expression_stmt);

impl fmt::Display for ExpressionStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "ExpressionStmt") }
}

// ============================================================================
// Global Statement
// ============================================================================

/// Global statement (e.g. `global x, y, z`).
#[derive(Debug, Clone)]
pub struct GlobalStmt {
    /// The names being declared as global
    pub names: Vec<NodeID>,
    /// The unique ID for this node
    pub id: NodeID,
    /// The parent node ID
    pub parent: Option<NodeID>,
    /// The span of source code for this statement
    pub span: Span,
}

impl GlobalStmt {
    /// Creates a new global statement
    #[must_use]
    pub const fn new(names: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { names, id, parent: None, span }
    }
}

impl ASTNode for GlobalStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.names.clone() }
}

impl_visitable!(GlobalStmt, visit_global_stmt);

impl fmt::Display for GlobalStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "global ")?;
        for (i, name_id) in self.names.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name_id}")?;
        }
        Ok(())
    }
}

// ============================================================================
// Nonlocal Statement
// ============================================================================

/// Nonlocal statement (e.g. `nonlocal x, y, z`).
#[derive(Debug, Clone)]
pub struct NonlocalStmt {
    /// The names being declared as nonlocal
    pub names: Vec<NodeID>,
    /// The unique ID for this node
    pub id: NodeID,
    /// The parent node ID
    pub parent: Option<NodeID>,
    /// The span of source code for this statement
    pub span: Span,
}

impl NonlocalStmt {
    /// Creates a new nonlocal statement
    #[must_use]
    pub const fn new(names: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { names, id, parent: None, span }
    }
}

impl ASTNode for NonlocalStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.names.clone() }
}

impl_visitable!(NonlocalStmt, visit_nonlocal_stmt);

impl fmt::Display for NonlocalStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nonlocal ")?;
        for (i, name_id) in self.names.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name_id}")?;
        }
        Ok(())
    }
}

// ============================================================================
// Pass Statement
// ============================================================================

/// Pass statement (e.g. `pass`).
#[derive(Debug, Clone, Copy)]
pub struct PassStmt {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl PassStmt {
    /// Creates a new pass statement
    #[must_use]
    pub const fn new(id: NodeID, span: Span) -> Self { Self { id, parent: None, span } }
}

impl ASTNode for PassStmt {
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

impl_visitable!(PassStmt, visit_pass_stmt);

impl fmt::Display for PassStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Pass") }
}
