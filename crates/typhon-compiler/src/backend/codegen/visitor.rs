//! AST node visitor implementation.
//!
//! This module defines visitors for traversing and processing the AST nodes.

use super::context::CodeGenContext;
use super::symbol_table::SymbolTable;
use super::types::CodeGenValue;
use crate::backend::CodeGenError;
use crate::backend::error::CodeGenResult;
use crate::frontend::ast::{
    BinaryOperator,
    Expression,
    Identifier,
    Literal,
    Module,
    Statement,
    TypeExpression,
    UnaryOperator,
};

/// A visitor for AST nodes that produces LLVM IR.
pub trait NodeVisitor {
    /// Visit a binary operation.
    fn visit_binary_op(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> CodeGenResult<CodeGenValue>;

    /// Visit an expression node.
    fn visit_expression(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        expr: &Expression,
    ) -> CodeGenResult<CodeGenValue>;

    /// Visit a literal node.
    fn visit_literal(
        &mut self,
        context: &mut CodeGenContext,
        lit: &Literal,
    ) -> CodeGenResult<CodeGenValue>;

    /// Visit a module node.
    fn visit_module(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        module: &Module,
    ) -> CodeGenResult<()>;

    /// Visit a statement node.
    fn visit_statement(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        stmt: &Statement,
    ) -> CodeGenResult<CodeGenValue>;

    /// Visit a type expression node.
    fn visit_type_expression(
        &mut self,
        context: &mut CodeGenContext,
        type_expr: &TypeExpression,
    ) -> CodeGenResult<()>;

    /// Visit a unary operation.
    fn visit_unary_op(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> CodeGenResult<CodeGenValue>;

    /// Visit a variable reference.
    fn visit_variable(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        name: &Identifier,
    ) -> CodeGenResult<CodeGenValue>;
}

/// Default implementation of the NodeVisitor.
pub struct DefaultNodeVisitor;

impl NodeVisitor for DefaultNodeVisitor {
    fn visit_binary_op(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> CodeGenResult<CodeGenValue> {
        let context = &mut context.clone();

        // Evaluate left and right expressions
        let left_result = self.visit_expression(context, symbol_table, left)?;
        let left_value = left_result.as_basic_value_enum()?;
        let right_result = self.visit_expression(context, symbol_table, right)?;
        let right_value = right_result.as_basic_value_enum()?;

        // Use context to build the binary operation
        let result = context.build_binary_op(*op, left_value, right_value, "binop")?;

        Ok(CodeGenValue::new_basic(result))
    }

    fn visit_expression(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        expr: &Expression,
    ) -> CodeGenResult<CodeGenValue> {
        // Delegate to appropriate handlers based on expression type
        match expr {
            Expression::Literal { value, .. } => self.visit_literal(context, value),
            Expression::BinaryOp { left, op, right, .. } => {
                self.visit_binary_op(context, symbol_table, left, op, right)
            }
            Expression::UnaryOp { op, operand, .. } => {
                self.visit_unary_op(context, symbol_table, op, operand)
            }
            Expression::Variable { name, .. } => self.visit_variable(context, symbol_table, name),
            // Handle other expression types
            _ => Ok(CodeGenValue::Void),
        }
    }

    fn visit_literal(
        &mut self,
        context: &mut CodeGenContext,
        lit: &Literal,
    ) -> CodeGenResult<CodeGenValue> {
        // Simple delegation to context's literal handling
        context.visit_literal(lit, &Default::default())
    }

    fn visit_module(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        module: &Module,
    ) -> CodeGenResult<()> {
        // Process each statement in the module
        for stmt in &module.statements {
            self.visit_statement(context, symbol_table, stmt)?;
        }

        Ok(())
    }

    fn visit_statement(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        stmt: &Statement,
    ) -> CodeGenResult<CodeGenValue> {
        // Delegate to appropriate handlers based on statement type
        match stmt {
            Statement::FunctionDef { .. } => {
                // Function definition logic would go here
                Ok(CodeGenValue::Void)
            }
            Statement::Return { .. } => {
                // Return statement logic would go here
                Ok(CodeGenValue::Void)
            }
            Statement::VariableDecl { .. } => {
                // Variable declaration logic would go here
                Ok(CodeGenValue::Void)
            }
            Statement::Assignment { .. } => {
                // Assignment logic would go here
                Ok(CodeGenValue::Void)
            }
            Statement::Expression(expr) => {
                // Expression statement logic
                self.visit_expression(context, symbol_table, expr)
            }
            _ => {
                // Default case for other statement types
                Ok(CodeGenValue::Void)
            }
        }
    }

    fn visit_type_expression(
        &mut self,
        _context: &mut CodeGenContext,
        _type_expr: &TypeExpression,
    ) -> CodeGenResult<()> {
        // Type expressions don't generate code directly
        Ok(())
    }

    fn visit_unary_op(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> CodeGenResult<CodeGenValue> {
        // Evaluate operand
        let op_result = self.visit_expression(context, symbol_table, operand)?;
        let operand_value = op_result.as_basic_value_enum()?;

        // Use context to build the unary operation
        let result = context.build_unary_op(*op, operand_value, "unop")?;

        Ok(CodeGenValue::new_basic(result))
    }

    fn visit_variable(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        name: &Identifier,
    ) -> CodeGenResult<CodeGenValue> {
        // Look up the variable in the symbol table and load its value
        let entry = symbol_table.lookup(&name.name).ok_or_else(|| {
            CodeGenError::code_gen_error(
                format!("Undefined variable: {}", name.name),
                Some(name.source_info),
            )
        })?;

        // Load the variable value using the context
        let value = build_load(context, entry.value, &name.name);

        Ok(CodeGenValue::new_basic(value))
    }
}
