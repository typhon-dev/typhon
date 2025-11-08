// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen.rs
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
//! Code generation module for the Typhon compiler.
//!
//! This module provides code generation from Typhon AST to LLVM IR.
//! The architecture is designed to handle LLVM's complex lifetime requirements by:
//!
//! 1. Separating immutable context from mutable state
//! 2. Using `Arc<Mutex<>>` for thread-safe shared mutable access
//! 3. Carefully managing borrowing patterns to avoid conflicts
//! 4. Ensuring proper lifetimes for LLVM objects
//!
//! The main components are:
//! - `CodeGenContext`: Immutable context for code generation
//! - `CodeGenState`: Mutable state for code generation
//! - `CodeGenerator`: Main code generator that combines context and state
//! - `SymbolTable`: Tracks variables in scope during code generation

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{
    Arc,
    Mutex,
};

use inkwell::AddressSpace;
use inkwell::types::{
    AnyType,
    AnyTypeEnum,
    BasicTypeEnum,
};
use inkwell::values::{
    BasicValue,
    BasicValueEnum,
    FunctionValue,
};

use crate::backend::error::{
    CodeGenError,
    CodeGenResult,
};
use crate::backend::llvm::LLVMContext;
use crate::common::{
    SourceInfo,
    Span,
};
use crate::frontend::ast::visitor::Visitor;
use crate::frontend::ast::{
    BinaryOperator,
    Expression,
    Identifier,
    Literal,
    Statement,
    TypeExpression,
    UnaryOperator,
};
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

/// A symbol entry in the symbol table.
#[derive(Debug)]
pub struct SymbolEntry<'ctx> {
    /// The LLVM value representing the variable.
    pub value: BasicValueEnum<'ctx>,
    /// The type of the variable.
    pub ty: Rc<Type>,
    /// Whether the variable is mutable.
    pub mutable: bool,
}

/// A symbol table for tracking variables in scope.
#[derive(Debug)]
pub struct SymbolTable<'ctx> {
    /// Nested scopes, with the last one being the current scope.
    scopes: Vec<HashMap<String, SymbolEntry<'ctx>>>,
}

impl<'ctx> Default for SymbolTable<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ctx> SymbolTable<'ctx> {
    /// Create a new symbol table.
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());
        SymbolTable { scopes }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add a symbol to the current scope.
    pub fn add_symbol(&mut self, name: String, entry: SymbolEntry<'ctx>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, entry);
        }
    }

    /// Look up a symbol in the current scope chain.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry<'ctx>> {
        // Look in scopes from inner to outer
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }
}

/// Separate immutable context for code generation
pub struct CodeGenContext<'ctx> {
    /// The LLVM context.
    llvm_context: &'ctx LLVMContext<'ctx>,
}

/// Mutable state for code generation
pub struct CodeGenState<'ctx> {
    /// The symbol table.
    symbol_table: SymbolTable<'ctx>,
    /// The current function being generated.
    current_function: Option<FunctionValue<'ctx>>,
}

