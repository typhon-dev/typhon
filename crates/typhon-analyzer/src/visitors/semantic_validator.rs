//! Semantic validation visitor for context-dependent semantic rules.
//!
//! This visitor validates semantic rules that depend on context, such as:
//! - `break` and `continue` only in loops
//! - `return` only in functions
//! - Missing return statements in non-void functions

use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    ASTNode,
    BreakStmt,
    ContinueStmt,
    ForStmt,
    FunctionDecl,
    NodeID,
    NodeKind,
    ReturnStmt,
    WhileStmt,
};
use typhon_ast::visitor::{MutVisitor, VisitorError, VisitorResult};

use crate::analysis::{
    ControlFlowGraph,
    DeadCodeDetector,
    DeadCodeWarning,
    DefiniteAssignmentAnalyzer,
};
use crate::error::SemanticError;
use crate::symbol::SymbolTable;

/// Validation context tracking.
#[derive(Debug, Clone)]
struct ValidationContext {
    /// Current loop nesting depth (0 = not in loop)
    loop_depth: usize,
    /// Current function nesting depth (0 = not in function)
    function_depth: usize,
}

impl ValidationContext {
    /// Creates a new validation context.
    const fn new() -> Self { Self { loop_depth: 0, function_depth: 0 } }

    /// Enters a function context.
    const fn enter_function(&mut self) { self.function_depth += 1; }

    /// Enters a loop context.
    const fn enter_loop(&mut self) { self.loop_depth += 1; }

    /// Exits a function context.
    const fn exit_function(&mut self) {
        if self.function_depth > 0 {
            self.function_depth -= 1;
        }
    }

    /// Exits a loop context.
    const fn exit_loop(&mut self) {
        if self.loop_depth > 0 {
            self.loop_depth -= 1;
        }
    }

    /// Returns true if currently in a function.
    const fn in_function(&self) -> bool { self.function_depth > 0 }

    /// Returns true if currently in a loop.
    const fn in_loop(&self) -> bool { self.loop_depth > 0 }
}

/// Semantic validator visitor.
///
/// Validates context-dependent semantic rules during AST traversal.
#[derive(Debug)]
pub struct SemanticValidatorVisitor<'ast> {
    /// Reference to the AST
    ast: &'ast AST,
    /// Reference to the symbol table
    symbol_table: &'ast SymbolTable,
    /// Validation context
    context: ValidationContext,
    /// Collected errors
    errors: Vec<SemanticError>,
    /// Collected warnings
    warnings: Vec<DeadCodeWarning>,
}

