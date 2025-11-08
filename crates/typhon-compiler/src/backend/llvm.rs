// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/llvm.rs
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
//! LLVM wrapper for the Typhon compiler.
//!
//! This module provides a safe wrapper around the LLVM C API for code generation.

use std::env;
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::{
    PassBuilderOptions,
    PassManager,
};
use inkwell::targets::{
    CodeModel,
    FileType,
    InitializationConfig,
    RelocMode,
    Target,
    TargetMachine,
};
use inkwell::types::{
    BasicMetadataTypeEnum,
    BasicTypeEnum,
    FunctionType,
    StructType,
};
use inkwell::{
    AddressSpace,
    OptimizationLevel,
};

use crate::backend::error::{
    CodeGenError,
    CodeGenResult,
};
use crate::typesystem::types::{
    FunctionType as TyphonFunctionType,
    PrimitiveTypeKind,
    Type as TyphonType,
};

/// LLVM context wrapper.
pub struct LLVMContext<'ctx> {
    /// The LLVM context.
    context: &'ctx Context,
    /// The LLVM module.
    module: Module<'ctx>,
    /// The LLVM builder.
    builder: Builder<'ctx>,
    /// The LLVM pass manager.
    pass_manager: PassManager<Module<'ctx>>,
}

impl<'ctx> LLVMContext<'ctx> {
    /// Creates a new LLVM context.
    ///
    /// Note: The context parameter must outlive the returned LLVMContext.
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let pass_manager = PassManager::create(());

        // Initialize target
        Self::initialize_target();

