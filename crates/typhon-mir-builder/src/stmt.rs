//! Statement lowering
//!
//! This module handles lowering of AST statements to MIR.
//! Statements are lowered top-down: create blocks first, then fill with lowered code.

use typhon_ast::nodes::{
    AnyNode,
    AssignmentStmt,
    BreakStmt,
    ContinueStmt,
    ExpressionStmt,
    ForStmt,
    IfStmt,
    NodeID,
    ReturnStmt,
    WhileStmt,
};
use typhon_mir::instr::{MIRInstr, Terminator};
use typhon_source::types::Span;

use crate::context::{LoopContext, LoweringContext};
use crate::error::{LoweringError, LoweringResult};

impl LoweringContext<'_> {
    /// Lower an assignment statement
    fn lower_assignment(&mut self, assign: &AssignmentStmt) -> LoweringResult<()> {
        // Lower RHS first
        let value = self.lower_expr(assign.value)?;

        // Lower LHS - for now, only support simple variable assignments
        let target_node =
            self.ast().get_node(assign.target).ok_or_else(|| LoweringError::InternalError {
                message: format!("Target node not found: {}", assign.target),
                span: assign.span,
            })?;

        match &target_node.data {
            AnyNode::VariableExpr(var) => {
                // Simple variable assignment: x = value
                // Check if the variable already has a local, if not, allocate one
                let local = self.get_local(&var.name).unwrap_or_else(|_| {
                    // Allocate a new local for this variable
                    let ty = self.get_type(var.id);
                    let new_local = self.new_local(Some(var.name.clone()), ty, true);
                    self.register_local(var.name.clone(), new_local);

                    new_local
                });
                let _ = self.emit(MIRInstr::Store { local, value });

                Ok(())
            }
            AnyNode::AttributeExpr(attr) => {
                // Attribute assignment: obj.attr = value
                let object = self.lower_expr(attr.value)?;
                let _ = self.emit(MIRInstr::SetAttr { object, attr: attr.name.clone(), value });

                Ok(())
            }
            AnyNode::SubscriptionExpr(sub) => {
                // Subscript assignment: obj[key] = value
                let object = self.lower_expr(sub.value)?;
                let key = self.lower_expr(sub.index)?;
                let _ = self.emit(MIRInstr::SetItem { object, key, value });

                Ok(())
            }
            _ => Err(LoweringError::InvalidAssignmentTarget { span: assign.span }),
        }
    }

    /// Lower a break statement
    fn lower_break(&mut self, stmt: &BreakStmt) -> LoweringResult<()> {
        // Get the current loop context
        let loop_ctx =
            self.current_loop().map_err(|_| LoweringError::BreakOutsideLoop { span: stmt.span })?;

        // Branch to the break block
        let break_block = loop_ctx.break_block;
        let builder = self.current_function()?;
        builder.set_terminator(Terminator::Branch(break_block));

        Ok(())
    }

    /// Lower a continue statement
    fn lower_continue(&mut self, stmt: &ContinueStmt) -> LoweringResult<()> {
        // Get the current loop context
        let loop_ctx = self
            .current_loop()
            .map_err(|_| LoweringError::ContinueOutsideLoop { span: stmt.span })?;

        // Branch to the continue block
        let continue_block = loop_ctx.continue_block;
        let builder = self.current_function()?;
        builder.set_terminator(Terminator::Branch(continue_block));

        Ok(())
    }

    /// Lower an expression statement
    fn lower_expression_stmt(&mut self, stmt: &ExpressionStmt) -> LoweringResult<()> {
        // Lower the expression (its side effects are what matter)
        let _ = self.lower_expr(stmt.expression)?;

        Ok(())
    }

    /// Lower a for loop statement
    ///
    /// ## Implementation Status
    ///
    /// For loops require iterator protocol and exception handling infrastructure that is not yet
    /// implemented in the MIR layer. For loops would be desugared into while loops with iterator
    /// protocol:
    ///
    /// ```python
    /// for x in items:
    ///     body()
    /// ```
    ///
    /// Becomes (conceptually):
    ///
    /// ```python
    /// _iter = iter(items)
    /// while True:
    ///     try:
    ///         x = next(_iter)
    ///     except StopIteration:
    ///         break
    ///     body()
    /// ```
    ///
    /// ## Future Work
    ///
    /// - Implement iterator protocol (`__iter__` and `__next__` methods)
    /// - Add exception handling support (try/except lowering)
    /// - Implement `StopIteration` exception type
    /// - Add support for iterator unpacking in loop variables
    ///
    /// ## Errors
    ///
    /// Currently returns [`LoweringError::UnsupportedNode`] as the required infrastructure
    /// is not yet available.
    fn lower_for(&self, for_stmt: &ForStmt) -> LoweringResult<()> {
        // TODO: For loops are not yet supported - requires iterator protocol and exception handling
        Err(LoweringError::UnsupportedNode {
            node_kind: "For loop (requires iterator protocol and exception handling)".to_string(),
            span: for_stmt.span,
        })
    }

    /// Lower an if statement
    fn lower_if(&mut self, if_stmt: &IfStmt) -> LoweringResult<()> {
        // Lower condition
        let condition = self.lower_expr(if_stmt.condition)?;

        // Create blocks
        let then_block = self.new_block();
        let else_block = self.new_block();
        let continue_block = self.new_block();

        // Emit conditional branch
        let builder = self.current_function()?;
        builder.set_terminator(Terminator::CondBranch { condition, then_block, else_block });

        // Lower then branch
        builder.switch_to_block(then_block);
        for stmt_id in &if_stmt.body {
            self.lower_stmt(*stmt_id)?;
        }
        // If not already terminated, branch to continue
        if !self.is_current_block_terminated() {
            let builder = self.current_function()?;
            builder.set_terminator(Terminator::Branch(continue_block));
        }

        // Lower elif branches and else branch
        let builder = self.current_function()?;
        builder.switch_to_block(else_block);

        if !if_stmt.elif_branches.is_empty() {
            // Handle elif branches recursively
            for (elif_idx, (elif_cond, elif_body)) in if_stmt.elif_branches.iter().enumerate() {
                let elif_cond_val = self.lower_expr(*elif_cond)?;
                let elif_then = self.new_block();
                let next_elif_or_else = self.new_block();

                let builder = self.current_function()?;
                builder.set_terminator(Terminator::CondBranch {
                    condition: elif_cond_val,
                    then_block: elif_then,
                    else_block: next_elif_or_else,
                });

                // Lower elif body
                builder.switch_to_block(elif_then);
                for stmt_id in elif_body {
                    self.lower_stmt(*stmt_id)?;
                }

                if !self.is_current_block_terminated() {
                    let builder = self.current_function()?;
                    builder.set_terminator(Terminator::Branch(continue_block));
                }

                // Move to next elif or final else
                let builder = self.current_function()?;
                builder.switch_to_block(next_elif_or_else);

                // If this is the last elif and there's an else, lower it
                if elif_idx == if_stmt.elif_branches.len() - 1
                    && let Some(else_body) = &if_stmt.else_body
                {
                    for stmt_id in else_body {
                        self.lower_stmt(*stmt_id)?;
                    }
                }
            }
        } else if let Some(else_body) = &if_stmt.else_body {
            // No elif, just else
            for stmt_id in else_body {
                self.lower_stmt(*stmt_id)?;
            }
        }

        // If not already terminated, branch to continue
        if !self.is_current_block_terminated() {
            let builder = self.current_function()?;
            builder.set_terminator(Terminator::Branch(continue_block));
        }

        // Continue after if
        let builder = self.current_function()?;
        builder.switch_to_block(continue_block);

        Ok(())
    }

    /// Lower a return statement
    fn lower_return(&mut self, ret: &ReturnStmt) -> LoweringResult<()> {
        let value =
            if let Some(val_id) = ret.value { Some(self.lower_expr(val_id)?) } else { None };

        let builder = self.current_function()?;
        builder.set_terminator(Terminator::Return(value));

        Ok(())
    }

    /// Lower a statement node to MIR
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The node does not exist in the AST
    /// - The node is not a supported statement type
    /// - Lowering of the statement fails
    pub fn lower_stmt(&mut self, node_id: NodeID) -> LoweringResult<()> {
        // Get the node from the AST
        let node = self.ast().get_node(node_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Node not found: {node_id}"),
            span: Span::new(0, 0),
        })?;

        match &node.data {
            AnyNode::AssignmentStmt(stmt) => self.lower_assignment(stmt),
            AnyNode::BreakStmt(stmt) => self.lower_break(stmt),
            AnyNode::ContinueStmt(stmt) => self.lower_continue(stmt),
            AnyNode::ExpressionStmt(stmt) => self.lower_expression_stmt(stmt),
            AnyNode::ForStmt(stmt) => self.lower_for(stmt),
            // Nested function declaration - handle as a closure
            AnyNode::FunctionDecl(_) => self.lower_nested_function(node_id),
            AnyNode::IfStmt(stmt) => self.lower_if(stmt),
            AnyNode::ReturnStmt(stmt) => self.lower_return(stmt),
            AnyNode::WhileStmt(stmt) => self.lower_while(stmt),
            _ => Err(LoweringError::UnsupportedNode {
                node_kind: format!("{:?}", node.kind),
                span: node.span,
            }),
        }
    }

    /// Lower a while loop statement
    fn lower_while(&mut self, while_stmt: &WhileStmt) -> LoweringResult<()> {
        let header_block = self.new_block();
        let body_block = self.new_block();
        let exit_block = self.new_block();

        // Push loop context for break/continue
        self.push_loop_context(LoopContext {
            continue_block: header_block,
            break_block: exit_block,
        });

        // Branch to header
        let builder = self.current_function()?;
        builder.set_terminator(Terminator::Branch(header_block));

        // Lower header (condition)
        builder.switch_to_block(header_block);
        let condition = self.lower_expr(while_stmt.test)?;
        let builder = self.current_function()?;
        builder.set_terminator(Terminator::CondBranch {
            condition,
            then_block: body_block,
            else_block: exit_block,
        });

        // Lower body
        builder.switch_to_block(body_block);
        for stmt_id in &while_stmt.body {
            self.lower_stmt(*stmt_id)?;
        }
        // If not already terminated (e.g., by break), loop back to header
        if !self.is_current_block_terminated() {
            let builder = self.current_function()?;
            builder.set_terminator(Terminator::Branch(header_block));
        }

        // Pop loop context
        self.pop_loop_context();

        // Continue after loop
        let builder = self.current_function()?;
        builder.switch_to_block(exit_block);

        Ok(())
    }
}
