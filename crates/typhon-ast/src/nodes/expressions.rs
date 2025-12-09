//! Expression node types
//!
//! This file contains the core expression types used in the AST.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Function Call Arguments
// ============================================================================

/// Represents a function call argument in the AST (e.g. `name=value`).
#[derive(Debug, Clone)]
pub struct ArgumentExpr {
    /// The argument name (if keyword argument)
    pub name: String,
    /// The argument value
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ArgumentExpr {
    /// Creates a new function call argument
    #[must_use]
    pub const fn new(name: String, value: NodeID, id: NodeID, span: Span) -> Self {
        Self { name, value, id, parent: None, span }
    }
}

impl ASTNode for ArgumentExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value] }
}

impl_visitable!(ArgumentExpr, visit_argument_expr);

impl fmt::Display for ArgumentExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Argument({})", self.name)
    }
}

// ============================================================================
// Assignment Expressions
// ============================================================================

/// Represents an assignment expression in the AST (e.g. `target := value`).
#[derive(Debug, Clone, Copy)]
pub struct AssignmentExpr {
    /// The assignment target
    pub target: NodeID,
    /// The value being assigned
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AssignmentExpr {
    /// Creates a new assignment expression
    #[must_use]
    pub const fn new(target: NodeID, value: NodeID, id: NodeID, span: Span) -> Self {
        Self { target, value, id, parent: None, span }
    }
}

impl ASTNode for AssignmentExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.target, self.value] }
}

impl_visitable!(AssignmentExpr, visit_assignment_expr);

impl fmt::Display for AssignmentExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Assignment") }
}

// ============================================================================
// Attribute Access
// ============================================================================

/// Represents an attribute access in the AST (e.g. `obj.attr`).
#[derive(Debug, Clone)]
pub struct AttributeExpr {
    /// The object being accessed
    pub value: NodeID,
    /// The attribute name
    pub name: String,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AttributeExpr {
    /// Creates a new attribute access
    #[must_use]
    pub const fn new(value: NodeID, name: String, id: NodeID, span: Span) -> Self {
        Self { value, name, id, parent: None, span }
    }
}

impl ASTNode for AttributeExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value] }
}

impl_visitable!(AttributeExpr, visit_attribute_expr);

impl fmt::Display for AttributeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Attribute({})", self.name)
    }
}

// ============================================================================
// Await Expressions
// ============================================================================

/// Represents an await expression in the AST (e.g. `await coroutine()`).
#[derive(Debug, Clone, Copy)]
pub struct AwaitExpr {
    /// The expression being awaited
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AwaitExpr {
    /// Creates a new await expression
    #[must_use]
    pub const fn new(value: NodeID, id: NodeID, span: Span) -> Self {
        Self { value, id, parent: None, span }
    }
}

impl ASTNode for AwaitExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value] }
}

impl_visitable!(AwaitExpr, visit_await_expr);

impl fmt::Display for AwaitExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Await") }
}

// ============================================================================
// Binary Operations
// ============================================================================

/// Represents a binary operation in the AST (e.g. `a + b`, `x * y`).
#[derive(Debug, Clone, Copy)]
pub struct BinaryOpExpr {
    /// The operator type
    pub op: BinaryOpKind,
    /// The left operand (expression)
    pub left: NodeID,
    /// The right operand (expression)
    pub right: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

/// Represents the type of binary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpKind {
    // Arithmetic
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    FloorDiv, // //
    Mod,      // %
    Pow,      // **

    // Bitwise
    BitAnd, // &
    BitOr,  // |
    BitXor, // ^
    LShift, // <<
    RShift, // >>

    // Comparison
    Eq,    // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=
    Is,    // is
    IsNot, // is not
    In,    // in
    NotIn, // not in

    // Logical
    And, // and
    Or,  // or

    // Matrix
    MatMul, // @
}

impl BinaryOpExpr {
    /// Creates a new binary operation
    #[must_use]
    pub const fn new(
        op: BinaryOpKind,
        left: NodeID,
        right: NodeID,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { op, left, right, id, parent: None, span }
    }
}

impl ASTNode for BinaryOpExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.left, self.right] }
}

impl_visitable!(BinaryOpExpr, visit_binary_op_expr);

impl fmt::Display for BinaryOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Binary({:?})", self.op) }
}

// ============================================================================
// Function Calls
// ============================================================================

/// Represents a function call in the AST (e.g. `func(a, b)`).
#[derive(Debug, Clone)]
pub struct CallExpr {
    /// The function being called (expression)
    pub func: NodeID,
    /// The arguments passed to the function
    pub args: Vec<NodeID>,
    /// The keyword arguments passed to the function
    pub keywords: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl CallExpr {
    /// Creates a new function call
    #[must_use]
    pub const fn new(
        func: NodeID,
        args: Vec<NodeID>,
        keywords: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { func, args, keywords, id, parent: None, span }
    }
}

impl ASTNode for CallExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::with_capacity(1 + self.args.len() + self.keywords.len());
        children.push(self.func);
        children.extend(&self.args);
        children.extend(&self.keywords);
        children
    }
}

