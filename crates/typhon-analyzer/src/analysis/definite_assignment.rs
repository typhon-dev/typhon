//! Definite assignment analysis for variables.
//!
//! This module implements dataflow analysis to track which variables are definitely
//! assigned before use. This helps catch use-before-assignment errors.

use rustc_hash::{FxHashMap, FxHashSet};
use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    ASTNode,
    AssignmentExpr,
    AssignmentStmt,
    AugmentedAssignmentStmt,
    ForStmt,
    FunctionDecl,
    IfStmt,
    NodeID,
    NodeKind,
    ParameterIdent,
    VariableDecl,
    VariableExpr,
    WhileStmt,
};

use super::ControlFlowGraph;
use crate::error::SemanticError;
use crate::symbol::BUILTINS;

/// Tracks definitely-assigned variables through control flow.
#[derive(Debug)]
pub struct DefiniteAssignmentAnalyzer<'ctx> {
    /// The control flow graph
    cfg: &'ctx ControlFlowGraph,
    /// Definitely-assigned variables at the start of each block
    /// Map: `block_id` -> set of variable names
    block_in: FxHashMap<usize, FxHashSet<String>>,
    /// Definitely-assigned variables at the end of each block
    block_out: FxHashMap<usize, FxHashSet<String>>,
    /// Variables assigned in each block
    block_gen: FxHashMap<usize, FxHashSet<String>>,
    /// Collected errors
    errors: Vec<SemanticError>,
}

impl<'ctx> DefiniteAssignmentAnalyzer<'ctx> {
    /// Creates a new analyzer for the given CFG.
    #[must_use]
    pub fn new(cfg: &'ctx ControlFlowGraph) -> Self {
        Self {
            cfg,
            block_in: FxHashMap::default(),
            block_out: FxHashMap::default(),
            block_gen: FxHashMap::default(),
            errors: Vec::new(),
        }
    }

    /// Analyzes the CFG and returns any definite assignment errors.
    ///
    /// ## Errors
    ///
    /// Returns collected errors if any variables are used before assignment.
    pub fn analyze(&mut self, ast: &AST, func_id: NodeID) -> Result<(), Vec<SemanticError>> {
        // Initialize entry block with function parameters
        self.initialize_parameters(ast, func_id);

        // Collect for-loop targets and add them to appropriate blocks
        self.collect_loop_targets(ast, func_id);

        // Compute GEN sets for all blocks
        for block in self.cfg.blocks() {
            self.compute_gen_set(block.id, ast);
        }

        // Perform forward dataflow analysis
        self.compute_dataflow();

        // Validate all variable uses
        self.validate_uses(ast);

        // Return errors if any were collected
        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        Ok(())
    }

    /// Gets the collected errors.
    #[must_use]
    pub fn errors(&self) -> &[SemanticError] { &self.errors }

    /// Checks if a variable is definitely assigned at a given block.
    #[must_use]
    pub fn is_definitely_assigned(&self, var_name: &str, block_id: usize) -> bool {
        self.block_in.get(&block_id).is_some_and(|vars| vars.contains(var_name))
    }

    /// Initializes the entry block with function parameters and builtins as definitely assigned.
    fn initialize_parameters(&mut self, ast: &AST, func_id: NodeID) {
        if let Ok(func) = ast.get_as::<FunctionDecl>(func_id) {
            let mut assigned = FxHashSet::default();

            // Add function parameters - they are ALWAYS assigned when function is called
            // Parameters are ParameterIdent nodes (e.g., `x: int`)
            for param_id in &func.parameters {
                if let Ok(param) = ast.get_as::<ParameterIdent>(*param_id) {
                    let _ = assigned.insert(param.name.clone());
                }
            }

            // Add builtins that are always available
            for builtin in BUILTINS {
                let _ = assigned.insert((*builtin).to_string());
            }

            let entry_block = self.cfg.entry_block();
            drop(self.block_in.insert(entry_block.id, assigned));
        }
    }