        Self {
            context,
            module,
            builder,
            pass_manager,
        }
    }

    /// Initializes the LLVM target.
    fn initialize_target() {
        // Print a helpful message about LLVM path configuration
        if env::var("LLVM_SYS_181_PREFIX").is_err() {
            eprintln!(
                "Note: You can set LLVM_SYS_181_PREFIX environment variable to help find LLVM"
            );
            eprintln!("Example: LLVM_SYS_181_PREFIX=/usr/local/opt/llvm@18 cargo build");
        }

        let config = InitializationConfig {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        };

        Target::initialize_all(&config);
    }

    /// Gets the LLVM context.
    pub fn context(&self) -> &Context {
        self.context
    }

    /// Gets the LLVM module.
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Gets the LLVM builder.
    pub fn builder(&self) -> &Builder<'ctx> {
        &self.builder
    }

    /// Gets a mutable reference to the LLVM module.
    pub fn module_mut(&mut self) -> &mut Module<'ctx> {
        &mut self.module
    }

    /// Gets a mutable reference to the LLVM builder.
    pub fn builder_mut(&mut self) -> &mut Builder<'ctx> {
        &mut self.builder
    }

    /// Converts a Typhon type to an LLVM type.
    pub fn convert_type(&self, ty: &TyphonType) -> CodeGenResult<BasicTypeEnum<'ctx>> {
        match ty {
            TyphonType::Primitive(p) => {
                match p.kind {
                    PrimitiveTypeKind::Int => Ok(self.context.i64_type().into()),
                    PrimitiveTypeKind::Float => Ok(self.context.f64_type().into()),
                    PrimitiveTypeKind::Bool => Ok(self.context.bool_type().into()),
                    PrimitiveTypeKind::Str => {
                        // String is represented as a pointer to a char array
                        Ok(self.context.ptr_type(AddressSpace::default()).into())
                    }
                    PrimitiveTypeKind::Bytes => {
                        // Bytes is represented as a pointer to a byte array
                        Ok(self.context.ptr_type(AddressSpace::default()).into())
                    }
                }
            }
            TyphonType::Class(_) => {
                // Classes are represented as pointers to structures
                let _struct_type = self.create_class_struct(ty)?;
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::Function(func_type) => {
                // Function types are pointers to function values
                let _function_type = self.create_function_type(func_type)?;
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::Tuple(tuple_type) => {
                // Create a struct type for the tuple
                let mut element_types = Vec::new();
                for element_type in &tuple_type.element_types {
                    let llvm_type = self.convert_type(element_type)?;
                    element_types.push(llvm_type);
                }

                let tuple_struct = self.context.struct_type(&element_types, false);
                Ok(tuple_struct.into())
            }
            TyphonType::List(list_type) => {
                // Lists are represented as a structure containing:
                // - A pointer to the data
                // - The length of the list
                // - The capacity of the list
                // Convert element type (needed for correct LLVM type registration)
                let _element_llvm_type = self.convert_type(&list_type.element_type)?;
                // Create a pointer to the element type
                // In LLVM 18, we need to convert to a specific type before calling ptr_type
                // So we create a generic pointer type instead
                let element_ptr_type = self.context.ptr_type(AddressSpace::default());

                let list_struct_type = self.context.struct_type(
                    &[
                        element_ptr_type.into(),
                        self.context.i64_type().into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );

                Ok(list_struct_type.into())
            }
            TyphonType::Union(_) => {
                // Union types are implemented using a tagged union
                // This is a simplification - a proper implementation would use LLVM's type-based alias analysis
                Err(CodeGenError::unsupported_feature(
                    "Union types are not yet supported in code generation",
                    None,
                ))
            }
            TyphonType::TypeVar(_) => Err(CodeGenError::type_conversion_error(
                "Type variables should be resolved before code generation",
                None,
            )),
            TyphonType::GenericInstance(_) => Err(CodeGenError::type_conversion_error(
                "Generic instances should be monomorphized before code generation",
                None,
            )),
            TyphonType::Any => {
                // Any is represented as a pointer to i8 (void*)
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::None => {
                // None is represented as a null pointer
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::Never => Err(CodeGenError::type_conversion_error(
                "Cannot convert Never type to LLVM type",
                None,
            )),
        }
    }

    /// Creates a class structure type.
    fn create_class_struct<'a>(&'a self, ty: &TyphonType) -> CodeGenResult<StructType<'a>> {
        if let TyphonType::Class(class_type) = ty {
            // Create fields for the class
            let mut field_types = Vec::new();

            // First field is the vtable pointer for method dispatch
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            field_types.push(ptr_type.into());

            // Add fields for instance variables
            for field_type in class_type.fields.values() {
                let llvm_type = self.convert_type(field_type)?;
                field_types.push(llvm_type);
            }

            // Create the struct type with name
            let struct_name = format!("class.{}", class_type.name);
            // In LLVM 18, we should use identified structs properly
            let struct_type = if let Some(existing) = self.context.get_struct_type(&struct_name) {
                // Reuse the existing type if it exists
                existing.set_body(&field_types, false);
                existing
            } else {
                // Create a new identified struct type
                let new_type = self.context.opaque_struct_type(&struct_name);
                new_type.set_body(&field_types, false);
                new_type
            };

            Ok(struct_type)
        } else {
            Err(CodeGenError::type_conversion_error(
                "Expected a class type",
                None,
            ))
        }
    }

    /// Creates an LLVM function type from a Typhon function type.
    fn create_function_type(
        &self,
        func_type: &TyphonFunctionType,
    ) -> CodeGenResult<FunctionType<'ctx>> {
        // Convert parameter types
        let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
        for param in &func_type.parameters {
            let llvm_type = self.convert_type(&param.ty)?;
            param_types.push(llvm_type.into());
        }

        // Convert return type
        let return_type = self.convert_type(&func_type.return_type)?;

        // Create function type
        let function_type = match return_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, false),
        };

        Ok(function_type)
    }

    /// Optimizes the module.
    pub fn optimize_module(&self) {
        // In LLVM 18, we need to use the PassBuilderOptions approach
        // instead of the individual pass methods like add_instruction_combining_pass()
        let pass_builder_options = PassBuilderOptions::create();

        // Configure optimization options
        pass_builder_options.set_loop_vectorization(true);
        pass_builder_options.set_loop_unrolling(true);
        pass_builder_options.set_loop_slp_vectorization(true);

        // Run the pass manager on the module
        self.pass_manager.run_on(&self.module);
    }

    /// Writes the LLVM IR to a file.
    pub fn write_ir_to_file(&self, path: &Path) -> CodeGenResult<()> {
        if let Err(e) = self.module.print_to_file(path) {
            return Err(CodeGenError::code_gen_error(
                format!("Failed to write IR to file: {e}"),
                None,
            ));
        }
        Ok(())
    }

    /// Compiles the module to an object file.
    pub fn compile_to_object(&self, path: &Path) -> CodeGenResult<()> {
        // Get the host triple
        let triple = TargetMachine::get_default_triple();

        // Get the target
        let target = Target::from_triple(&triple)
            .map_err(|e| CodeGenError::llvm_setup_error(format!("Failed to get target: {e}")))?;

        // Create a target machine
        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| {
                CodeGenError::llvm_setup_error("Failed to create target machine".to_string())
            })?;

        // Compile the module to an object file
        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| {
                CodeGenError::code_gen_error(format!("Failed to write object file: {e}"), None)
            })?;

        Ok(())
    }
}
