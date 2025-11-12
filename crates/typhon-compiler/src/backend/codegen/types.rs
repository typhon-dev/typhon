// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/types.rs
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

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue};

use crate::backend::error::{CodeGenError, CodeGenResult};
use crate::backend::llvm::LLVMContext;
use crate::typesystem::types::Type;

/// Represents the result of code generation for an AST node.
#[derive(Debug)]
pub enum CodeGenValue<'ctx> {
    /// A basic LLVM value (integer, float, pointer, etc.).
    Basic(BasicValueEnum<'ctx>),
    /// A function value.
    Function(FunctionValue<'ctx>),
    /// No value (void).
    Void,
}

/// Convert a Typhon type to an LLVM type.
pub fn get_llvm_type<'ctx>(
    llvm_context: &'ctx LLVMContext<'ctx>,
    ty: &Type,
) -> BasicTypeEnum<'ctx> {
    let ctx = llvm_context.context();

    // Create the appropriate LLVM type based on the Typhon type
    match ty {
        Type::Primitive(_) => {
            // In a full implementation, we'd have different handling for various primitive types
            ctx.i64_type().into()
        }
        Type::None => {
            // Void type if used as return, or pointer in other contexts
            ctx.ptr_type(AddressSpace::default()).into()
        }
        Type::Any => {
            // Generic type, use a pointer
            ctx.ptr_type(AddressSpace::default()).into()
        }
        Type::Function(_) => {
            // Function pointer type
            ctx.ptr_type(AddressSpace::default()).into()
        }
        _ => {
            // Default to a void pointer for complex types
            ctx.ptr_type(AddressSpace::default()).into()
        }
    }
}

impl<'ctx> CodeGenValue<'ctx> {
    /// Convert to a basic value, returns an error if this is not a basic value.
    pub fn as_basic_value(&self) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match self {
            CodeGenValue::Basic(value) => Ok(*value),
            _ => Err(CodeGenError::code_gen_error(
                "Expected a basic value".to_string(),
                None,
            )),
        }
    }

    /// Convert to a function value, returns an error if this is not a function value.
    pub fn as_function_value(&self) -> CodeGenResult<FunctionValue<'ctx>> {
        match self {
            CodeGenValue::Function(value) => Ok(*value),
            _ => Err(CodeGenError::code_gen_error(
                "Expected a function value".to_string(),
                None,
            )),
        }
    }

    /// Create a new basic value from the given value.
    /// This ensures we're not holding references to temporary values.
    pub fn new_basic<'a>(value: BasicValueEnum<'a>) -> CodeGenValue<'a> {
        CodeGenValue::Basic(value)
    }

    /// Create a new function value from the given value.
    /// This ensures we're not holding references to temporary values.
    pub fn new_function<'a>(value: FunctionValue<'a>) -> CodeGenValue<'a> {
        CodeGenValue::Function(value)
    }
}