    /// Checks for uses of variables in a statement, ensuring they're assigned.
    fn check_uses_in_statement(
        &mut self,
        node_id: NodeID,
        ast: &AST,
        assigned: &FxHashSet<String>,
    ) {
        self.check_uses_in_statement_impl(node_id, ast, assigned, None);
    }

    /// Internal implementation of `check_uses_in_statement` with `skip_node` support.
    ///
    /// The `skip_node` parameter allows skipping a specific node (used for for-loop targets).
    fn check_uses_in_statement_impl(
        &mut self,
        node_id: NodeID,
        ast: &AST,
        assigned: &FxHashSet<String>,
        skip_node: Option<NodeID>,
    ) {
        // Skip this node if it's in the skip list
        if skip_node == Some(node_id) {
            return;
        }

        let Some(node) = ast.get_node(node_id) else { return };

        match node.kind {
            NodeKind::Expression => {
                // Check for variable uses
                if let Ok(var_expr) = ast.get_as::<VariableExpr>(node_id)
                    && !assigned.contains(&var_expr.name)
                {
                    self.errors.push(SemanticError::UseBeforeAssignment {
                        name: var_expr.name.clone(),
                        span: var_expr.span,
                    });
                }

                // Visit children
                for child_id in node.data.children() {
                    self.check_uses_in_statement_impl(child_id, ast, assigned, skip_node);
                }
            }

            NodeKind::Statement => {
                // For assignments, check the value side but NOT the target
                if let Ok(assign) = ast.get_as::<AssignmentStmt>(node_id) {
                    // Only check the value expression, not the target
                    self.check_uses_in_statement_impl(assign.value, ast, assigned, skip_node);
                } else if let Ok(aug_assign) = ast.get_as::<AugmentedAssignmentStmt>(node_id) {
                    // For augmented assignments, check both target and value
                    self.check_uses_in_statement_impl(aug_assign.target, ast, assigned, skip_node);
                    self.check_uses_in_statement_impl(aug_assign.value, ast, assigned, skip_node);
                } else if let Ok(for_stmt) = ast.get_as::<ForStmt>(node_id) {
                    // For for-loops, check the iterable but NOT the target (it's assigned by the loop)
                    // Skip the target node when checking the iterable
                    self.check_uses_in_statement_impl(
                        for_stmt.iter,
                        ast,
                        assigned,
                        Some(for_stmt.target),
                    );

                    // Check the loop body with the target variable now assigned
                    let mut loop_assigned = assigned.clone();
                    Self::collect_assignment_target(for_stmt.target, ast, &mut loop_assigned);

                    for &body_stmt in &for_stmt.body {
                        self.check_uses_in_statement_impl(body_stmt, ast, &loop_assigned, None);
                    }

                    // Check else_body clause with original assigned set (target not assigned if loop didn't run)
                    if let Some(else_body) = &for_stmt.else_body {
                        for &else_stmt in else_body {
                            self.check_uses_in_statement_impl(else_stmt, ast, assigned, None);
                        }
                    }
                } else {
                    // Visit children for other statements
                    for child_id in node.data.children() {
                        self.check_uses_in_statement_impl(child_id, ast, assigned, skip_node);
                    }
                }
            }

            _ => {
                // Visit children
                for child_id in node.data.children() {
                    self.check_uses_in_statement_impl(child_id, ast, assigned, skip_node);
                }
            }
        }
    }

    /// Collects variable names from an assignment target.
    fn collect_assignment_target(
        target_id: NodeID,
        ast: &AST,
        assignments: &mut FxHashSet<String>,
    ) {
        let Some(node) = ast.get_node(target_id) else { return };

        match node.kind {
            NodeKind::Identifier | NodeKind::Expression => {
                // Try to get as VariableExpr
                if let Ok(var_expr) = ast.get_as::<VariableExpr>(target_id) {
                    let _ = assignments.insert(var_expr.name.clone());
                }

                // Also check children (e.g., tuple unpacking)
                for child_id in node.data.children() {
                    Self::collect_assignment_target(child_id, ast, assignments);
                }
            }
            _ => {
                // Check children for nested targets
                for child_id in node.data.children() {
                    Self::collect_assignment_target(child_id, ast, assignments);
                }
            }
        }
    }

