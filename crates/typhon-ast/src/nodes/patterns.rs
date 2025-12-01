//! Pattern matching node types
//!
//! This file contains all pattern types for structural pattern matching.

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// Pattern Base Types
// ============================================================================

/// The kind of pattern matching pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    /// An AS pattern (e.g. `case pattern as name:`)
    As,
    /// A class pattern (e.g. `case Point(x=0, y=0):`)
    Class,
    /// An identifier pattern (e.g. `case x:`)
    Identifier,
    /// A literal pattern (e.g. `case 42:`, `case "string":`)
    Literal,
    /// A mapping pattern (e.g. `case {"key": value, **rest}:`)
    Mapping,
    /// An OR pattern (e.g. `case 1 | 2 | 3:`)
    Or,
    /// A sequence pattern (e.g. `case [a, b, *rest]:`)
    Sequence,
    /// A wildcard pattern (e.g. `case _:`)
    Wildcard,
}

/// Base trait for pattern matching patterns
pub trait Pattern {
    /// Returns the kind of pattern
    fn pattern_kind(&self) -> PatternKind;
}

// ============================================================================
// AsPattern
// ============================================================================

/// An AS pattern node for pattern matching (e.g. `case pattern as name:`)
///
/// This allows binding the matched value to a name while also matching against a pattern.
#[derive(Debug, Clone, Copy)]
pub struct AsPattern {
    /// The pattern to match against
    pub pattern: NodeID,
    /// The name to bind the matched value to
    pub name: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl AsPattern {
    /// Creates a new AS pattern
    #[must_use]
    pub const fn new(pattern: NodeID, name: NodeID, id: NodeID, span: Span) -> Self {
        Self { pattern, name, id, parent: None, span }
    }
}

impl ASTNode for AsPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.pattern, self.name] }
}

impl Pattern for AsPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::As }
}

impl_visitable!(AsPattern, visit_as_pattern);

impl fmt::Display for AsPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AsPattern(pattern: {}, name: {}, id: {})", self.pattern, self.name, self.id)
    }
}

// ============================================================================
// ClassPattern
// ============================================================================

/// A keyword argument in a class pattern
#[derive(Debug, Clone, Copy)]
pub struct ClassPatternKeyword {
    /// The keyword name
    pub name: NodeID,
    /// The pattern to match the keyword argument against
    pub pattern: NodeID,
}

impl ClassPatternKeyword {
    /// Creates a new class pattern keyword
    #[must_use]
    pub const fn new(name: NodeID, pattern: NodeID) -> Self { Self { name, pattern } }
}

/// A class pattern node for pattern matching (e.g. `case Point(x=0, y=0):`)
#[derive(Debug, Clone)]
pub struct ClassPattern {
    /// The class name to match against
    pub class_name: NodeID,
    /// Positional patterns
    pub patterns: Vec<NodeID>,
    /// Keyword patterns
    pub keywords: Vec<ClassPatternKeyword>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ClassPattern {
    /// Creates a new class pattern
    #[must_use]
    pub const fn new(
        class_name: NodeID,
        patterns: Vec<NodeID>,
        keywords: Vec<ClassPatternKeyword>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { class_name, patterns, keywords, id, parent: None, span }
    }
}

impl ASTNode for ClassPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = Vec::with_capacity(1 + self.patterns.len() + self.keywords.len() * 2);
        children.push(self.class_name);
        children.extend(self.patterns.clone());
        for keyword in &self.keywords {
            children.push(keyword.name);
            children.push(keyword.pattern);
        }
        children
    }
}

impl Pattern for ClassPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Class }
}

impl_visitable!(ClassPattern, visit_class_pattern);

impl fmt::Display for ClassPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClassPattern(class_name: {}, patterns: {}, keywords: {}, id: {})",
            self.class_name,
            self.patterns.len(),
            self.keywords.len(),
            self.id
        )
    }
}

// ============================================================================
// IdentifierPattern
// ============================================================================

