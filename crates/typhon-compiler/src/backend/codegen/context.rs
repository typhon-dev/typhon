// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/context.rs
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

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue};

use super::types::CodeGenValue;
use crate::backend::error::CodeGenResult;
use crate::backend::llvm::LLVMContext;
use crate::common::SourceInfo;
use crate::frontend::ast::{BinaryOperator, Literal, UnaryOperator};
use crate::typesystem::types::Type;

#[derive(Clone)]
/// Separate immutable context for code generation
pub struct CodeGenContext<'ctx> {
    /// The LLVM context.
    pub llvm_context: &'ctx LLVMContext<'ctx>,
    /// Set of imported modules to avoid duplicate imports
    pub imported_modules: HashSet<PathBuf>,
    /// Map of function declarations
    pub declared_functions: HashMap<String, Type>,
}

impl<'ctx> CodeGenContext<'ctx> {
    /// Create a new code generation context.
    pub fn new(llvm_context: &'ctx LLVMContext<'ctx>) -> Self {
        Self { llvm_context, imported_modules: HashSet::new(), declared_functions: HashMap::new() }
    }

    /// Build a binary operation.
    pub fn build_binary_op(
        &mut self,
        op: BinaryOperator,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        // Ensure both operands are of the same type
        if left.get_type() != right.get_type() {
            return Err(crate::backend::error::CodeGenError::code_gen_error(
                format!(
                    "Mismatched operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ));
        }

        // Get builder directly from our context
        let builder = self.llvm_context.builder();

        // Handle integer operations
        if left.is_int_value() && right.is_int_value() {
            let (left_int, right_int) = (left.into_int_value(), right.into_int_value());

            match op {
                BinaryOperator::Add => Ok(builder
                    .build_int_add(left_int, right_int, name)
                    .expect("Failed to build integer addition")
                    .as_basic_value_enum()),
                BinaryOperator::Sub => Ok(builder
                    .build_int_sub(left_int, right_int, name)
                    .expect("Failed to build integer subtraction")
                    .as_basic_value_enum()),
                BinaryOperator::Mul => Ok(builder
                    .build_int_mul(left_int, right_int, name)
                    .expect("Failed to build integer multiplication")
                    .as_basic_value_enum()),
                BinaryOperator::Div => Ok(builder
                    .build_int_signed_div(left_int, right_int, name)
                    .expect("Failed to build integer division")
                    .as_basic_value_enum()),
                BinaryOperator::Mod => Ok(builder
                    .build_int_signed_rem(left_int, right_int, name)
                    .expect("Failed to build integer remainder")
                    .as_basic_value_enum()),
                BinaryOperator::Eq => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::EQ, left_int, right_int, name)
                    .expect("Failed to build integer equality comparison")
                    .as_basic_value_enum()),
                BinaryOperator::NotEq => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::NE, left_int, right_int, name)
                    .expect("Failed to build integer inequality comparison")
                    .as_basic_value_enum()),
                BinaryOperator::Lt => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SLT, left_int, right_int, name)
                    .expect("Failed to build integer less-than comparison")
                    .as_basic_value_enum()),
                BinaryOperator::LtE => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SLE, left_int, right_int, name)
                    .expect("Failed to build integer less-than-or-equal comparison")
                    .as_basic_value_enum()),
                BinaryOperator::Gt => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SGT, left_int, right_int, name)
                    .expect("Failed to build integer greater-than comparison")
                    .as_basic_value_enum()),
                BinaryOperator::GtE => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SGE, left_int, right_int, name)
                    .expect("Failed to build integer greater-than-or-equal comparison")
                    .as_basic_value_enum()),
                BinaryOperator::BitAnd => Ok(builder
                    .build_and(left_int, right_int, name)
                    .expect("Failed to build bitwise AND operation")
                    .as_basic_value_enum()),
                BinaryOperator::BitOr => Ok(builder
                    .build_or(left_int, right_int, name)
                    .expect("Failed to build bitwise OR operation")
                    .as_basic_value_enum()),
                BinaryOperator::BitXor => Ok(builder
                    .build_xor(left_int, right_int, name)
                    .expect("Failed to build bitwise XOR operation")
                    .as_basic_value_enum()),
                BinaryOperator::LShift => Ok(builder
                    .build_left_shift(left_int, right_int, name)
                    .expect("Failed to build left shift operation")
                    .as_basic_value_enum()),
                BinaryOperator::RShift => Ok(builder
                    .build_right_shift(left_int, right_int, true, name)
                    .expect("Failed to build right shift operation")
                    .as_basic_value_enum()),

                _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation: {op:?}"),
                    None,
                )),
            }
        }
        // Handle float operations
        else if left.is_float_value() && right.is_float_value() {
            let (left_float, right_float) = (left.into_float_value(), right.into_float_value());

            match op {
                BinaryOperator::Add => Ok(builder
                    .build_float_add(left_float, right_float, name)
                    .expect("Failed to build float addition")
                    .as_basic_value_enum()),
                BinaryOperator::Sub => Ok(builder
                    .build_float_sub(left_float, right_float, name)
                    .expect("Failed to build float subtraction")
                    .as_basic_value_enum()),
                BinaryOperator::Mul => Ok(builder
                    .build_float_mul(left_float, right_float, name)
                    .expect("Failed to build float multiplication")
                    .as_basic_value_enum()),
                BinaryOperator::Div => Ok(builder
                    .build_float_div(left_float, right_float, name)
                    .expect("Failed to build float division")
                    .as_basic_value_enum()),
                BinaryOperator::Eq => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        left_float,
                        right_float,
                        name,
                    )
                    .expect("Failed to build float equality comparison")
                    .as_basic_value_enum()),
                BinaryOperator::NotEq => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        left_float,
                        right_float,
                        name,
                    )
                    .expect("Failed to build float inequality comparison")
                    .as_basic_value_enum()),
                BinaryOperator::Lt => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OLT,
                        left_float,
                        right_float,
                        name,
                    )
                    .expect("Failed to build float less-than comparison")
                    .as_basic_value_enum()),
                BinaryOperator::LtE => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OLE,
                        left_float,
                        right_float,
                        name,
                    )
                    .expect("Failed to build float less-than-or-equal comparison")
                    .as_basic_value_enum()),
                BinaryOperator::Gt => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OGT,
                        left_float,
                        right_float,
                        name,
                    )
                    .expect("Failed to build float greater-than comparison")
                    .as_basic_value_enum()),
                BinaryOperator::GtE => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OGE,
                        left_float,
                        right_float,
                        name,
                    )
                    .expect("Failed to build float greater-than-or-equal comparison")
                    .as_basic_value_enum()),

                _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation for float: {op:?}"),
                    None,
                )),
            }
        } else {
            Err(crate::backend::error::CodeGenError::unsupported_feature(
                format!(
                    "Unsupported operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ))
        }
    }

    /// Build a unary operation.
    pub fn build_unary_op(
        &mut self,
        op: UnaryOperator,
        operand: BasicValueEnum<'ctx>,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let builder = self.llvm_context.builder();

        // Handle integer operations
        if operand.is_int_value() {
            let int_operand = operand.into_int_value();

            match op {
                UnaryOperator::Pos => {
                    // For integers, positive is the value itself
                    Ok(int_operand.into())
                }
                UnaryOperator::Neg => {
                    // For integers, negation is 0 - operand
                    let zero = int_operand.get_type().const_int(0, false);
                    Ok(builder
                        .build_int_sub(zero, int_operand, name)
                        .expect("Failed to build integer negation")
                        .as_basic_value_enum())
                }
                UnaryOperator::Not => {
                    // For integers, logical not is comparison with zero
                    let zero = int_operand.get_type().const_int(0, false);
                    Ok(builder
                        .build_int_compare(inkwell::IntPredicate::EQ, int_operand, zero, name)
                        .expect("Failed to build integer comparison for logical not")
                        .as_basic_value_enum())
                }
                UnaryOperator::Invert => {
                    // Bitwise not is XOR with all ones
                    let all_ones = int_operand.get_type().const_all_ones();
                    Ok(builder
                        .build_xor(int_operand, all_ones, name)
                        .expect("Failed to build bitwise NOT operation")
                        .as_basic_value_enum())
                }
            }
        }
        // Handle float operations
        else if operand.is_float_value() {
            let float_operand = operand.into_float_value();
            match op {
                UnaryOperator::Pos => Ok(float_operand.into()),
                UnaryOperator::Neg => Ok(builder
                    .build_float_neg(float_operand, name)
                    .expect("Failed to build float negation")
                    .as_basic_value_enum()),
                UnaryOperator::Not => {
                    // For floats, logical not is comparison with zero
                    let zero = float_operand.get_type().const_float(0.0);
                    Ok(builder
                        .build_float_compare(
                            inkwell::FloatPredicate::OEQ,
                            float_operand,
                            zero,
                            name,
                        )
                        .expect("Failed to build float comparison for logical not")
                        .into())
                }
                _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                    format!("Unsupported unary operation for float: {op:?}"),
                    None,
                )),
            }
        } else {
            Err(crate::backend::error::CodeGenError::unsupported_feature(
                format!("Unsupported operand type for unary operation: {:?}", operand.get_type()),
                None,
            ))
        }
    }

    /// Get the current function being built
    pub fn current_function(&self) -> Option<FunctionValue<'ctx>> {
        // Get the current basic block
        let builder = self.llvm_context.builder();
        let current_block = builder.get_insert_block();

        // If we have a current block, get its parent function
        current_block.and_then(|block| block.get_parent())
    }

    /// Visit a literal and generate LLVM IR for it.
    pub fn visit_literal(
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
            _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                format!("Unsupported literal type: {literal:?}"),
                Some(*source_info),
            )),
        }
    }
}