    /// Recursively collects all assignments in a statement and its children.
    fn collect_assignments(node_id: NodeID, ast: &AST, assignments: &mut FxHashSet<String>) {
        let Some(node) = ast.get_node(node_id) else { return };

        match node.kind {
            NodeKind::Declaration => {
                // Check if it's a VariableDecl (annotated assignment like `x: int = 5`)
                if let Ok(var_decl) = ast.get_as::<VariableDecl>(node_id) {
                    // Only mark as assigned if the variable has an initial value
                    // e.g., `x: int = 5` marks x as assigned
                    // but `x: int` (without value) does NOT mark x as assigned
                    if var_decl.value.is_some() {
                        let _ = assignments.insert(var_decl.name.clone());
                    }
                }

                // Visit children for nested declarations
                for child_id in node.data.children() {
                    Self::collect_assignments(child_id, ast, assignments);
                }
            }
            NodeKind::Statement => {
                // Try specific statement types that assign variables
                if let Ok(assign) = ast.get_as::<AssignmentStmt>(node_id) {
                    Self::collect_assignment_target(assign.target, ast, assignments);
                } else if let Ok(aug_assign) = ast.get_as::<AugmentedAssignmentStmt>(node_id) {
                    Self::collect_assignment_target(aug_assign.target, ast, assignments);
                } else if let Ok(for_stmt) = ast.get_as::<ForStmt>(node_id) {
                    // For loop target is assigned
                    Self::collect_assignment_target(for_stmt.target, ast, assignments);
                } else if ast.get_as::<IfStmt>(node_id).is_ok()
                    || ast.get_as::<WhileStmt>(node_id).is_ok()
                    || ast.get_as::<ForStmt>(node_id).is_ok()
                {
                    // Control flow statements: DON'T recursively visit children
                    // Their bodies are in separate blocks, so assignments there
                    // should NOT be in this block's GEN set
                    return;
                }

                // Visit children for nested statements (but not for control flow)
                for child_id in node.data.children() {
                    Self::collect_assignments(child_id, ast, assignments);
                }
            }
            NodeKind::Expression => {
                // Assignment expressions also assign variables
                if let Ok(assign_expr) = ast.get_as::<AssignmentExpr>(node_id) {
                    Self::collect_assignment_target(assign_expr.target, ast, assignments);
                }

                // Visit children
                for child_id in node.data.children() {
                    Self::collect_assignments(child_id, ast, assignments);
                }
            }
            _ => {
                // Visit children
                for child_id in node.data.children() {
                    Self::collect_assignments(child_id, ast, assignments);
                }
            }
        }
    }

    /// Performs forward dataflow analysis to compute IN/OUT sets.
    fn compute_dataflow(&mut self) {
        // Note: IN[entry] is already initialized with function parameters
        // by initialize_parameters(), so we preserve it during iteration

        // Iterate until convergence
        let mut changed = true;
        let entry_id = self.cfg.entry_block().id;

        while changed {
            changed = false;

            for block in self.cfg.blocks() {
                // For entry block, preserve the initialized IN set (parameters)
                // For other blocks, compute IN[B] = ∩ OUT[P] for all predecessors P
                // (intersection means variable is assigned in ALL paths)
                let in_set = if block.id == entry_id {
                    // Entry block: keep initial parameters
                    self.block_in.get(&block.id).cloned().unwrap_or_default()
                } else if block.predecessors.is_empty() {
                    // No predecessors: empty set
                    FxHashSet::default()
                } else {
                    // Non-entry blocks with predecessors: intersection of predecessor OUT sets
                    let mut in_set: Option<FxHashSet<String>> = None;
                    for &pred_id in &block.predecessors {
                        if let Some(pred_out) = self.block_out.get(&pred_id) {
                            if let Some(ref mut current) = in_set {
                                // Intersection: keep only variables in both sets
                                current.retain(|var| pred_out.contains(var));
                            } else {
                                // First predecessor: initialize with its OUT set
                                in_set = Some(pred_out.clone());
                            }
                        }
                    }

                    in_set.unwrap_or_default()
                };

                // Update IN[B] if changed (only for non-entry blocks)
                if block.id != entry_id {
                    let old_in = self.block_in.get(&block.id).cloned().unwrap_or_default();
                    if in_set != old_in {
                        drop(self.block_in.insert(block.id, in_set.clone()));
                        changed = true;
                    }
                }

                // Compute OUT[B] = IN[B] ∪ GEN[B]
                let gen_set = self.block_gen.get(&block.id).cloned().unwrap_or_default();
                let mut out_set = in_set;
                out_set.extend(gen_set);

                // Update OUT[B] if changed
                let old_out = self.block_out.get(&block.id).cloned().unwrap_or_default();
                if out_set != old_out {
                    drop(self.block_out.insert(block.id, out_set));
                    changed = true;
                }
            }
        }
    }