impl_visitable!(CallExpr, visit_call_expr);

impl fmt::Display for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Call(args: {}, keywords: {})", self.args.len(), self.keywords.len())
    }
}

// ============================================================================
// Formatted Strings
// ============================================================================

/// A part of a formatted string (f-string), which can be either a literal text segment
/// or an expression to be evaluated and formatted.
#[derive(Debug, Clone)]
pub enum FmtStringPart {
    /// A literal text segment
    Literal(String),
    /// An expression to be evaluated and formatted
    Expression(NodeID),
}

/// Represents a formatted string expression in the AST (e.g. `f"Hello {name}"`).
#[derive(Debug, Clone)]
pub struct FmtStringExpr {
    /// The parts that make up the formatted string
    pub parts: Vec<FmtStringPart>,
    /// Whether this is a raw f-string (rf-string or fr-string)
    pub is_raw: bool,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl FmtStringExpr {
    /// Creates a new formatted string expression
    #[must_use]
    pub const fn new(parts: Vec<FmtStringPart>, is_raw: bool, id: NodeID, span: Span) -> Self {
        Self { parts, is_raw, id, parent: None, span }
    }
}

impl ASTNode for FmtStringExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();

        for part in &self.parts {
            if let FmtStringPart::Expression(expr) = part {
                children.push(*expr);
            }
        }

        children
    }
}

impl_visitable!(FmtStringExpr, visit_fmt_string_expr);

impl fmt::Display for FmtStringExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = if self.is_raw { "rf" } else { "f" };
        write!(f, "{prefix}\"...\"")
    }
}

// ============================================================================
// Grouping Expressions
// ============================================================================

/// Represents a parenthesized grouping expression in the AST (e.g. `(expr)`).
///
/// This node explicitly represents grouping/parenthesization in the AST, which is important
/// for preserving the original syntax and for code formatting/generation tools.
#[derive(Debug, Clone, Copy)]
pub struct GroupingExpr {
    /// The expression inside the parentheses
    pub expression: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl GroupingExpr {
    /// Creates a new grouping expression
    #[must_use]
    pub const fn new(expression: NodeID, id: NodeID, span: Span) -> Self {
        Self { expression, id, parent: None, span }
    }
}

impl ASTNode for GroupingExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.expression] }
}

impl_visitable!(GroupingExpr, visit_grouping_expr);

impl fmt::Display for GroupingExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Grouping") }
}

// ============================================================================
// Lambda Expressions
// ============================================================================

/// Represents a lambda expression in the AST (e.g. `lambda x, y: x + y`).
#[derive(Debug, Clone)]
pub struct LambdaExpr {
    /// The parameters of the lambda
    pub parameters: Vec<NodeID>,
    /// The body expression of the lambda
    pub body: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl LambdaExpr {
    /// Creates a new lambda expression
    #[must_use]
    pub const fn new(parameters: Vec<NodeID>, body: NodeID, id: NodeID, span: Span) -> Self {
        Self { parameters, body, id, parent: None, span }
    }
}

impl ASTNode for LambdaExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = self.parameters.clone();
        children.push(self.body);
        children
    }
}

impl_visitable!(LambdaExpr, visit_lambda_expr);

impl fmt::Display for LambdaExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lambda(params: {})", self.parameters.len())
    }
}

// ============================================================================
// Literal Expressions
// ============================================================================

/// Represents the value of a literal in the AST
#[derive(Debug, Clone)]
pub enum LiteralValue {
    /// Boolean literal
    Bool(bool),
    /// Bytes literal
    Bytes(Vec<u8>),
    /// Ellipsis literal
    Ellipsis,
    /// Float literal
    Float(f64),
    /// Integer literal
    Int(i64),
    /// None literal
    None,
    /// String literal
    String(String),
}

/// Represents a literal value in the AST (e.g. `42`, `"hello"`, `True`).
#[derive(Debug, Clone)]
pub struct LiteralExpr {
    /// The literal kind
    pub kind: LiteralValue,
    /// The raw value as a string
    pub raw_value: String,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl LiteralExpr {
    /// Creates a new literal
    #[must_use]
    pub const fn new(kind: LiteralValue, raw_value: String, id: NodeID, span: Span) -> Self {
        Self { kind, raw_value, id, parent: None, span }
    }
}

impl ASTNode for LiteralExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    // Literals have no children
    fn children(&self) -> Vec<NodeID> { vec![] }
}

impl_visitable!(LiteralExpr, visit_literal_expr);

