// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/expressions.rs
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
//! This module handles expression code generation.

use inkwell::values::BasicValue;

use super::context::CodeGenContext;
use super::functions::build_load;
use super::symbol_table::SymbolTable;
use super::types::CodeGenValue;
use crate::backend::error::{CodeGenError, CodeGenResult};
use crate::common::SourceInfo;
use crate::frontend::ast::{Expression, Literal};

/// Extension trait for expression operations on CodeGenContext
pub trait CodeGenExpressions<'ctx> {
    /// Visit an expression and generate LLVM IR for it.
    fn visit_expression(
        &mut self,
        expr: &Expression,
        symbol_table: &SymbolTable<'ctx>,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;

    /// Visit a literal and generate LLVM IR for it.
    fn visit_literal(
        &mut self,
        literal: &Literal,
        source_info: &SourceInfo,
    ) -> CodeGenResult<CodeGenValue<'ctx>>;
}

impl<'ctx> CodeGenExpressions<'ctx> for CodeGenContext<'ctx> {
    fn visit_expression(
        &mut self,
        expr: &Expression,
        symbol_table: &SymbolTable<'ctx>,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        match expr {
            Expression::Literal { value, source_info } => self.visit_literal(value, source_info),
            Expression::BinaryOp { left, op, right, .. } => {
                let left_value = self.visit_expression(left, symbol_table)?.as_basic_value()?;
                let right_value = self.visit_expression(right, symbol_table)?.as_basic_value()?;

                // Binary operation implementation
                let result = self.build_binary_op(*op, left_value, right_value, "binop")?;
                Ok(CodeGenValue::new_basic(result))
            }
            Expression::UnaryOp { op, operand, .. } => {
                let operand_value =
                    self.visit_expression(operand, symbol_table)?.as_basic_value()?;

                // Unary operation implementation
                let result = self.build_unary_op(*op, operand_value, "unop")?;
                Ok(CodeGenValue::new_basic(result))
            }
            Expression::Variable { name, source_info } => {
                // Variable lookup implementation
                let entry = symbol_table.lookup(&name.name).ok_or_else(|| {
                    CodeGenError::code_gen_error(
                        format!("Undefined variable: {}", name.name),
                        Some(*source_info),
                    )
                })?;

                // Load the variable value
                let value = build_load(self, entry.value, &name.name);
                Ok(CodeGenValue::new_basic(value))
            }
            // Placeholder for other expression types
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported expression type: {expr:?}"),
                None,
            )),
        }
    }

    fn visit_literal(
        &mut self,
        literal: &Literal,
        source_info: &SourceInfo,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Create the values outside the context borrow to avoid lifetime issues
        match literal {
            Literal::Int(i) => {
                let value;
                {
                    let int_type = self.llvm_context.context().i64_type();
                    value = int_type.const_int(*i as u64, true);
                }
                Ok(CodeGenValue::new_basic(value.into()))
            }
            Literal::Float(f) => {
                let value;
                {
                    let float_type = self.llvm_context.context().f64_type();
                    value = float_type.const_float(*f);
                }
                Ok(CodeGenValue::new_basic(value.into()))
            }
            Literal::String(s) => {
                // Create global string
                let builder = self.llvm_context.builder();
                let value = builder
                    .build_global_string_ptr(s, "str")
                    .expect("Failed to build global string pointer")
                    .as_basic_value_enum();

                Ok(CodeGenValue::new_basic(value))
            }
            Literal::Bool(b) => {
                let value;
                {
                    let bool_type = self.llvm_context.context().bool_type();
                    value = bool_type.const_int(*b as u64, false);
                }
                Ok(CodeGenValue::new_basic(value.into()))
            }
            // Placeholder for other literal types
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported literal type: {literal:?}"),
                Some(*source_info),
            )),
        }
    }
}
