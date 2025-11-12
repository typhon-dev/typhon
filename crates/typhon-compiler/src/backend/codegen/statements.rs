// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/statements.rs
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
//! This module handles statement code generation.

use std::rc::Rc;

use super::context::CodeGenContext;
use super::expressions::CodeGenExpressions;
use super::functions::{build_store, create_entry_block_alloca};
use super::symbol_table::{SymbolEntry, SymbolTable};
use super::types::CodeGenValue;
use crate::backend::error::{CodeGenError, CodeGenResult};
use crate::frontend::ast::{Expression, Identifier, Statement, TypeExpression};
use crate::typesystem::types::Type;

/// Extension trait for statement operations on CodeGenContext
pub trait CodeGenStatements<'ctx> {
    /// Visit a statement and generate LLVM IR for it.
    fn visit_statement(
        &mut self,
        stmt: &Statement,
        symbol_table: &mut SymbolTable<'ctx>,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Complete variable declaration statement.
    fn complete_variable_decl_stmt(
        &mut self,
        name: &Identifier,
        type_annotation: &Option<TypeExpression>,
        value: &Option<Box<Expression>>,
        mutable: bool,
        symbol_table: &mut SymbolTable<'ctx>,
    ) -> CodeGenResult<()>;
}

impl<'ctx> CodeGenStatements<'ctx> for CodeGenContext<'ctx> {
    fn visit_statement(
        &mut self,
        stmt: &Statement,
        symbol_table: &mut SymbolTable<'ctx>,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        match stmt {
            Statement::VariableDecl {
                name,
                type_annotation,
                value,
                mutable,
                ..
            } => {
                // Implement the variable declaration
                self.complete_variable_decl_stmt(
                    name,
                    type_annotation,
                    value,
                    *mutable,
                    symbol_table,
                )?;
                Ok(CodeGenValue::Void)
            }
            Statement::Assignment { target, value, .. } => {
                // Evaluate the target
                match target {
                    Expression::Variable { name, .. } => {
                        // Look up the variable in the symbol table
                        let entry = symbol_table.lookup(&name.name).ok_or_else(|| {
                            CodeGenError::code_gen_error(
                                format!("Undefined variable: {}", name.name),
                                Some(name.source_info),
                            )
                        })?;

                        // Check if the variable is mutable
                        if !entry.mutable {
                            return Err(CodeGenError::code_gen_error(
                                format!("Cannot assign to immutable variable: {}", name.name),
                                Some(name.source_info),
                            ));
                        }

                        // Save the entry value
                        let entry_value = entry.value;

                        // Evaluate the value
                        let value_result = self.visit_expression(value, symbol_table)?;
                        let value_basic = value_result.as_basic_value()?;

                        // Store the value
                        build_store(self, entry_value, value_basic);

                        Ok(CodeGenValue::Void)
                    }
                    _ => Err(CodeGenError::unsupported_feature(
                        format!("Unsupported assignment target: {target:?}"),
                        None,
                    )),
                }
            }
            Statement::Expression(expr) => {
                // Evaluate the expression and discard the result
                self.visit_expression(expr, symbol_table)?;
                Ok(CodeGenValue::Void)
            }
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported statement type: {stmt:?}"),
                None,
            )),
        }
    }

    fn complete_variable_decl_stmt(
        &mut self,
        name: &Identifier,
        type_annotation: &Option<TypeExpression>,
        value: &Option<Box<Expression>>,
        mutable: bool,
        symbol_table: &mut SymbolTable<'ctx>,
    ) -> CodeGenResult<()> {
        // Get the LLVM type based on the type annotation or infer from the value
        let llvm_type;

        {
            let ctx = self.llvm_context.context();
            llvm_type = match type_annotation {
                Some(_ty) => {
                    // Convert the Typhon type to an LLVM type
                    ctx.i64_type().into()
                }
                None => {
                    if let Some(_val) = value {
                        // Infer the type from the value
                        ctx.i64_type().into()
                    } else {
                        // Default to i64 if no type or value is provided
                        ctx.i64_type().into()
                    }
                }
            };
        }

        // Create an alloca for the variable
        let alloca = create_entry_block_alloca(self, &name.name, llvm_type);

        // If there's an initial value, generate code for it and store it
        if let Some(val) = value {
            let init_value = self.visit_expression(val, symbol_table)?.as_basic_value()?;
            build_store(self, alloca, init_value);
        }

        // Add the variable to the symbol table
        let entry = SymbolEntry {
            value: alloca,
            ty: Rc::new(Type::Any), // Simplified type handling
            mutable,
        };

        symbol_table.add_symbol(name.name.clone(), entry);

        Ok(())
    }
}
