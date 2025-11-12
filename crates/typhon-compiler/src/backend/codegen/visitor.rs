// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/visitor.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! AST node visitor implementation.
//!
//! This module defines visitors for traversing and processing the AST nodes.

use super::context::CodeGenContext;
use super::functions::build_load;
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
pub trait NodeVisitor<'ctx> {
    /// Visit a binary operation.
    fn visit_binary_op(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Visit an expression node.
    fn visit_expression(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        expr: &Expression,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Visit a literal node.
    fn visit_literal(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        lit: &Literal,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Visit a module node.
    fn visit_module(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        module: &Module,
    ) -> CodeGenResult<()>;

    /// Visit a statement node.
    fn visit_statement(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        stmt: &Statement,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Visit a type expression node.
    fn visit_type_expression(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        type_expr: &TypeExpression,
    ) -> CodeGenResult<()>;

    /// Visit a unary operation.
    fn visit_unary_op(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Visit a variable reference.
    fn visit_variable(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        name: &Identifier,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;
}

/// Default implementation of the NodeVisitor.
pub struct DefaultNodeVisitor;

impl<'ctx> NodeVisitor<'ctx> for DefaultNodeVisitor {
    fn visit_binary_op(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Evaluate left and right expressions
        let left_value = self.visit_expression(context, symbol_table, left)?.as_basic_value()?;
        let right_value = self.visit_expression(context, symbol_table, right)?.as_basic_value()?;
        // Use context to build the binary operation
        let result = context.build_binary_op(*op, left_value, right_value, "binop")?;

        Ok(CodeGenValue::new_basic(result))
    }

    fn visit_expression(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        expr: &Expression,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
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
        context: &mut CodeGenContext<'ctx>,
        lit: &Literal,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Simple delegation to context's literal handling
        context.visit_literal(lit, &Default::default())
    }

    fn visit_module(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
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
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        stmt: &Statement,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
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
        _context: &mut CodeGenContext<'ctx>,
        _type_expr: &TypeExpression,
    ) -> CodeGenResult<()> {
        // Type expressions don't generate code directly
        Ok(())
    }

    fn visit_unary_op(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Evaluate operand
        let operand_value =
            self.visit_expression(context, symbol_table, operand)?.as_basic_value()?;
        // Use context to build the unary operation
        let result = context.build_unary_op(*op, operand_value, "unop")?;

        Ok(CodeGenValue::new_basic(result))
    }

    fn visit_variable(
        &mut self,
        context: &mut CodeGenContext<'ctx>,
        symbol_table: &mut SymbolTable<'ctx>,
        name: &Identifier,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
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