/// Code generator for Typhon AST.
pub struct CodeGenerator<'ctx> {
    /// The immutable context.
    context: CodeGenContext<'ctx>,
    /// The mutable state.
    state: Arc<Mutex<CodeGenState<'ctx>>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// Create a new code generator.
    pub fn new(llvm_context: &'ctx LLVMContext<'ctx>) -> Self {
        let context = CodeGenContext { llvm_context };
        let state = CodeGenState {
            symbol_table: SymbolTable::new(),
            current_function: None,
        };

        CodeGenerator {
            context,
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Build an alloca instruction.
    fn create_alloca(&self, name: &str, ty: BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        // Get the state with a scoped lock
        let state = self.state.lock().unwrap();

        // Create alloca at the entry of the function
        if let Some(function) = state.current_function {
            // Get builder from context
            let builder = self.context.llvm_context.builder();
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
                    builder
                        .build_free(temp)
                        .expect("Failed to build free instruction");
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

    /// Build a load instruction.
    fn build_load(&self, ptr: BasicValueEnum<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
        // Check if the pointer is actually a pointer type
        let ptr_value = ptr.into_pointer_value();
        // Get the type of the pointed-to value
        let pointee_type = ptr_value.get_type().as_any_type_enum();

        // Build the load based on the pointee type
        let builder = self.context.llvm_context.builder();

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

    /// Build a store instruction.
    fn build_store(&self, ptr: BasicValueEnum<'ctx>, value: BasicValueEnum<'ctx>) {
        // Check if the pointer is actually a pointer type
        let ptr_value = ptr.into_pointer_value();
        let builder = self.context.llvm_context.builder();
        builder
            .build_store(ptr_value, value)
            .expect("Failed to build store instruction");
    }

    /// Create a global string constant.
    fn create_global_string(&self, string: &str, name: &str) -> BasicValueEnum<'ctx> {
        // Get builder directly from our context
        let builder = self.context.llvm_context.builder();

        // Create the global string
        

        builder
            .build_global_string_ptr(string, name)
            .expect("Failed to build global string pointer")
            .as_basic_value_enum()
    }

    /// Build a binary operation.
    fn build_binary_op(
        &self,
        op: BinaryOperator,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        // Ensure both operands are of the same type
        if left.get_type() != right.get_type() {
            return Err(CodeGenError::code_gen_error(
                format!(
                    "Mismatched operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ));
        }

        // Get builder directly from our context
        let builder = self.context.llvm_context.builder();

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

                _ => Err(CodeGenError::unsupported_feature(
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

                _ => Err(CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation for float: {op:?}"),
                    None,
                )),
            }
        } else {
            Err(CodeGenError::unsupported_feature(
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
    fn build_unary_op(
        &self,
        op: UnaryOperator,
        operand: BasicValueEnum<'ctx>,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let builder = self.context.llvm_context.builder();

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
                        .as_basic_value_enum())
                }
                _ => Err(CodeGenError::unsupported_feature(
                    format!("Unsupported unary operation for float: {op:?}"),
                    None,
                )),
            }
        } else {
            Err(CodeGenError::unsupported_feature(
                format!(
                    "Unsupported operand type for unary operation: {:?}",
                    operand.get_type()
                ),
                None,
            ))
        }
    }

    // Code generation for a function declaration
    fn gen_function(&mut self, func: &Statement) -> CodeGenResult<FunctionValue<'ctx>> {
        if let Statement::FunctionDef {
            name,
            parameters,
            return_type: _,
            body,
            ..
        } = func
        {
            // Extract all values from context at once to avoid multiple borrows
            let mut param_types = Vec::new();
            let fn_return_type;
            let fn_type;
            let function;
            let function_name = name.name.clone();

            // First get all values from the context
            {
                // Create a single borrow of the context that will be dropped when this block ends
                let ctx = self.context.llvm_context.context();

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
                function =
                    self.context
                        .llvm_context
                        .module()
                        .add_function(&function_name, fn_type, None);
            }

            // Create a basic block
            let ctx = self.context.llvm_context.context();
            ctx.append_basic_block(function, "entry");

            // Set the current function
            self.state.lock().unwrap().current_function = Some(function);

            // Position the builder has already been set in the context block above

            // Push a new scope for the function parameters
            self.state.lock().unwrap().symbol_table.push_scope();

            // Add parameters to the symbol table
            for (i, param) in parameters.iter().enumerate() {
                let param_value = function.get_nth_param(i as u32).unwrap();
                let param_name = &param.name.name;

                // Create an alloca for the parameter
                let param_alloca = self.create_alloca(param_name, param_value.get_type());

                // Store the parameter value
                self.build_store(param_alloca, param_value);

                // Add to the symbol table
                let param_entry = SymbolEntry {
                    value: param_alloca,
                    ty: Rc::new(Type::Any), // Simplified type handling
                    mutable: true,
                };

                self.state
                    .lock()
                    .unwrap()
                    .symbol_table
                    .add_symbol(param_name.clone(), param_entry);
            }

            // Generate code for the function body
            for stmt in body {
                self.visit_statement(stmt)?;
            }

            // If there's no return statement, add a default return value
            {
                let builder = self.context.llvm_context.builder();
                let ctx = self.context.llvm_context.context();

                // If there's no terminator, we need to add a return
                if builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator().is_none()
                {
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

            if is_valid {
                Ok(function)
            } else {
                // Pop the scope before returning
                self.state.lock().unwrap().symbol_table.pop_scope();

                Err(CodeGenError::code_gen_error(
                    format!("Invalid function: {function_name}"),
                    Some(name.source_info),
                ))
            }
        } else {
            Err(CodeGenError::code_gen_error(
                "Expected a function definition statement".to_string(),
                None,
            ))
        }
    }

    /// Convert a Typhon type to an LLVM type.
    pub fn get_llvm_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        let result;

        {
            let llvm_context = self.context.llvm_context.context();

            // Create the appropriate LLVM type based on the Typhon type
            result = match ty {
                Type::Primitive(_) => llvm_context.ptr_type(AddressSpace::default()).into(),
                Type::None => llvm_context.ptr_type(AddressSpace::default()).into(),
                Type::Any => llvm_context.ptr_type(AddressSpace::default()).into(),
                _ => {
                    // Default to a void pointer for complex types
                    llvm_context.ptr_type(AddressSpace::default()).into()
                }
            };
        }

        result
    }

    /// Compile an AST to LLVM IR.
    pub fn compile(&mut self, statements: &[Statement]) -> CodeGenResult<()> {
        // Process each top-level statement
        for stmt in statements {
            self.visit_statement(stmt)?;
        }

        // Verify the module
        if self.context.llvm_context.module().verify().is_err() {
            return Err(CodeGenError::code_gen_error(
                "Module verification failed".to_string(),
                None,
            ));
        }

        Ok(())
    }

    /// Complete variable declaration statement.
    pub fn complete_variable_decl_stmt(
        &mut self,
        name: &Identifier,
        type_annotation: &Option<TypeExpression>,
        value: &Option<Box<Expression>>,
        mutable: bool,
    ) -> CodeGenResult<()> {
        // Get the LLVM type based on the type annotation or infer from the value
        let llvm_type;

        {
            let ctx = self.context.llvm_context.context();
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
        let alloca = self.create_alloca(&name.name, llvm_type);

        // If there's an initial value, generate code for it and store it
        if let Some(val) = value {
            let init_value = self.visit_expression(val)?.as_basic_value()?;
            self.build_store(alloca, init_value);
        }

        // Add the variable to the symbol table
        let entry = SymbolEntry {
            value: alloca,
            ty: Rc::new(Type::Any), // Simplified type handling
            mutable,
        };

        self.state
            .lock()
            .unwrap()
            .symbol_table
            .add_symbol(name.name.clone(), entry);

        Ok(())
    }

    /// Visit an expression.
    fn visit_expression(&mut self, expr: &Expression) -> CodeGenResult<CodeGenValue<'ctx>> {
        match expr {
            Expression::Literal { value, source_info } => self.visit_literal(value, source_info),
            Expression::BinaryOp {
                left,
                op,
                right,
                source_info: _,
            } => {
                let left_value = self.visit_expression(left)?.as_basic_value()?;
                let right_value = self.visit_expression(right)?.as_basic_value()?;

                // Binary operation implementation
                let result = self.build_binary_op(*op, left_value, right_value, "binop")?;
                Ok(CodeGenValue::new_basic(result))
            }
            Expression::UnaryOp {
                op,
                operand,
                source_info: _,
            } => {
                let operand_value = self.visit_expression(operand)?.as_basic_value()?;

                // Unary operation implementation
                let result = self.build_unary_op(*op, operand_value, "unop")?;
                Ok(CodeGenValue::new_basic(result))
            }
            Expression::Variable { name, source_info } => {
                let state = self.state.lock().unwrap();

                // Variable lookup implementation
                let entry = state.symbol_table.lookup(&name.name).ok_or_else(|| {
                    CodeGenError::code_gen_error(
                        format!("Undefined variable: {}", name.name),
                        Some(*source_info),
                    )
                })?;

                // Load the variable value
                let value = self.build_load(entry.value, &name.name);
                Ok(CodeGenValue::new_basic(value))
            }
            // Placeholder for other expression types
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported expression type: {expr:?}"),
                None,
            )),
        }
    }

    /// Visit a literal.
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
                    let int_type = self.context.llvm_context.context().i64_type();
                    value = int_type.const_int(*i as u64, true);
                }
                Ok(CodeGenValue::new_basic(value.into()))
            }
            Literal::Float(f) => {
                let value;
                {
                    let float_type = self.context.llvm_context.context().f64_type();
                    value = float_type.const_float(*f);
                }
                Ok(CodeGenValue::new_basic(value.into()))
            }
            Literal::String(s) => {
                // Create global string uses its own method that properly returns a value
                let value = self.create_global_string(s, "str");
                Ok(CodeGenValue::new_basic(value))
            }
            Literal::Bool(b) => {
                let value;
                {
                    let bool_type = self.context.llvm_context.context().bool_type();
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

// Visitor trait implementation for the code generator
impl<'ctx> Visitor<CodeGenResult<CodeGenValue<'ctx>>> for CodeGenerator<'ctx> {
    fn visit_module(
        &mut self,
        module: &crate::frontend::ast::Module,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Process each statement in the module
        for stmt in &module.statements {
            self.visit_statement(stmt)?;
        }
        Ok(CodeGenValue::Void)
    }

    fn visit_expression(&mut self, expr: &Expression) -> CodeGenResult<CodeGenValue<'ctx>> {
        // We already have an implementation of this method in the CodeGenerator class
        // Call that implementation here
        CodeGenerator::visit_expression(self, expr)
    }

    fn visit_type_expression(
        &mut self,
        _type_expr: &TypeExpression,
    ) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Type expressions don't generate code directly, just return Void
        Ok(CodeGenValue::Void)
    }

    fn visit_literal(&mut self, lit: &Literal) -> CodeGenResult<CodeGenValue<'ctx>> {
        // Simplified implementation just to make the Visitor trait happy
        // Visit_literal takes a source_info parameter as well, which we don't have here
        // Use a dummy SourceInfo
        let dummy_source = SourceInfo::new(Span::new(0, 0));
        CodeGenerator::visit_literal(self, lit, &dummy_source)
    }

    fn visit_statement(&mut self, stmt: &Statement) -> CodeGenResult<CodeGenValue<'ctx>> {
        match stmt {
            Statement::FunctionDef { .. } => {
                let function = self.gen_function(stmt)?;
                Ok(CodeGenValue::new_function(function))
            }
            Statement::Return { value, .. } => {
                let builder = self.context.llvm_context.builder();

                if let Some(expr) = value {
                    // Evaluate the expression
                    let value_result = self.visit_expression(expr)?;
                    let return_value = value_result.as_basic_value()?;

                    // Build the return instruction
                    let _ = builder.build_return(Some(&return_value));
                } else {
                    // Return void
                    builder
                        .build_return(None)
                        .expect("Failed to build void return instruction");
                }

                Ok(CodeGenValue::Void)
            }
            Statement::VariableDecl {
                name,
                type_annotation,
                value,
                mutable,
                ..
            } => {
                // Implement the variable declaration
                self.complete_variable_decl_stmt(name, type_annotation, value, *mutable)?;
                Ok(CodeGenValue::Void)
            }
            Statement::Assignment { target, value, .. } => {
                // Evaluate the target
                match target {
                    Expression::Variable { name, .. } => {
                        let state = self.state.lock().unwrap();

                        // Look up the variable in the symbol table
                        let entry = state.symbol_table.lookup(&name.name).ok_or_else(|| {
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
                        // Drop the state lock before calling visit_expression
                        drop(state);

                        // Evaluate the value
                        let value_result = self.visit_expression(value)?;
                        let value_basic = value_result.as_basic_value()?;

                        // Store the value
                        self.build_store(entry_value, value_basic);

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
                self.visit_expression(expr)?;
                Ok(CodeGenValue::Void)
            }
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported statement type: {stmt:?}"),
                None,
            )),
        }
    }
}
