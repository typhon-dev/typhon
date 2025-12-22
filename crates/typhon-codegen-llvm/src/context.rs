//! Code generation context management.
//!
//! This module provides the [`CodegenContext`] struct which manages all state
//! during LLVM IR generation from MIR.

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use rustc_hash::FxHashMap;
use typhon_mir::instr::{BasicBlockID, LocalID, ValueID};
use typhon_mir::types::MIRType;

use crate::error::{CodegenError, CodegenResult};

/// Central state management for code generation.
///
/// This struct holds all the state needed during code generation including:
/// - LLVM context, module, and builder
/// - Type caches for performance
/// - Value and local variable tracking
/// - Basic block mapping
/// - Runtime function declarations
#[derive(Debug)]
pub struct CodegenContext<'ctx> {
    /// LLVM context reference
    pub context: &'ctx Context,
    /// LLVM module being generated
    pub module: Module<'ctx>,
    /// LLVM instruction builder
    pub builder: Builder<'ctx>,
    /// Cache of translated MIR types to LLVM types
    pub type_cache: FxHashMap<MIRType, BasicTypeEnum<'ctx>>,
    /// Cache of named struct types (e.g. `TyphonInt`, `TyphonList`)
    pub struct_types: FxHashMap<String, StructType<'ctx>>,
    /// Mapping from MIR `ValueID`s to LLVM values (SSA values)
    pub values: FxHashMap<ValueID, BasicValueEnum<'ctx>>,
    /// Mapping from MIR `LocalID`s to LLVM stack allocations
    pub locals: FxHashMap<LocalID, PointerValue<'ctx>>,
    /// Mapping from MIR `BasicBlockID`s to LLVM basic blocks
    pub blocks: FxHashMap<BasicBlockID, BasicBlock<'ctx>>,
    /// Cache of runtime function declarations
    pub runtime_functions: FxHashMap<String, FunctionValue<'ctx>>,
    /// Currently active function being generated
    pub current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// Create a new code generation context.
    ///
    /// ## Arguments
    ///
    /// - `context` - LLVM context reference
    /// - `module` - LLVM module to generate code into
    /// - `builder` - LLVM instruction builder
    pub fn new(context: &'ctx Context, module: Module<'ctx>, builder: Builder<'ctx>) -> Self {
        Self {
            context,
            module,
            builder,
            type_cache: FxHashMap::default(),
            struct_types: FxHashMap::default(),
            values: FxHashMap::default(),
            locals: FxHashMap::default(),
            blocks: FxHashMap::default(),
            runtime_functions: FxHashMap::default(),
            current_function: None,
        }
    }

    /// Cache a translated MIR type to avoid redundant translation.
    ///
    /// ## Arguments
    ///
    /// - `ty` - MIR type to cache
    /// - `llvm_type` - Corresponding LLVM type
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let llvm_type = translate_type_uncached(&ctx, &mir_type)?;
    /// ctx.cache_type(mir_type.clone(), llvm_type);
    /// ```
    pub fn cache_type(&mut self, ty: MIRType, llvm_type: BasicTypeEnum<'ctx>) {
        self.type_cache.insert(ty, llvm_type);
    }

    /// Get a basic block by MIR `BasicBlockID`.
    ///
    /// ## Arguments
    ///
    /// - `block_id` - MIR basic block ID
    ///
    /// ## Returns
    ///
    /// The LLVM basic block.
    ///
    /// ## Errors
    ///
    /// Returns [`CodegenError::BlockNotFound`] if the block was not created.
    pub fn get_block(&self, block_id: BasicBlockID) -> CodegenResult<BasicBlock<'ctx>> {
        self.blocks.get(&block_id).copied().ok_or(CodegenError::BlockNotFound(block_id))
    }

    /// Get a local variable allocation by MIR `LocalID`.
    ///
    /// ## Arguments
    ///
    /// - `local` - MIR local variable ID
    ///
    /// ## Returns
    ///
    /// Pointer to the stack allocation for this local.
    ///
    /// ## Errors
    ///
    /// Returns [`CodegenError::LocalNotFound`] if the local was not allocated.
    pub fn get_local(&self, local: LocalID) -> CodegenResult<PointerValue<'ctx>> {
        self.locals.get(&local).copied().ok_or(CodegenError::LocalNotFound(local))
    }

    /// Get a runtime function by name.
    ///
    /// ## Arguments
    ///
    /// - `name` - Runtime function name (e.g. `typhon_add`)
    ///
    /// ## Returns
    ///
    /// The function value if found, or an error if not found.
    ///
    /// ## Errors
    ///
    /// Returns [`CodegenError::RuntimeFunctionNotFound`] if the function was not declared.
    pub fn get_runtime_function(&self, name: &str) -> CodegenResult<FunctionValue<'ctx>> {
        self.runtime_functions
            .get(name)
            .copied()
            .ok_or_else(|| CodegenError::RuntimeFunctionNotFound(name.to_string()))
    }

    /// Get a previously declared struct type by name.
    ///
    /// ## Arguments
    ///
    /// - `name` - Name of the struct type (e.g. `TyphonInt`)
    ///
    /// ## Returns
    ///
    /// The struct type if found, or an error if not found.
    ///
    /// ## Errors
    ///
    /// Returns [`CodegenError::StructTypeNotFound`] if the struct type was not declared.
    pub fn get_struct_type(&self, name: &str) -> CodegenResult<StructType<'ctx>> {
        self.struct_types
            .get(name)
            .copied()
            .ok_or_else(|| CodegenError::StructTypeNotFound(name.to_string()))
    }

    /// Get an SSA value by MIR `ValueID`.
    ///
    /// ## Arguments
    ///
    /// - `value_id` - MIR value ID
    ///
    /// ## Returns
    ///
    /// The LLVM value.
    ///
    /// ## Errors
    ///
    /// Returns [`CodegenError::ValueNotFound`] if the value was not generated.
    pub fn get_value(&self, value_id: ValueID) -> CodegenResult<BasicValueEnum<'ctx>> {
        self.values.get(&value_id).copied().ok_or(CodegenError::ValueNotFound(value_id))
    }

    /// Store a basic block mapped to a MIR `BasicBlockID`.
    ///
    /// ## Arguments
    ///
    /// - `block_id` - MIR basic block ID
    /// - `block` - LLVM basic block to store
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let llvm_block = ctx.context.append_basic_block(function, "bb0");
    /// ctx.set_block(block_id, llvm_block);
    /// ```
    pub fn set_block(&mut self, block_id: BasicBlockID, block: BasicBlock<'ctx>) {
        self.blocks.insert(block_id, block);
    }

    /// Store an SSA value mapped to a MIR `ValueID`.
    ///
    /// ## Arguments
    ///
    /// - `value_id` - MIR value ID
    /// - `value` - LLVM value to store
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let llvm_value = translate_instruction(&mut ctx, &instr)?;
    /// ctx.set_value(value_id, llvm_value);
    /// ```
    pub fn set_value(&mut self, value_id: ValueID, value: BasicValueEnum<'ctx>) {
        self.values.insert(value_id, value);
    }
}
