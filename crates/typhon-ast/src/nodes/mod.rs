//! AST node type definitions
//!
//! This module provides the consolidated AST node types organized in a flat structure.

mod collections;
mod declarations;
mod expressions;
mod identifiers;
mod modules;
mod patterns;
mod statements_control;
mod statements_core;
mod statements_error;
mod types;

use std::{fmt, process};

pub use collections::*;
pub use declarations::*;
pub use expressions::*;
pub use identifiers::*;
pub use modules::*;
pub use patterns::*;
pub use statements_control::*;
pub use statements_core::*;
pub use statements_error::*;
pub use types::*;
use typhon_source::types::Span;

/// A type-safe identifier for nodes in the AST arena.
///
/// `NodeID` is a handle that uniquely identifies a node in the arena. It includes
/// a generation counter to prevent use-after-free bugs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeID {
    /// The index of the node in the arena.
    pub(crate) index: u32,
    /// The generation counter for this node.
    pub(crate) generation: u32,
}

impl NodeID {
    /// Creates a new `NodeID` with the given index and generation.
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self { Self { index, generation } }

    /// Creates a placeholder `NodeID` for struct initialization.
    ///
    /// This is used when creating AST node structs that need an ID field
    /// before being allocated. The actual `NodeID` returned by `alloc_node()`
    /// should be used for references.
    #[must_use]
    pub const fn placeholder() -> Self { Self { index: 0, generation: 0 } }

    /// Returns the index of this node.
    #[must_use]
    pub const fn index(&self) -> u32 { self.index }

    /// Returns the generation of this node.
    #[must_use]
    pub const fn generation(&self) -> u32 { self.generation }
}

// Display implementation for NodeID
impl fmt::Display for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeID({}, {})", self.index, self.generation)
    }
}

impl process::Termination for NodeID {
    fn report(self) -> process::ExitCode { process::ExitCode::SUCCESS }
}

