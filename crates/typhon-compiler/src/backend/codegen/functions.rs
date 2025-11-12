// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/functions.rs
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
//! This module handles function code generation.

use std::rc::Rc;

use inkwell::types::{AnyType, AnyTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue};

use super::context::CodeGenContext;
use super::symbol_table::{SymbolEntry, SymbolTable};
use crate::backend::error::CodeGenResult;
use crate::frontend::ast::Statement;
use crate::typesystem::types::Type;

/// Compile an AST to LLVM IR.
/// This is the main entry point for the compiler.
pub fn compile<'ctx>(
    context: &mut CodeGenContext<'ctx>,
    symbol_table: &mut SymbolTable<'ctx>,
    statements: &[Statement],
) -> CodeGenResult<()> {
    // Process each top-level statement
    for stmt in statements {
        // In the refactored version, we would use the visitor pattern
        // For example: visitor.visit_statement(stmt, context, symbol_table)?
        // This is a simplified version
        match stmt {
            Statement::FunctionDef { .. } => {
                gen_function(context, symbol_table, stmt)?;
            }
            // For other statement types, we'd call the appropriate handler
            _ => {
                // In a full implementation, we would handle all statement types
                // but for now, we just skip unsupported statements
            }
        }
    }

    // Verify the module
    if context.llvm_context.module().verify().is_err() {
        return Err(crate::backend::error::CodeGenError::code_gen_error(
            "Module verification failed".to_string(),
            None,
        ));
    }

    Ok(())
}

/// Generates LLVM IR for a function definition
pub fn gen_function<'ctx>(
    context: &mut CodeGenContext<'ctx>,
    symbol_table: &mut SymbolTable<'ctx>,
    function_def: &Statement,
) -> CodeGenResult<FunctionValue<'ctx>> {
    if let Statement::FunctionDef { name, parameters, .. } = function_def {
        // Extract all values from context at once to avoid multiple borrows
        let mut param_types = Vec::new();
        let fn_return_type;
        let fn_type;
        let function;
        let function_name = name.name.clone();

        // First get all values from the context
        {
            // Create a single borrow of the context that will be dropped when this block ends
            let ctx = context.llvm_context.context();

            // Create parameter types
            for _ in parameters.iter() {
                // Simplify by using i64 for all parameters for now
                param_types.push(ctx.i64_type().into());
            }

            // Create the function type
            // Simplify by using i64 as the return type for now
            fn_return_type = ctx.i64_type();
            fn_type = fn_return_type.fn_type(&param_types, false);

            // Create the function in this scope to avoid keeping the borrow too long
            function = context.llvm_context.module().add_function(&function_name, fn_type, None);
        }

        // Create a basic block
        let ctx = context.llvm_context.context();
        ctx.append_basic_block(function, "entry");

        // Position the builder at the start of the entry block
        let builder = context.llvm_context.builder();
        let entry_block = function.get_first_basic_block().unwrap();
        builder.position_at_end(entry_block);

        // Push a new scope for the function parameters
        symbol_table.push_scope();

        // Add parameters to the symbol table
        for (i, param) in parameters.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let param_name = &param.name.name;

            // Create an alloca for the parameter
            let param_alloca =
                create_entry_block_alloca(context, param_name, param_value.get_type());

            // Store the parameter value
            build_store(context, param_alloca, param_value);

            // Add to the symbol table
            let param_entry = SymbolEntry {
                value: param_alloca,
                ty: Rc::new(Type::Any), // Simplified type handling
                mutable: true,
            };

            symbol_table.add_symbol(param_name.clone(), param_entry);
        }

        // Note: The code for generating function body would go here in a full implementation.
        // In the original code, this is done by calling:
        // for stmt in body {
        //     self.visit_statement(stmt)?;
        // }
        // However, this requires the Visitor trait implementation which is outside the
        // scope of this specific module refactoring.

        // If there's no return statement, add a default return value
        {
            let builder = context.llvm_context.builder();
            let ctx = context.llvm_context.context();

            // If there's no terminator, we need to add a return
            if builder.get_insert_block().unwrap().get_terminator().is_none() {
                // For now, just return 0 for any function
                let return_type = ctx.i64_type();
                let return_value = return_type.const_int(0, false);
                builder
                    .build_return(Some(&return_value))
                    .expect("Failed to build return instruction");
            }
        }

        // Verify the function - store result first to avoid borrow issues
        let is_valid = function.verify(true);

        if !is_valid {
            // Pop the scope before returning
            symbol_table.pop_scope();

            return Err(crate::backend::error::CodeGenError::code_gen_error(
                format!("Invalid function: {function_name}"),
                Some(name.source_info),
            ));
        }

        Ok(function)
    } else {
        Err(crate::backend::error::CodeGenError::code_gen_error(
            "Expected a function definition statement".to_string(),
            None,
        ))
    }
}