impl fmt::Display for LiteralExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            LiteralValue::Bytes(val) => write!(f, "{val:?}"),
            LiteralValue::Int(val) => write!(f, "{val}"),
            LiteralValue::Float(val) => write!(f, "{val}"),
            LiteralValue::String(val) => write!(f, "{val:?}"),
            LiteralValue::Bool(val) => write!(f, "{val}"),
            LiteralValue::None => write!(f, "None"),
            LiteralValue::Ellipsis => write!(f, "..."),
        }
    }
}

// ============================================================================
// Slice Expression
// ============================================================================

/// Represents a slice expression in the AST (e.g. `start:stop:step`).
///
/// Python slices have the syntax `[start:stop:step]` where each component is optional:
/// - `[:]` - full slice (all elements)
/// - `[:stop]` - slice from beginning to stop
/// - `[start:]` - slice from start to end
/// - `[start:stop]` - slice from start to stop
/// - `[start:stop:step]` - slice with custom step
///
/// ## Examples
///
/// ```python
/// arr[:]           # SliceExpr { start: None, stop: None, step: None }
/// arr[1:]          # SliceExpr { start: Some(1), stop: None, step: None }
/// arr[:5]          # SliceExpr { start: None, stop: Some(5), step: None }
/// arr[1:5]         # SliceExpr { start: Some(1), stop: Some(5), step: None }
/// arr[::2]         # SliceExpr { start: None, stop: None, step: Some(2) }
/// arr[1:10:2]      # SliceExpr { start: Some(1), stop: Some(10), step: Some(2) }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SliceExpr {
    /// The start index (optional)
    pub start: Option<NodeID>,
    /// The stop index (optional)
    pub stop: Option<NodeID>,
    /// The step value (optional)
    pub step: Option<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl SliceExpr {
    /// Creates a new slice expression
    #[must_use]
    #[allow(clippy::similar_names)]
    pub const fn new(
        start: Option<NodeID>,
        stop: Option<NodeID>,
        step: Option<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { start, stop, step, id, parent: None, span }
    }
}

impl ASTNode for SliceExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();
        if let Some(start) = self.start {
            children.push(start);
        }
        if let Some(stop) = self.stop {
            children.push(stop);
        }
        if let Some(step) = self.step {
            children.push(step);
        }
        children
    }
}

impl_visitable!(SliceExpr, visit_slice_expr);

impl fmt::Display for SliceExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Slice({}:{}:{})",
            if self.start.is_some() { "start" } else { "" },
            if self.stop.is_some() { "stop" } else { "" },
            if self.step.is_some() { "step" } else { "" }
        )
    }
}

// ============================================================================
// Starred Expressions
// ============================================================================

/// Represents a starred expression in the AST (e.g. `*args`).
///
/// Starred expressions are used in multiple contexts:
/// - Function call arguments: `func(*args)`
/// - Unpacking in assignments: `a, *rest = values`
/// - List/set literals: `[1, *other_list, 3]`
#[derive(Debug, Clone, Copy)]
pub struct StarredExpr {
    /// The expression being starred
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl StarredExpr {
    /// Creates a new starred expression
    #[must_use]
    pub const fn new(value: NodeID, id: NodeID, span: Span) -> Self {
        Self { value, id, parent: None, span }
    }
}

impl ASTNode for StarredExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value] }
}

impl_visitable!(StarredExpr, visit_starred_expr);

impl fmt::Display for StarredExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Starred") }
}

// ============================================================================
// Subscription
// ============================================================================

/// Represents a subscription operation in the AST (e.g. `arr[idx]`).
#[derive(Debug, Clone, Copy)]
pub struct SubscriptionExpr {
    /// The object being subscripted
    pub value: NodeID,
    /// The index/slice expression
    pub index: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl SubscriptionExpr {
    /// Creates a new subscription operation
    #[must_use]
    pub const fn new(value: NodeID, index: NodeID, id: NodeID, span: Span) -> Self {
        Self { value, index, id, parent: None, span }
    }
}

impl ASTNode for SubscriptionExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value, self.index] }
}

impl_visitable!(SubscriptionExpr, visit_subscription_expr);

impl fmt::Display for SubscriptionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Subscription") }
}

// ============================================================================
// Template Strings
// ============================================================================

/// A part of a template string, which can be either a literal text segment
/// or an expression to be evaluated.
#[derive(Debug, Clone)]
pub enum TemplateStringPart {
    /// A literal text segment
    Literal(String),
    /// An expression to be evaluated
    Expression(NodeID),
}

/// Represents a template string in the AST (e.g. `t"Hello {name}"`).
///
/// Template strings allow embedding expressions within string literals
#[derive(Debug, Clone)]
pub struct TemplateStringExpr {
    /// The parts that make up the template string
    pub parts: Vec<TemplateStringPart>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl TemplateStringExpr {
    /// Creates a new template string
    #[must_use]
    pub const fn new(parts: Vec<TemplateStringPart>, id: NodeID, span: Span) -> Self {
        Self { parts, id, parent: None, span }
    }
}

impl ASTNode for TemplateStringExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::new();

