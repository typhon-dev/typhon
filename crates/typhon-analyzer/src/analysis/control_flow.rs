//! Control flow graph construction and analysis.
//!
//! This module provides data structures and algorithms for building and analyzing
//! control flow graphs (CFGs) for functions. CFGs are used for definite assignment
//! checking, dead code detection, and other flow-sensitive analyses.

use rustc_hash::FxHashSet;
use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    BreakStmt,
    ContinueStmt,
    ForStmt,
    FunctionDecl,
    IfStmt,
    NodeID,
    ReturnStmt,
    WhileStmt,
};

/// Represents a basic block in a control flow graph.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier for this block
    pub id: usize,
    /// AST nodes (statements) in this block
    pub statements: Vec<NodeID>,
    /// IDs of successor blocks
    pub successors: Vec<usize>,
    /// IDs of predecessor blocks
    pub predecessors: Vec<usize>,
    /// Whether this block ends with a terminator (return, break, continue, raise)
    pub has_terminator: bool,
}

/// Control flow graph for a function.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// All basic blocks in the graph
    blocks: Vec<BasicBlock>,
    /// ID of the entry block
    entry_block: usize,
    /// IDs of exit blocks (blocks that end the function)
    exit_blocks: Vec<usize>,
    /// Set of reachable block IDs (computed lazily)
    reachable: Option<FxHashSet<usize>>,
}

impl ControlFlowGraph {
    /// Creates a new empty CFG.
    #[must_use]
    pub const fn new() -> Self {
        Self { blocks: Vec::new(), entry_block: 0, exit_blocks: Vec::new(), reachable: None }
    }