impl<'ast> SemanticValidatorVisitor<'ast> {
    /// Creates a new semantic validator.
    #[must_use]
    pub const fn new(ast: &'ast AST, symbol_table: &'ast SymbolTable) -> Self {
        Self {
            ast,
            symbol_table,
            context: ValidationContext::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Consumes the validator and returns collected errors.
    #[must_use]
    pub fn into_errors(self) -> Vec<SemanticError> { self.errors }

    /// Consumes the validator and returns collected errors and warnings.
    #[must_use]
    pub fn into_results(self) -> (Vec<SemanticError>, Vec<DeadCodeWarning>) {
        (self.errors, self.warnings)
    }

    /// Validates a break statement.
    fn validate_break(&mut self, node_id: NodeID) {
        if !self.context.in_loop()
            && let Ok(break_stmt) = self.ast.get_as::<BreakStmt>(node_id)
        {
            self.errors.push(SemanticError::BreakOutsideLoop { span: break_stmt.span });
        }
    }

    /// Validates a continue statement.
    fn validate_continue(&mut self, node_id: NodeID) {
        if !self.context.in_loop()
            && let Ok(continue_stmt) = self.ast.get_as::<ContinueStmt>(node_id)
        {
            self.errors.push(SemanticError::ContinueOutsideLoop { span: continue_stmt.span });
        }
    }

    /// Validates a function's control flow.
    fn validate_function_returns(&mut self, func_id: NodeID, func: &FunctionDecl) {
        // Build CFG for the function
        let mut cfg = ControlFlowGraph::build_from_function(self.ast, func_id);

        // Check if function has a non-None return type
        let has_return_type = func.return_type.is_some();

        if has_return_type {
            // Check if all paths return
            let all_paths = cfg.all_paths_reach_exit();

            if !all_paths {
                self.errors.push(SemanticError::MissingReturn {
                    function_name: func.name.clone(),
                    span: func.span,
                });
            }
        }

        // Run definite assignment analysis on the CFG
        let mut def_assign = DefiniteAssignmentAnalyzer::new(&cfg);
        if let Err(errors) = def_assign.analyze(self.ast, func_id) {
            self.errors.extend(errors);
        }

        // Run dead code detection
        let mut detector = DeadCodeDetector::new(&mut cfg, self.symbol_table);
        if detector.analyze(self.ast) == Ok(()) {
            self.warnings.extend(detector.warnings().to_vec());
        }
    }

    /// Validates a return statement.
    fn validate_return(&mut self, node_id: NodeID) {
        if !self.context.in_function()
            && let Ok(return_stmt) = self.ast.get_as::<ReturnStmt>(node_id)
        {
            self.errors.push(SemanticError::ReturnOutsideFunction { span: return_stmt.span });
        }
    }

    /// Validates semantic rules for a module.
    ///
    /// This is the main entry point for semantic validation.
    ///
    /// ## Errors
    ///
    /// Returns semantic errors if validation fails. Warnings are collected but don't
    /// cause validation to fail.
    pub fn validate(
        ast: &'ast AST,
        symbol_table: &'ast SymbolTable,
        module_id: NodeID,
    ) -> Result<Vec<DeadCodeWarning>, Vec<SemanticError>> {
        let mut validator = Self::new(ast, symbol_table);

        // Visit the module to perform validation
        drop(validator.visit_module(module_id));

        let (errors, warnings) = validator.into_results();

        if errors.is_empty() { Ok(warnings) } else { Err(errors) }
    }
}

impl MutVisitor<()> for SemanticValidatorVisitor<'_> {
    fn visit(&mut self, node_id: NodeID) -> Option<()> {
        // Get node and dispatch based on kind
        let node = self.ast.get_node(node_id)?;

        match node.kind {
            NodeKind::Module => self.visit_module(node_id).ok(),
            NodeKind::Statement => {
                // Try specific statement types
                if self.visit_break_stmt(node_id).is_ok()
                    || self.visit_continue_stmt(node_id).is_ok()
                    || self.visit_return_stmt(node_id).is_ok()
                    || self.visit_for_stmt(node_id).is_ok()
                    || self.visit_while_stmt(node_id).is_ok()
                {
                    return Some(());
                }

                // For other statements, visit children
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }

                Some(())
            }

            NodeKind::Declaration => {
                // Try specific declaration types
                if self.visit_function_decl(node_id).is_ok() {
                    return Some(());
                }

                // For other declarations, visit children
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }

                Some(())
            }
            _ => {
                // Visit all children for other node types
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }

                Some(())
            }
        }
    }

    fn visit_break_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        // Only validate if this is actually a BreakStmt
        if self.ast.get_as::<BreakStmt>(node_id).is_ok() {
            self.validate_break(node_id);

            Ok(())
        } else {
            Err(VisitorError::Custom("Not a BreakStmt".to_string()))
        }
    }

    fn visit_continue_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        // Only validate if this is actually a ContinueStmt
        if self.ast.get_as::<ContinueStmt>(node_id).is_ok() {
            self.validate_continue(node_id);

            Ok(())
        } else {
            Err(VisitorError::Custom("Not a ContinueStmt".to_string()))
        }
    }

    fn visit_for_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        // Only process if this is actually a ForStmt
        if let Ok(for_stmt) = self.ast.get_as::<ForStmt>(node_id) {
            self.context.enter_loop();

            // Visit loop body
            for stmt_id in &for_stmt.body {
                let _ = self.visit(*stmt_id);
            }

            self.context.exit_loop();

            Ok(())
        } else {
            Err(VisitorError::Custom("Not a ForStmt".to_string()))
        }
    }

    fn visit_function_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let Ok(func) = self.ast.get_as::<FunctionDecl>(node_id) else {
            return Ok(());
        };

        // Validate return paths
        self.validate_function_returns(node_id, func);

        self.context.enter_function();

        // Visit function body
        for stmt_id in &func.body {
            let _ = self.visit(*stmt_id);
        }

        self.context.exit_function();

        Ok(())
    }

    fn visit_module(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let Some(module) = self.ast.get_node(node_id) else {
            return Ok(());
        };

        // Visit all children of the module
        for child_id in module.data.children() {
            let _ = self.visit(child_id);
        }

        Ok(())
    }

    fn visit_return_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        // Only validate if this is actually a ReturnStmt
        if self.ast.get_as::<ReturnStmt>(node_id).is_ok() {
            self.validate_return(node_id);

            Ok(())
        } else {
            Err(VisitorError::Custom("Not a ReturnStmt".to_string()))
        }
    }

    fn visit_while_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        // Only process if this is actually a WhileStmt
        if let Ok(while_stmt) = self.ast.get_as::<WhileStmt>(node_id) {
            self.context.enter_loop();

            // Visit loop body
            for stmt_id in &while_stmt.body {
                let _ = self.visit(*stmt_id);
            }

            self.context.exit_loop();

            Ok(())
        } else {
            Err(VisitorError::Custom("Not a WhileStmt".to_string()))
        }
    }
}
