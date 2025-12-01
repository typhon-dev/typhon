//! Module and import node types for the AST.
//!
//! Contains: `FromImport`, `Import`, `Module`

use std::fmt;

use typhon_source::types::Span;

use super::{ASTNode, NodeID, NodeKind};

// ============================================================================
// From Import Statements
// ============================================================================

/// `FromImport` statement node in the AST (e.g. `from module import name1, name2 as alias`)
#[derive(Debug, Clone)]
pub struct FromImportStmt {
    /// Module being imported from (as a dotted name)
    pub module_parts: Vec<String>,
    /// Names being imported (name, alias)
    pub names: Vec<(String, Option<String>)>,
    /// Level of relative import (0 for absolute, >0 for relative imports with dots)
    pub level: usize,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl FromImportStmt {
    /// Creates a new from-import statement
    #[must_use]
    pub const fn new(
        module_parts: Vec<String>,
        names: Vec<(String, Option<String>)>,
        level: usize,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { module_parts, names, level, id, parent: None, span }
    }
}

impl ASTNode for FromImportStmt {
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

impl_visitable!(FromImportStmt, visit_from_import_stmt);

impl fmt::Display for FromImportStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let module = self.module_parts.join(".");

        let imports = self
            .names
            .iter()
            .map(|(name, alias)| {
                alias.as_ref().map_or_else(|| name.clone(), |alias| format!("{name} as {alias}"))
            })
            .collect::<Vec<_>>()
            .join(", ");

        let prefix = if self.level > 0 { ".".repeat(self.level) } else { String::new() };

        write!(f, "from {prefix}{module} import {imports}")
    }
}

// ============================================================================
// Import Statement
// ============================================================================

/// Import statement (e.g. `import module [as name]`).
#[derive(Debug, Clone)]
pub struct ImportStmt {
    /// The module name or path components
    pub module_parts: Vec<String>,
    /// Optional alias for the imported module
    pub alias: Option<String>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl ImportStmt {
    /// Creates a new import statement
    #[must_use]
    pub const fn new(
        module_parts: Vec<String>,
        alias: Option<String>,
        id: NodeID,
        span: Span,
    ) -> Self {
        Self { module_parts, alias, id, parent: None, span }
    }
}

impl ASTNode for ImportStmt {
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

impl_visitable!(ImportStmt, visit_import_stmt);

impl fmt::Display for ImportStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let module = self.module_parts.join(".");
        match &self.alias {
            Some(alias) => write!(f, "Import({module} as {alias})"),
            None => write!(f, "Import({module})"),
        }
    }
}

// ============================================================================
// Module - Top-level module representing a source file
// ============================================================================

/// Module node in the AST
///
/// Represents an entire source file with its statements.
#[derive(Debug, Clone)]
pub struct Module {
    /// The name of the module (typically the filename without extension)
    pub name: String,
    /// The statements in the module
    pub statements: Vec<NodeID>,
    /// The ID of this node in the AST arena
    pub id: NodeID,
    /// The ID of the parent node in the AST arena (if any)
    pub parent: Option<NodeID>,
    /// The span of this node in the source code
    pub span: Span,
}

impl Module {
    /// Creates a new module
    #[must_use]
    pub const fn new(name: String, statements: Vec<NodeID>, id: NodeID, span: Span) -> Self {
        Self { name, statements, id, parent: None, span }
    }
}

impl ASTNode for Module {
    fn id(&self) -> NodeID { self.id }

    fn parent(&self) -> Option<NodeID> { self.parent }

    fn with_parent(mut self, parent: NodeID) -> Self {
        self.parent = Some(parent);
        self
    }

    fn kind(&self) -> NodeKind { NodeKind::Module }

    fn span(&self) -> Span { self.span }

    fn children(&self) -> Vec<NodeID> { self.statements.clone() }
}

impl_visitable!(Module, visit_module);

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Module({})", self.name) }
}