    /// Adds a new basic block and returns its ID.
    pub fn add_block(&mut self) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            statements: Vec::new(),
            successors: Vec::new(),
            predecessors: Vec::new(),
            has_terminator: false,
        });
        id
    }

    /// Adds an edge from one block to another.
    pub fn add_edge(&mut self, from: usize, to: usize) {
        if let Some(from_block) = self.blocks.get_mut(from)
            && !from_block.successors.contains(&to)
        {
            from_block.successors.push(to);
        }

        if let Some(to_block) = self.blocks.get_mut(to)
            && !to_block.predecessors.contains(&from)
        {
            to_block.predecessors.push(from);
        }
    }

    /// Returns true if all paths from entry reach an exit block.
    ///
    /// This is used to check if a function is guaranteed to return on all code paths.
    ///
    /// Algorithm:
    ///
    /// 1. A block is "complete" if:
    ///    - It has a return statement (is an exit block), OR
    ///    - ALL its successors are complete
    /// 2. Check if entry block is complete
    #[must_use]
    pub fn all_paths_reach_exit(&self) -> bool {
        // If no exit blocks, no paths return
        if self.exit_blocks.is_empty() {
            return false;
        }

        // Check if a block's all paths lead to returns
        // visited tracks blocks we're currently analyzing (for cycle detection)
        // complete_cache tracks blocks we've determined are complete
        let mut complete_cache: FxHashSet<usize> = FxHashSet::default();
        let mut visited: FxHashSet<usize> = FxHashSet::default();

        self.is_block_complete(self.entry_block, &mut visited, &mut complete_cache)
    }

    /// Gets all blocks.
    #[must_use]
    pub fn blocks(&self) -> &[BasicBlock] { &self.blocks }

    /// Computes and returns the set of reachable blocks from entry.
    ///
    /// Uses depth-first search to find all blocks reachable from the entry block.
    pub fn compute_reachable(&mut self) -> &FxHashSet<usize> {
        if self.reachable.is_none() {
            let mut reachable = FxHashSet::default();
            let mut stack = vec![self.entry_block];

            while let Some(block_id) = stack.pop() {
                if reachable.insert(block_id) {
                    // First time visiting this block - add successors
                    if let Some(block) = self.blocks.get(block_id) {
                        for &successor in &block.successors {
                            if !reachable.contains(&successor) {
                                stack.push(successor);
                            }
                        }
                    }
                }
            }

            self.reachable = Some(reachable);
        }

        // SAFETY: We ensure reachable is Some in the block above
        self.reachable.as_ref().unwrap_or_else(|| unreachable!())
    }

    /// Gets the entry block.
    #[must_use]
    pub fn entry_block(&self) -> &BasicBlock { &self.blocks[self.entry_block] }

    /// Gets exit blocks.
    #[must_use]
    pub fn exit_blocks(&self) -> &[usize] { &self.exit_blocks }

    /// Gets a block by ID.
    #[must_use]
    pub fn get_block(&self, id: usize) -> Option<&BasicBlock> { self.blocks.get(id) }

    /// Returns true if the given block is reachable from entry.
    pub fn is_reachable(&mut self, block_id: usize) -> bool {
        self.compute_reachable().contains(&block_id)
    }

    /// Inner helper that does the actual checking.
    fn check_block_complete(
        &self,
        block_id: usize,
        visited: &mut FxHashSet<usize>,
        complete_cache: &mut FxHashSet<usize>,
    ) -> bool {
        let Some(block) = self.blocks.get(block_id) else {
            return false;
        };

        // If this block is an exit block (has return), all paths from here are complete
        if self.exit_blocks.contains(&block_id) {
            return true;
        }

        // If block has terminator but isn't an exit, it's break/continue
        // which don't count as returns
        if block.has_terminator {
            return false;
        }

        // If no successors and not an exit block, path doesn't return
        if block.successors.is_empty() {
            return false;
        }

        // Check if ALL successors are complete
        for &successor in &block.successors {
            if !self.is_block_complete(successor, visited, complete_cache) {
                return false;
            }
        }

        true
    }

    /// Creates basic blocks for a loop structure.
    fn create_loop_blocks(&mut self, current_block: usize) -> (usize, usize, usize) {
        let loop_cond = self.add_block();
        let loop_body = self.add_block();
        let loop_exit = self.add_block();

        self.add_edge(current_block, loop_cond);
        self.add_edge(loop_cond, loop_body);
        self.add_edge(loop_cond, loop_exit);

        (loop_cond, loop_body, loop_exit)
    }

    /// Helper: checks if all paths from a block reach an exit (return statement).
    /// Uses memoization to avoid recomputing the same blocks.
    fn is_block_complete(
        &self,
        block_id: usize,
        visited: &mut FxHashSet<usize>,
        complete_cache: &mut FxHashSet<usize>,
    ) -> bool {
        // If already determined to be complete, return true
        if complete_cache.contains(&block_id) {
            return true;
        }

        // If we're visiting this block again (cycle), it's not complete
        // because we haven't hit a return yet
        if !visited.insert(block_id) {
            return false;
        }

        let result = self.check_block_complete(block_id, visited, complete_cache);

        // Remove from visited for other paths
        let _ = visited.remove(&block_id);

        // If complete, cache it
        if result {
            let _ = complete_cache.insert(block_id);
        }

        result
    }

    /// Processes a body of statements.
    fn process_body(
        &mut self,
        ast: &AST,
        body: &[NodeID],
        start_block: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        let mut current = start_block;
        for stmt in body {
            current = self.process_statement(ast, *stmt, current, loop_stack);
        }

        current
    }

    /// Processes a break statement.
    fn process_break(
        &mut self,
        stmt_id: NodeID,
        current_block: usize,
        loop_stack: &[(usize, usize)],
    ) -> usize {
        if let Some(block) = self.blocks.get_mut(current_block) {
            block.statements.push(stmt_id);
            block.has_terminator = true;
        }

        if let Some((_loop_cond, loop_exit)) = loop_stack.last() {
            self.add_edge(current_block, *loop_exit);
        }

        self.add_block()
    }

    /// Processes a continue statement.
    fn process_continue(
        &mut self,
        stmt_id: NodeID,
        current_block: usize,
        loop_stack: &[(usize, usize)],
    ) -> usize {
        if let Some(block) = self.blocks.get_mut(current_block) {
            block.statements.push(stmt_id);
            block.has_terminator = true;
        }

        if let Some((loop_cond, _loop_exit)) = loop_stack.last() {
            self.add_edge(current_block, *loop_cond);
        }

        self.add_block()
    }

    /// Processes an elif branch.
    fn process_elif(
        &mut self,
        ast: &AST,
        elif_cond: NodeID,
        elif_body: &[NodeID],
        prev_else_block: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> (usize, usize) {
        let elif_block = self.add_block();
        self.add_edge(prev_else_block, elif_block);

        if let Some(block) = self.blocks.get_mut(elif_block) {
            block.statements.push(elif_cond);
        }

        let elif_then = self.add_block();
        self.add_edge(elif_block, elif_then);
        let elif_exit = self.process_body(ast, elif_body, elif_then, loop_stack);

        (elif_exit, elif_block)
    }

    /// Processes else branch and creates merge block.
    fn process_else_and_merge(
        &mut self,
        ast: &AST,
        if_stmt: &IfStmt,
        prev_else_block: usize,
        elif_exit_blocks: Vec<usize>,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        let merge_block = self.add_block();

        if let Some(else_body) = &if_stmt.else_body {
            let else_block = self.add_block();
            self.add_edge(prev_else_block, else_block);
            let else_exit = self.process_body(ast, else_body, else_block, loop_stack);

            if let Some(block) = self.blocks.get(else_exit)
                && !block.has_terminator
            {
                self.add_edge(else_exit, merge_block);
            }
        } else {
            self.add_edge(prev_else_block, merge_block);
        }

        // Connect all branch exits to merge
        for exit in elif_exit_blocks {
            if let Some(block) = self.blocks.get(exit)
                && !block.has_terminator
            {
                self.add_edge(exit, merge_block);
            }
        }

        merge_block
    }

    /// Processes a for loop.
    fn process_for(
        &mut self,
        ast: &AST,
        for_stmt: &ForStmt,
        current_block: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        let (loop_cond, loop_body, loop_exit) = self.create_loop_blocks(current_block);

        if let Some(block) = self.blocks.get_mut(loop_cond) {
            block.statements.push(for_stmt.iter);
        }

        loop_stack.push((loop_cond, loop_exit));
        let body_exit = self.process_body(ast, &for_stmt.body, loop_body, loop_stack);

        if let Some(block) = self.blocks.get(body_exit)
            && !block.has_terminator
        {
            self.add_edge(body_exit, loop_cond);
        }

        self.process_loop_else(ast, for_stmt.else_body.as_ref(), loop_exit, loop_stack)
    }

    /// Processes an if statement with elif and else branches.
    fn process_if(
        &mut self,
        ast: &AST,
        stmt_id: NodeID,
        if_stmt: &IfStmt,
        current_block: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        if let Some(block) = self.blocks.get_mut(current_block) {
            block.statements.push(stmt_id);
        }

        // Process then branch
        let then_block = self.add_block();
        self.add_edge(current_block, then_block);
        let then_exit = self.process_body(ast, &if_stmt.body, then_block, loop_stack);

        // Process elif branches
        let mut elif_exit_blocks = vec![then_exit];
        let mut prev_else_block = current_block;

        for (elif_cond, elif_body) in &if_stmt.elif_branches {
            let (elif_exit, next_else_block) =
                self.process_elif(ast, *elif_cond, elif_body, prev_else_block, loop_stack);
            elif_exit_blocks.push(elif_exit);
            prev_else_block = next_else_block;
        }

        // Process else branch and create merge block
        self.process_else_and_merge(ast, if_stmt, prev_else_block, elif_exit_blocks, loop_stack)
    }

    /// Processes the else clause of a loop.
    fn process_loop_else(
        &mut self,
        ast: &AST,
        else_body: Option<&Vec<NodeID>>,
        loop_exit: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        let _ = loop_stack.pop();

        else_body.map_or(loop_exit, |else_stmts| {
            let else_block = self.add_block();
            self.add_edge(loop_exit, else_block);

            self.process_body(ast, else_stmts, else_block, loop_stack)
        })
    }

    /// Processes a return statement.
    fn process_return(&mut self, stmt_id: NodeID, current_block: usize) -> usize {
        if let Some(block) = self.blocks.get_mut(current_block) {
            block.statements.push(stmt_id);
            block.has_terminator = true;
        }

        self.exit_blocks.push(current_block);

        self.add_block()
    }

    /// Processes a single statement and updates the CFG accordingly.
    fn process_statement(
        &mut self,
        ast: &AST,
        stmt_id: NodeID,
        current_block: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        // Try to get statement as different types
        if ast.get_as::<ReturnStmt>(stmt_id).is_ok() {
            Self::process_return(self, stmt_id, current_block)
        } else if ast.get_as::<BreakStmt>(stmt_id).is_ok() {
            Self::process_break(self, stmt_id, current_block, loop_stack)
        } else if ast.get_as::<ContinueStmt>(stmt_id).is_ok() {
            Self::process_continue(self, stmt_id, current_block, loop_stack)
        } else if let Ok(if_stmt) = ast.get_as::<IfStmt>(stmt_id) {
            self.process_if(ast, stmt_id, if_stmt, current_block, loop_stack)
        } else if let Ok(while_stmt) = ast.get_as::<WhileStmt>(stmt_id) {
            self.process_while(ast, while_stmt, current_block, loop_stack)
        } else if let Ok(for_stmt) = ast.get_as::<ForStmt>(stmt_id) {
            self.process_for(ast, for_stmt, current_block, loop_stack)
        } else {
            // Regular statement - add to current block
            if let Some(block) = self.blocks.get_mut(current_block) {
                block.statements.push(stmt_id);
            }

            current_block
        }
    }

    /// Processes a while loop.
    fn process_while(
        &mut self,
        ast: &AST,
        while_stmt: &WhileStmt,
        current_block: usize,
        loop_stack: &mut Vec<(usize, usize)>,
    ) -> usize {
        let (loop_cond, loop_body, loop_exit) = self.create_loop_blocks(current_block);

        if let Some(block) = self.blocks.get_mut(loop_cond) {
            block.statements.push(while_stmt.test);
        }

        loop_stack.push((loop_cond, loop_exit));
        let body_exit = self.process_body(ast, &while_stmt.body, loop_body, loop_stack);

        if let Some(block) = self.blocks.get(body_exit)
            && !block.has_terminator
        {
            self.add_edge(body_exit, loop_cond);
        }

        self.process_loop_else(ast, while_stmt.else_body.as_ref(), loop_exit, loop_stack)
    }

    /// Builds a CFG from a function's AST.
    ///
    /// This method constructs a control flow graph by analyzing the function's body,
    /// creating basic blocks for sequential code, branches, and loops.
    pub fn build_from_function(ast: &AST, func_id: NodeID) -> Self {
        let mut cfg = Self::new();
        let entry_block = cfg.add_block();
        cfg.entry_block = entry_block;

        // Get function declaration
        let Ok(func) = ast.get_as::<FunctionDecl>(func_id) else {
            return cfg;
        };

        // Build CFG from function body
        let mut current_block = entry_block;
        let mut loop_stack: Vec<(usize, usize)> = Vec::new(); // (condition_block, after_block)

        for stmt_id in &func.body {
            current_block = cfg.process_statement(ast, *stmt_id, current_block, &mut loop_stack);
        }

        cfg
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self { Self::new() }
}