/// An identifier pattern node for pattern matching (e.g. `case x:`)
#[derive(Debug, Clone, Copy)]
pub struct IdentifierPattern {
    /// The identifier to bind the value to
    pub name: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl IdentifierPattern {
    /// Creates a new identifier pattern
    #[must_use]
    pub const fn new(name: NodeID, id: NodeID, span: Span) -> Self {
        Self { name, id, parent: None, span }
    }
}

impl ASTNode for IdentifierPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.name] }
}

impl Pattern for IdentifierPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Identifier }
}

impl_visitable!(IdentifierPattern, visit_identifier_pattern);

impl fmt::Display for IdentifierPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IdentifierPattern(name: {}, id: {})", self.name, self.id)
    }
}

// ============================================================================
// LiteralPattern
// ============================================================================

/// A literal pattern node for pattern matching (e.g. `case 42:`, `case "hello":`)
#[derive(Debug, Clone, Copy)]
pub struct LiteralPattern {
    /// The literal value to match against
    pub value: NodeID,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl LiteralPattern {
    /// Creates a new literal pattern
    #[must_use]
    pub const fn new(value: NodeID, id: NodeID, span: Span) -> Self {
        Self { value, id, parent: None, span }
    }
}

impl ASTNode for LiteralPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![self.value] }
}

impl Pattern for LiteralPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Literal }
}

impl_visitable!(LiteralPattern, visit_literal_pattern);

impl fmt::Display for LiteralPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LiteralPattern(value: {}, id: {})", self.value, self.id)
    }
}

// ============================================================================
// MappingPattern
// ============================================================================

/// A key-value pair in a mapping pattern
#[derive(Debug, Clone, Copy)]
pub struct MappingPatternItem {
    /// The key expression
    pub key: NodeID,
    /// The value pattern
    pub value: NodeID,
}

impl MappingPatternItem {
    /// Creates a new mapping pattern item
    #[must_use]
    pub const fn new(key: NodeID, value: NodeID) -> Self { Self { key, value } }
}

/// A mapping pattern node for pattern matching (e.g. `case {"key": value, **rest}:`)
#[derive(Debug, Clone)]
pub struct MappingPattern {
    /// The key-value pairs to match against
    pub items: Vec<MappingPatternItem>,
    /// Optional double-starred pattern (e.g. `**rest`)
    pub starred: Option<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl MappingPattern {
    /// Creates a new mapping pattern
    #[must_use]
    pub const fn new(
        items: Vec<MappingPatternItem>,
        starred: Option<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { items, starred, id, parent: None, span }
    }
}

impl ASTNode for MappingPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children =
            Vec::with_capacity(self.items.len() * 2 + usize::from(self.starred.is_some()));
        for item in &self.items {
            children.push(item.key);
            children.push(item.value);
        }
        if let Some(starred) = self.starred {
            children.push(starred);
        }
        children
    }
}

impl Pattern for MappingPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Mapping }
}

impl_visitable!(MappingPattern, visit_mapping_pattern);

impl fmt::Display for MappingPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MappingPattern(items: {}, starred: {}, id: {})",
            self.items.len(),
            self.starred.map_or("None".to_string(), |s| s.to_string()),
            self.id
        )
    }
}

// ============================================================================
// MatchCase
// ============================================================================

/// A case statement within a match statement (e.g. `case pattern: body`)
#[derive(Debug, Clone)]
pub struct MatchCase {
    /// The pattern to match against
    pub pattern: NodeID,
    /// The optional guard expression (e.g. `case pattern if condition:`)
    pub guard: Option<NodeID>,
    /// The body statements to execute if the pattern matches
    pub body: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl MatchCase {
    /// Creates a new case statement
    #[must_use]
    pub const fn new(
        pattern: NodeID,
        guard: Option<NodeID>,
        body: Vec<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { pattern, guard, body, id, parent: None, span }
    }
}

impl ASTNode for MatchCase {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.pattern];
        if let Some(guard) = self.guard {
            children.push(guard);
        }
        children.extend(self.body.clone());
        children
    }
}