    /// Collects for-loop targets and adds them to the GEN sets of loop condition blocks.
    /// This is called before `compute_gen_set` to ensure loop variables are treated as assigned.
    fn collect_loop_targets(&mut self, ast: &AST, func_id: NodeID) {
        if let Ok(func) = ast.get_as::<FunctionDecl>(func_id) {
            for stmt_id in &func.body {
                self.collect_loop_targets_from_stmt(*stmt_id, ast);
            }
        }
    }

    /// Recursively collects for-loop targets from a statement tree.
    fn collect_loop_targets_from_stmt(&mut self, stmt_id: NodeID, ast: &AST) {
        if let Ok(for_stmt) = ast.get_as::<ForStmt>(stmt_id) {
            // Find the block containing this for-loop's iterable
            // and add the target to that block's GEN set
            let mut target_block_id = None;
            for block in self.cfg.blocks() {
                if block.statements.contains(&for_stmt.iter) {
                    target_block_id = Some(block.id);

                    break;
                }
            }

            // Collect target variables and insert into block_gen
            if let Some(block_id) = target_block_id {
                let mut temp_vars = FxHashSet::default();
                Self::collect_assignment_target(for_stmt.target, ast, &mut temp_vars);
                self.block_gen.entry(block_id).or_default().extend(temp_vars);
            }

            // Recursively process loop body
            for &body_stmt in &for_stmt.body {
                self.collect_loop_targets_from_stmt(body_stmt, ast);
            }

            // Process else clause if present
            if let Some(else_body) = &for_stmt.else_body {
                for &else_stmt in else_body {
                    self.collect_loop_targets_from_stmt(else_stmt, ast);
                }
            }
        } else {
            // Check if this is an if statement or other control flow
            let Some(node) = ast.get_node(stmt_id) else { return };
            for child_id in node.data.children() {
                self.collect_loop_targets_from_stmt(child_id, ast);
            }
        }
    }

    /// Computes the GEN set for a block (variables assigned in the block).
    fn compute_gen_set(&mut self, block_id: usize, ast: &AST) {
        let mut gen_set = self.block_gen.get(&block_id).cloned().unwrap_or_default();

        if let Some(block) = self.cfg.get_block(block_id) {
            for &stmt_id in &block.statements {
                Self::collect_assignments(stmt_id, ast, &mut gen_set);
            }
        }

        drop(self.block_gen.insert(block_id, gen_set));
    }

    /// Validates that all variable uses have prior assignments.
    fn validate_uses(&mut self, ast: &AST) {
        for block in self.cfg.blocks() {
            let mut assigned = self.block_in.get(&block.id).cloned().unwrap_or_default();

            for &stmt_id in &block.statements {
                // Check uses before processing assignments in this statement
                self.check_uses_in_statement(stmt_id, ast, &assigned);

                // Now collect assignments from this statement for subsequent statements
                Self::collect_assignments(stmt_id, ast, &mut assigned);
            }
        }
    }
}