/// Main AST node type - discriminated union with all node variants
#[derive(Debug, Clone)]
pub enum AnyNode {
    /// Function argument (e.g. `name=value`)
    ArgumentExpr(ArgumentExpr),
    /// AS pattern (e.g. `case pattern as name:`)
    AsPattern(AsPattern),
    /// Assert statements
    AssertStmt(AssertStmt),
    /// Assignment expressions (e.g. `x = value`)
    AssignmentExpr(AssignmentExpr),
    /// Assignment statements
    AssignmentStmt(AssignmentStmt),
    /// Async function definitions
    AsyncFunctionDecl(AsyncFunctionDecl),
    /// Async for loops (e.g. `async for item in async_iterable:`)
    AsyncForStmt(AsyncForStmt),
    /// Async with statements (e.g. `async with ctx as var:`)
    AsyncWithStmt(AsyncWithStmt),
    /// Attribute access (e.g. `obj.attr`)
    AttributeExpr(AttributeExpr),
    /// Augmented assignment statements (e.g. `x += value`)
    AugmentedAssignmentStmt(AugmentedAssignmentStmt),
    /// Await expressions
    AwaitExpr(AwaitExpr),
    /// Binary operations (e.g. `a + b`)
    BinaryOpExpr(BinaryOpExpr),
    /// Break statements
    BreakStmt(BreakStmt),
    /// Function calls (e.g. `func(a, b)`)
    CallExpr(CallExpr),
    /// Callable type (e.g. `Callable[[int, str], bool]`)
    CallableType(CallableType),
    /// A case statement for a match block (e.g. `case [x, y, z, *rest]:`)
    MatchCase(MatchCase),
    /// Class definitions
    ClassDecl(ClassDecl),
    /// Class pattern (e.g. `case Point(x=0, y=0):`)
    ClassPattern(ClassPattern),
    /// Continue statements
    ContinueStmt(ContinueStmt),
    /// Delete statements (e.g. `del x, y, z`)
    DeleteStmt(DeleteStmt),
    /// Dictionary expressions (e.g. `{key: value, ...}`)
    DictExpr(DictExpr),
    /// Dictionary comprehension expressions (e.g. `{key: value for (key, value) in items}`)
    DictComprehensionExpr(DictComprehensionExpr),
    /// Exception handler (for try statements)
    ExceptHandler(ExceptHandler),
    /// Expression statements
    ExpressionStmt(ExpressionStmt),
    /// Format string expressions (e.g. `f"User {name}"`)
    FmtStringExpr(FmtStringExpr),
    /// For loops
    ForStmt(ForStmt),
    /// Global declarations (e.g. `global x, y, z`)
    GlobalStmt(GlobalStmt),
    /// From-import statement
    FromImportStmt(FromImportStmt),
    /// Function definitions
    FunctionDecl(FunctionDecl),
    /// Generator expressions (e.g. `(x for x in range(10))`)
    GeneratorExpr(GeneratorExpr),
    /// Generic type (e.g. `List[int]`)
    GenericType(GenericType),
    /// Grouping expressions (e.g. `(x + y) * z`)
    GroupingExpr(GroupingExpr),
    /// A unified identifier node (naming conventions determined by helper methods)
    BasicIdent(BasicIdent),
    /// Identifier pattern (e.g. `case x:`)
    IdentifierPattern(IdentifierPattern),
    /// If statements
    IfStmt(IfStmt),
    /// Import statements
    ImportStmt(ImportStmt),
    /// Lambda expressions (e.g. `lambda x: x+1`)
    LambdaExpr(LambdaExpr),
    /// List expressions (e.g. `[1, 2, 3]`)
    ListExpr(ListExpr),
    /// List comprehension expressions (e.g. `[value for value in items]`)
    ListComprehensionExpr(ListComprehensionExpr),
    /// Literal values (e.g. `42`, `"hello"`, `True`)
    LiteralExpr(LiteralExpr),
    /// Literal pattern (e.g. `case 42:`, `case "hello":`)
    LiteralPattern(LiteralPattern),
    /// Literal type (e.g. `Literal["red", "green", "blue"]`)
    LiteralType(LiteralType),
    /// Mapping pattern (e.g. `case {"key": value, **rest}:`)
    MappingPattern(MappingPattern),
    /// A match statement (e.g. `match x:`)
    MatchStmt(MatchStmt),
    /// Module node
    Module(Module),
    /// Nonlocal declarations (e.g. `nonlocal x, y, z`)
    NonlocalStmt(NonlocalStmt),
    /// OR pattern (e.g. `case 1 | 2 | 3:`)
    OrPattern(OrPattern),
    /// A parameter in a function or method.
    ParameterIdent(ParameterIdent),
    /// Pass statements
    PassStmt(PassStmt),
    /// Raise statements
    RaiseStmt(RaiseStmt),
    /// Return statements
    ReturnStmt(ReturnStmt),
    /// Sequence pattern (e.g. `case [a, b, *rest]:`)
    SequencePattern(SequencePattern),
    /// Set expressions (e.g. `{1, 2, 3}`)
    SetExpr(SetExpr),
    /// Set comprehension expressions (e.g. `{value for value in items}`)
    SetComprehensionExpr(SetComprehensionExpr),
    /// Slice expressions (e.g. `start:stop:step`)
    SliceExpr(SliceExpr),
    /// Starred expressions (e.g `*args`, `**kwargs`)
    StarredExpr(StarredExpr),
    /// Subscript operations (e.g. `arr[i]`)
    SubscriptionExpr(SubscriptionExpr),
    /// Template string expressions (e.g. `t"User {name}"`)
    TemplateStringExpr(TemplateStringExpr),
    /// Ternary expressions (e.g. `value if condition else other_value`)
    TernaryExpr(TernaryExpr),
    /// Try statements
    TryStmt(TryStmt),
    /// Tuple expressions (e.g. `(1, 2, 3)`)
    TupleExpr(TupleExpr),
    /// Tuple type (e.g. `tuple[int, str, float]`)
    TupleType(TupleType),
    /// Type definitions (aliases)
    TypeDecl(TypeDecl),
    /// Unary operations (e.g. `-a`, `not b`)
    UnaryOpExpr(UnaryOpExpr),
    /// Union type (e.g. `int | str`)
    UnionType(UnionType),
    /// Variable definitions
    VariableDecl(VariableDecl),
    /// A variable reference in an expression.
    VariableExpr(VariableExpr),
    /// While loops
    WhileStmt(WhileStmt),
    /// Wildcard pattern (e.g. `case _:`)
    WildcardPattern(WildcardPattern),
    /// With statements (e.g. `with open('file.txt') as f:`)
    WithStmt(WithStmt),
    /// Yield expressions
    YieldExpr(YieldExpr),
    /// Yield from expressions
    YieldFromExpr(YieldFromExpr),
}

// Generate get_as() method implementation using macro
for_each_node_variant!(impl_get_as_for_anynode);

/// A trait for AST nodes that can be stored in the arena.
pub trait ASTNode: fmt::Display {
    /// Returns the kind of this node.
    fn kind(&self) -> NodeKind;

    /// Returns the span of this node in the source code.
    fn span(&self) -> Span;

    /// Returns the children of this node.
    fn children(&self) -> Vec<NodeID> { vec![] }

    /// Returns the ID of this node.
    fn id(&self) -> NodeID;

    /// Returns the parent of this node, if it has one.
    fn parent(&self) -> Option<NodeID>;

    /// Builder pattern method to set the parent of this node.
    #[must_use]
    fn with_parent(self, parent: NodeID) -> Self;
}

// Generate ASTNode trait implementation using macro
for_each_node_variant!(impl_astnode_for_anynode);

// Generate Visitable trait implementation using macro
for_each_node_variant!(impl_visitable_for_anynode);

// Generate Display trait implementation using macro
for_each_node_variant!(impl_display_for_anynode);

/// High-level node categorization for quick filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Declaration, // Class, function, type, variable declarations
    Expression,  // All expressions
    Identifier,  // All identifier types
    Module,      // Top-level construct
    Pattern,     // Match patterns
    Statement,   // All statements
    Type,        // Type annotations
}

/// The node structure that contains common metadata and node-specific data
#[derive(Debug, Clone)]
pub struct Node {
    /// The kind of node
    pub kind: NodeKind,
    /// Node-specific data
    pub data: AnyNode,
    /// Source code span
    pub span: Span,
    /// Parent node reference
    pub parent: Option<NodeID>,
}
