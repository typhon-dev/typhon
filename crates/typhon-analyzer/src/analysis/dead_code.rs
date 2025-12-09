//! Dead code detection and code quality analysis.
//!
//! This module provides functionality for detecting unreachable code, unused variables,
//! and other code quality issues. These are reported as warnings rather than errors.

use rustc_hash::{FxHashMap, FxHashSet};
use typhon_ast::ast::AST;
use typhon_ast::nodes::{ASTNode, NodeID, NodeKind, VariableDecl, VariableExpr};
use typhon_source::types::Span;

use super::ControlFlowGraph;
use crate::symbol::SymbolTable;

/// Warning severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    /// Informational warning
    Info,
    /// Standard warning
    Warning,
}

/// A dead code warning.
#[derive(Debug, Clone)]
pub struct DeadCodeWarning {
    /// Warning message
    pub message: String,
    /// Source location
    pub span: Span,
    /// Severity level
    pub severity: WarningSeverity,
}

/// Detects dead code and code quality issues.
#[derive(Debug)]
pub struct DeadCodeDetector<'ctx> {
    /// Control flow graph
    cfg: &'ctx mut ControlFlowGraph,
    /// Symbol table (reserved for future use)
    _symbol_table: &'ctx SymbolTable,
    /// Collected warnings
    warnings: Vec<DeadCodeWarning>,
}

impl<'ctx> DeadCodeDetector<'ctx> {
    /// Creates a new dead code detector.
    #[must_use]
    pub const fn new(cfg: &'ctx mut ControlFlowGraph, symbol_table: &'ctx SymbolTable) -> Self {
        Self { cfg, _symbol_table: symbol_table, warnings: Vec::new() }
    }

    /// Performs dead code analysis.
    ///
    /// # Errors
    ///
    /// Returns an error if the analysis cannot be completed (though currently
    /// this always succeeds as warnings are collected).
    pub fn analyze(&mut self, ast: &AST) -> Result<(), String> {
        // Compute reachability first
        let reachable = self.cfg.compute_reachable().clone();

        self.detect_unreachable_blocks(ast, &reachable);
        self.detect_unused_variables(ast);
        Ok(())
    }

    /// Gets the collected warnings.
    #[must_use]
    pub fn warnings(&self) -> &[DeadCodeWarning] { &self.warnings }

    /// Detects unreachable basic blocks in the CFG.
    fn detect_unreachable_blocks(&mut self, ast: &AST, reachable: &FxHashSet<usize>) {
        for block in self.cfg.blocks() {
            if !reachable.contains(&block.id) && !block.statements.is_empty() {
                // Get the span of the first statement in the unreachable block
                if let Some(&first_stmt) = block.statements.first()
                    && let Some(node) = ast.get_node(first_stmt)
                {
                    self.warnings.push(DeadCodeWarning {
                        message: format!("Unreachable code in block {}", block.id),
                        span: node.span,
                        severity: WarningSeverity::Warning,
                    });
                }
            }
        }
    }

    /// Detects unused variables.
    fn detect_unused_variables(&mut self, ast: &AST) {
        let mut declared_vars: FxHashMap<String, Span> = FxHashMap::default();
        let mut used_vars: FxHashSet<String> = FxHashSet::default();

        // Collect all variable declarations
        for block in self.cfg.blocks() {
            for &stmt_id in &block.statements {
                Self::find_variable_declarations(stmt_id, ast, &mut declared_vars);
            }
        }

        // Collect all variable uses
        for block in self.cfg.blocks() {
            for &stmt_id in &block.statements {
                Self::collect_variable_uses(stmt_id, ast, &mut used_vars);
            }
        }

        // Report unused variables (excluding those starting with _)
        for (var_name, span) in &declared_vars {
            if !used_vars.contains(var_name) && !var_name.starts_with('_') {
                self.warnings.push(DeadCodeWarning {
                    message: format!("Unused variable '{var_name}'"),
                    span: *span,
                    severity: WarningSeverity::Info,
                });
            }
        }
    }

    /// Collects all variable uses in a node and its children.
    fn collect_variable_uses(node_id: NodeID, ast: &AST, uses: &mut FxHashSet<String>) {
        let Some(node) = ast.get_node(node_id) else { return };

        // Check if this is a variable use
        if node.kind == NodeKind::Expression
            && let Ok(var_expr) = ast.get_as::<VariableExpr>(node_id)
        {
            let _ = uses.insert(var_expr.name.clone());
        }

        // Recursively check children
        for child_id in node.data.children() {
            Self::collect_variable_uses(child_id, ast, uses);
        }
    }

    /// Recursively finds variable declarations in a node and its children.
    fn find_variable_declarations(
        node_id: NodeID,
        ast: &AST,
        declarations: &mut FxHashMap<String, Span>,
    ) {
        let Some(node) = ast.get_node(node_id) else { return };

        // Check if this is a variable declaration
        if node.kind == NodeKind::Declaration
            && let Ok(var_decl) = ast.get_as::<VariableDecl>(node_id)
        {
            let _ = declarations.insert(var_decl.name.clone(), var_decl.span);
        }

        // Recursively check children
        for child_id in node.data.children() {
            Self::find_variable_declarations(child_id, ast, declarations);
        }
    }
}