/// Creates an alloca instruction in the entry block of the function.
/// This is used for allocating space for function parameters and local variables.
pub fn create_entry_block_alloca<'ctx>(
    context: &CodeGenContext<'ctx>,
    name: &str,
    ty: BasicTypeEnum<'ctx>,
) -> BasicValueEnum<'ctx> {
    // Get the current function
    if let Some(function) = context.current_function() {
        // Get builder from context
        let builder = context.llvm_context.builder();
        let entry_block = function.get_first_basic_block().unwrap();
        let current_block = builder.get_insert_block().unwrap();

        // Move to the first instruction or the start of the block
        builder.position_at(
            entry_block,
            &entry_block.get_first_instruction().unwrap_or_else(|| {
                // If there are no instructions yet, add a temporary one to position at
                let temp = builder
                    .build_alloca(ty, "temp_positioning")
                    .expect("Failed to build temporary alloca for positioning");
                builder.build_free(temp).expect("Failed to build free instruction");
                entry_block.get_last_instruction().unwrap()
            }),
        );

        // Create the alloca
        let alloca = builder.build_alloca(ty, name);

        // Move back to where we were
        builder.position_at_end(current_block);

        return alloca.unwrap().into();
    }

    panic!("Cannot create alloca outside of a function");
}

/// Build a store instruction.
pub fn build_store<'ctx>(
    context: &CodeGenContext<'ctx>,
    ptr: BasicValueEnum<'ctx>,
    value: BasicValueEnum<'ctx>,
) {
    // Check if the pointer is actually a pointer type
    let ptr_value = ptr.into_pointer_value();
    let builder = context.llvm_context.builder();
    builder.build_store(ptr_value, value).expect("Failed to build store instruction");
}

/// Build a load instruction.
pub fn build_load<'ctx>(
    context: &CodeGenContext<'ctx>,
    ptr: BasicValueEnum<'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    // Check if the pointer is actually a pointer type
    let ptr_value = ptr.into_pointer_value();
    // Get the type of the pointed-to value
    let pointee_type = ptr_value.get_type().as_any_type_enum();

    // Build the load based on the pointee type
    let builder = context.llvm_context.builder();

    match pointee_type {
        AnyTypeEnum::IntType(_) => builder
            .build_load(pointee_type.into_int_type(), ptr_value, name)
            .expect("Failed to build int load instruction"),
        AnyTypeEnum::FloatType(_) => builder
            .build_load(pointee_type.into_float_type(), ptr_value, name)
            .expect("Failed to build float load instruction"),
        AnyTypeEnum::PointerType(_) => builder
            .build_load(pointee_type.into_pointer_type(), ptr_value, name)
            .expect("Failed to build pointer load instruction"),
        AnyTypeEnum::StructType(_) => builder
            .build_load(pointee_type.into_struct_type(), ptr_value, name)
            .expect("Failed to build struct load instruction"),
        AnyTypeEnum::ArrayType(_) => builder
            .build_load(pointee_type.into_array_type(), ptr_value, name)
            .expect("Failed to build array load instruction"),
        _ => panic!("Unsupported pointee type for load: {pointee_type:?}"),
    }
}