impl_visitable!(MatchCase, visit_match_case);

impl fmt::Display for MatchCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CaseStmt(pattern: {}, guard: {}, body: {}, id: {})",
            self.pattern,
            self.guard.map_or("None".to_string(), |g| g.to_string()),
            self.body.len(),
            self.id
        )
    }
}

// ============================================================================
// MatchStmt
// ============================================================================

/// A match statement in a program (e.g. `match value: case 1: ...`)
#[derive(Debug, Clone)]
pub struct MatchStmt {
    /// The value being matched against
    pub subject: NodeID,
    /// The cases to try to match
    pub cases: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl MatchStmt {
    /// Creates a new match statement
    #[must_use]
    pub const fn new(subject: NodeID, cases: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { subject, cases, id, parent: None, span }
    }
}

impl ASTNode for MatchStmt {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Statement }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = vec![self.subject];
        children.extend(self.cases.clone());
        children
    }
}

impl_visitable!(MatchStmt, visit_match_stmt);

impl fmt::Display for MatchStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MatchStmt(subject: {}, cases: {}, id: {})",
            self.subject,
            self.cases.len(),
            self.id
        )
    }
}

// ============================================================================
// OrPattern
// ============================================================================

/// An OR pattern node for pattern matching (e.g. `case 1 | 2 | 3:`)
#[derive(Debug, Clone)]
pub struct OrPattern {
    /// The alternative patterns
    pub patterns: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl OrPattern {
    /// Creates a new OR pattern
    #[must_use]
    pub const fn new(patterns: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { patterns, id, parent: None, span }
    }
}

impl ASTNode for OrPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.patterns.clone() }
}

impl Pattern for OrPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Or }
}

impl_visitable!(OrPattern, visit_or_pattern);

impl fmt::Display for OrPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OrPattern(patterns: {}, id: {})", self.patterns.len(), self.id)
    }
}

// ============================================================================
// SequencePattern
// ============================================================================

/// A sequence pattern node for pattern matching (e.g. `case [a, b, *rest]:`)
#[derive(Debug, Clone)]
pub struct SequencePattern {
    /// The patterns to match against the elements of the sequence
    pub patterns: Vec<NodeID>,
    /// Optional starred pattern (e.g. `*rest`)
    pub starred: Option<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl SequencePattern {
    /// Creates a new sequence pattern
    #[must_use]
    pub const fn new(
        patterns: Vec<NodeID>,
        starred: Option<NodeID>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { patterns, starred, id, parent: None, span }
    }
}

impl ASTNode for SequencePattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> {
        let mut children = self.patterns.clone();
        if let Some(starred) = self.starred {
            children.push(starred);
        }
        children
    }
}

impl Pattern for SequencePattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Sequence }
}

impl_visitable!(SequencePattern, visit_sequence_pattern);

impl fmt::Display for SequencePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SequencePattern(patterns: {}, starred: {}, id: {})",
            self.patterns.len(),
            self.starred.map_or("None".to_string(), |s| s.to_string()),
            self.id
        )
    }
}

// ============================================================================
// WildcardPattern
// ============================================================================

/// A wildcard pattern node for pattern matching (e.g. `case _:`)
#[derive(Debug, Clone, Copy)]
pub struct WildcardPattern {
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl WildcardPattern {
    /// Creates a new wildcard pattern
    #[must_use]
    pub const fn new(id: NodeID, span: Span) -> Self { Self { id, parent: None, span } }
}

impl ASTNode for WildcardPattern {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Pattern }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { vec![] }
}

impl Pattern for WildcardPattern {
    fn pattern_kind(&self) -> PatternKind { PatternKind::Wildcard }
}

impl_visitable!(WildcardPattern, visit_wildcard_pattern);

impl fmt::Display for WildcardPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WildcardPattern(id: {})", self.id)
    }
}
