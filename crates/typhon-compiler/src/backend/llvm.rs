// Copyright (c) 2024 The Typhon Project
// SPDX-License-Identifier: Apache-2.0
//! LLVM wrapper for the Typhon compiler.
//!
//! This module provides a safe wrapper around the LLVM C API for code generation.

use std::path::Path;
use std::rc::Rc;

use inkwell::{
    AddressSpace,
    FloatPredicate,
    IntPredicate,
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassManager,
    targets::{
        CodeModel,
        FileType,
        InitializationConfig,
        RelocMode,
        Target,
        TargetMachine,
        TargetTriple,
    },
    types::{
        AnyTypeEnum,
        BasicMetadataTypeEnum,
        BasicTypeEnum,
        FunctionType,
        PointerType,
        StructType,
    },
    values::{
        BasicMetadataValueEnum,
        BasicValueEnum,
        FunctionValue,
        PointerValue,
        StructValue,
    },
};

use crate::backend::error::{
    CodeGenError,
    CodeGenResult,
};
use crate::typesystem::types::{
    ClassType,
    FunctionType as TyphonFunctionType,
    PrimitiveTypeKind,
    Type as TyphonType,
};

/// LLVM context wrapper.
pub struct LLVMContext {
    /// The LLVM context.
    context: Context,
    /// The LLVM module.
    module: Module,
    /// The LLVM builder.
    builder: Builder,
    /// The LLVM pass manager.
    pass_manager: PassManager<Module>,
}

impl LLVMContext {
    /// Creates a new LLVM context.
    pub fn new(module_name: &str) -> Self {
        let context = Context::create();
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let pass_manager = PassManager::create(&module);

        // Add optimization passes
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        pass_manager.add_gvn_pass();
        pass_manager.add_cfg_simplification_pass();
        pass_manager.add_basic_alias_analysis_pass();
        pass_manager.add_promote_memory_to_register_pass();
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();

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
        &self.context
    }

    /// Gets the LLVM module.
    pub fn module(&self) -> &Module {
        &self.module
    }

    /// Gets the LLVM builder.
    pub fn builder(&self) -> &Builder {
        &self.builder
    }

    /// Gets a mutable reference to the LLVM module.
    pub fn module_mut(&mut self) -> &mut Module {
        &mut self.module
    }

    /// Gets a mutable reference to the LLVM builder.
    pub fn builder_mut(&mut self) -> &mut Builder {
        &mut self.builder
    }

    /// Converts a Typhon type to an LLVM type.
    pub fn convert_type(&self, ty: &TyphonType) -> CodeGenResult<BasicTypeEnum> {
        match ty {
            TyphonType::Primitive(p) => {
                match p.kind {
                    PrimitiveTypeKind::Int => Ok(self.context.i64_type().into()),
                    PrimitiveTypeKind::Float => Ok(self.context.f64_type().into()),
                    PrimitiveTypeKind::Bool => Ok(self.context.bool_type().into()),
                    PrimitiveTypeKind::Str => {
                        // String is represented as a pointer to a char array
                        let i8_type = self.context.i8_type();
                        Ok(i8_type.ptr_type(AddressSpace::default()).into())
                    }
                    PrimitiveTypeKind::Bytes => {
                        // Bytes is represented as a pointer to a byte array
                        let i8_type = self.context.i8_type();
                        Ok(i8_type.ptr_type(AddressSpace::default()).into())
                    }
                }
            }
            TyphonType::Class(_) => {
                // Classes are represented as pointers to structures
                let struct_type = self.create_class_struct(ty)?;
                Ok(struct_type.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::Function(func_type) => {
                // Function types are pointers to function values
                let function_type = self.create_function_type(func_type)?;
                Ok(function_type.ptr_type(AddressSpace::default()).into())
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
                let element_llvm_type = self.convert_type(&list_type.element_type)?;
                let element_ptr_type = element_llvm_type.ptr_type(AddressSpace::default());

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
                let i8_type = self.context.i8_type();
                Ok(i8_type.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::None => {
                // None is represented as a null pointer
                let i8_type = self.context.i8_type();
                Ok(i8_type.ptr_type(AddressSpace::default()).into())
            }
            TyphonType::Never => Err(CodeGenError::type_conversion_error(
                "Cannot convert Never type to LLVM type",
                None,
            )),
        }
    }

    /// Creates a class structure type.
    fn create_class_struct(&self, ty: &TyphonType) -> CodeGenResult<StructType> {
        if let TyphonType::Class(class_type) = ty {
            // Create fields for the class
            let mut field_types = Vec::new();

            // First field is the vtable pointer for method dispatch
            let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            field_types.push(i8_ptr_type.into());

            // Add fields for instance variables
            for (_, field_type) in &class_type.fields {
                let llvm_type = self.convert_type(field_type)?;
                field_types.push(llvm_type);
            }

            // Create the struct type
            let struct_name = format!("class.{}", class_type.name);
            let struct_type = self.context.struct_type(&field_types, false);
            self.module.add_struct_named(&struct_name, struct_type);

            Ok(struct_type)
        } else {
            Err(CodeGenError::type_conversion_error(
                "Expected a class type",
                None,
            ))
        }
    }

    /// Creates an LLVM function type from a Typhon function type.
    fn create_function_type(&self, func_type: &TyphonFunctionType) -> CodeGenResult<FunctionType> {
        // Convert parameter types
        let mut param_types = Vec::new();
        for param in &func_type.parameters {
            let llvm_type = self.convert_type(&param.ty)?;
            param_types.push(llvm_type);
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
        };

        Ok(function_type)
    }

    /// Optimizes the module.
    pub fn optimize_module(&self) {
        self.pass_manager.run_on(&self.module);
    }

    /// Writes the LLVM IR to a file.
    pub fn write_ir_to_file(&self, path: &Path) -> CodeGenResult<()> {
        if let Err(e) = self.module.print_to_file(path) {
            return Err(CodeGenError::code_gen_error(
                format!("Failed to write IR to file: {}", e),
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
            .map_err(|e| CodeGenError::llvm_setup_error(format!("Failed to get target: {}", e)))?;

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
                CodeGenError::code_gen_error(format!("Failed to write object file: {}", e), None)
            })?;

        Ok(())
    }
}