        for part in &self.parts {
            if let TemplateStringPart::Expression(expr) = part {
                children.push(*expr);
            }
        }

        children
    }
}

impl_visitable!(TemplateStringExpr, visit_template_string_expr);

impl fmt::Display for TemplateStringExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "`...`") }
}

// ============================================================================
// Ternary Conditional
// ============================================================================

/// Represents a ternary conditional expression in the AST (e.g. `value if condition else other_value`).
///
/// This is Python's conditional expression syntax, often called the "ternary operator" in other languages,
/// but with a different syntax than C-style languages.
#[derive(Debug, Clone, Copy)]
pub struct TernaryExpr {
    /// The value expression to be returned if the condition is true
    pub value: NodeID,
    /// The condition expression that determines which value to return
    pub condition: NodeID,
    /// The alternative value expression to be returned if the condition is false
    pub else_value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl TernaryExpr {
    /// Creates a new ternary conditional expression.
    #[must_use]
    pub const fn new(
        value: NodeID,
        condition: NodeID,
        else_value: NodeID,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { value, condition, else_value, id, parent: None, span }
    }
}

impl ASTNode for TernaryExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    // Ternary is categorized as a BinaryExpression like Conditional
    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value, self.condition, self.else_value] }
}

impl_visitable!(TernaryExpr, visit_ternary_expr);

impl fmt::Display for TernaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Ternary(value-if-else)") }
}

// ============================================================================
// Unary Operations
// ============================================================================

/// Represents a unary operation in the AST (e.g. `-a`, `not b`).
#[derive(Debug, Clone, Copy)]
pub struct UnaryOpExpr {
    /// The operator type
    pub op: UnaryOpKind,
    /// The operand (expression)
    pub operand: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

/// Represents the type of unary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpKind {
    Pos,    // + (positive)
    Neg,    // - (negative)
    Not,    // not
    BitNot, // ~ (bitwise not)
}

impl UnaryOpExpr {
    /// Creates a new unary operation
    #[must_use]
    pub const fn new(op: UnaryOpKind, operand: NodeID, id: NodeID, span: Span) -> Self {
        Self { op, operand, id, parent: None, span }
    }
}

impl ASTNode for UnaryOpExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.operand] }
}

impl_visitable!(UnaryOpExpr, visit_unary_op_expr);

impl fmt::Display for UnaryOpExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Unary({:?})", self.op) }
}

// ============================================================================
// Variable Expressions
// ============================================================================

/// A variable reference in an expression
///
/// Represents a reference to a variable in an expression context.
/// This is kept separate from `BasicIdent` because it has a different `NodeKind`
/// (`Expression` instead of `Identifier`) to distinguish usage context.
#[derive(Debug, Clone)]
pub struct VariableExpr {
    /// The variable name
    pub name: String,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl VariableExpr {
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

impl ASTNode for VariableExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }
}

impl_visitable!(VariableExpr, visit_variable_expr);

impl fmt::Display for VariableExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.name) }
}

// ============================================================================
// Yield Expressions
// ============================================================================

/// Represents a yield expression in the AST (e.g. `yield value`).
///
/// Yield expressions are used in generator functions to produce values
/// one at a time, allowing the function to be paused and resumed.
#[derive(Debug, Clone, Copy)]
pub struct YieldExpr {
    /// The value being yielded (optional - can be None for bare `yield`)
    pub value: Option<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl YieldExpr {
    /// Creates a new yield expression
    #[must_use]
    pub const fn new(value: Option<NodeID>, id: NodeID, span: Span) -> Self {
        Self { value, id, parent: None, span }
    }
}

impl ASTNode for YieldExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.value.map_or_else(Vec::new, |v| vec![v]) }
}

impl_visitable!(YieldExpr, visit_yield_expr);

impl fmt::Display for YieldExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Yield") }
}

// ============================================================================
// Yield From Expressions
// ============================================================================

/// Represents a yield from expression in the AST (e.g. `yield from iterable`).
///
/// Yield from expressions delegate part of a generator's operations to another
/// generator, allowing for composition of generators.
#[derive(Debug, Clone, Copy)]
pub struct YieldFromExpr {
    /// The iterable being yielded from
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl YieldFromExpr {
    /// Creates a new yield from expression
    #[must_use]
    pub const fn new(value: NodeID, id: NodeID, span: Span) -> Self {
        Self { value, id, parent: None, span }
    }
}

impl ASTNode for YieldFromExpr {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Expression }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value] }
}

impl_visitable!(YieldFromExpr, visit_yield_from_expr);

impl fmt::Display for YieldFromExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "YieldFrom") }
}
