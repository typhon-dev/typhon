//! MIR function to LLVM function translation.
//!
//! This module handles translating MIR functions to LLVM functions, including:
//!
//! - Function signature translation (parameters and return type)
//! - Parameter setup with stack allocations
//! - Local variable allocation
//! - Basic block creation and instruction translation
//! - Function verification

use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::FunctionValue;
use typhon_mir::function::MIRFunction;
use typhon_mir::types::MIRType;

use crate::blocks::{create_basic_blocks, translate_blocks};
use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};
use crate::types::translate_type;

/// Allocates stack space for local variables.
///
/// This function allocates stack space for all local variables that are not parameters.
/// Parameters are already allocated in [`setup_parameters`], so they are skipped here.
///
/// Each local variable gets:
///
/// - Type translation from MIR type to LLVM type
/// - Stack allocation (`alloca`) with appropriate type
/// - Tracking in context for load/store operations
///
/// ## Arguments
///
/// - `ctx` - Mutable code generation context
/// - `mir_func` - MIR function containing local variable information
///
/// ## Returns
///
/// `Ok(())` if locals were allocated successfully.
///
/// ## Errors
///
/// Returns an error if:
///
/// - Type translation fails
/// - Stack allocation fails
///
/// ## Local Variable Indexing
///
/// Locals are indexed by their position in the `mir_func.locals` vector. The `LocalID`
/// values used throughout the function correspond to indices in this vector. Parameters
/// have already been allocated and tracked, so we skip any local that's already in the
/// context.
///
/// ## Example
///
/// ```rust,ignore
/// allocate_locals(&mut ctx, &mir_func)?;
/// // Now all locals can be accessed via Load/Store instructions
/// ```
pub fn allocate_locals(ctx: &mut CodegenContext<'_>, mir_func: &MIRFunction) -> CodegenResult<()> {
    for (idx, local) in mir_func.locals.iter().enumerate() {
        let local_id = typhon_mir::instr::LocalID(idx as u32);

        // Skip if already allocated (parameter)
        if ctx.locals.contains_key(&local_id) {
            continue;
        }

        let local_type = translate_type(ctx, &local.ty)?;
        let name = local.name.as_deref().unwrap_or("tmp");
        let alloca = ctx
            .builder
            .build_alloca(local_type, name)
            .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

        ctx.locals.insert(local_id, alloca);
    }

    Ok(())
}

/// Generates complete LLVM IR for a MIR function.
///
/// This orchestrates the entire function generation process:
///
/// 1. Translate function signature
/// 2. Create basic blocks
/// 3. Setup parameters with stack allocations
/// 4. Allocate local variables
/// 5. Translate instructions and terminators
/// 6. Verify generated function
///
/// The function generation process follows LLVM's requirements for well-formed IR:
///
/// - Parameters must be allocated on the stack (`alloca`) for mutable access
/// - All basic blocks must have terminators
/// - SSA values must dominate their uses
/// - Function must pass LLVM's verification
///
/// ## Arguments
///
/// - `ctx` - Mutable code generation context
/// - `mir_func` - MIR function to translate
///
/// ## Returns
///
/// The generated LLVM function value.
///
/// ## Errors
///
/// Returns error if any step fails or if LLVM function verification fails.
///
/// ## Example
///
/// ```rust,ignore
/// let llvm_func = generate_function(&mut ctx, &mir_func)?;
/// assert!(llvm_func.verify(true));
/// ```
pub fn generate_function<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    mir_func: &MIRFunction,
) -> CodegenResult<FunctionValue<'ctx>> {
    // 1. Translate signature and create function
    let llvm_func = translate_function_signature(ctx, mir_func)?;
    ctx.current_function = Some(llvm_func);

    // 2. Create basic blocks FIRST so we can position in them
    create_basic_blocks(ctx, &mir_func.blocks, llvm_func)?;

    // 3. Setup parameters with stack allocations at start of first block
    setup_parameters(ctx, mir_func, llvm_func)?;

    // 4. Allocate local variables
    allocate_locals(ctx, mir_func)?;

    // 5. Translate blocks (instructions and terminators)
    translate_blocks(ctx, &mir_func.blocks)?;

    // 6. Verify function
    if !llvm_func.verify(true) {
        return Err(CodegenError::InvalidFunction(mir_func.name.clone()));
    }

    Ok(llvm_func)
}

/// Sets up function parameters with stack allocations.
///
/// This function allocates stack slots for each parameter at the beginning of the
/// first basic block. Parameters must be stored on the stack to allow mutable access
/// and to maintain a uniform representation for all local variables.
///
/// The process:
///
/// 1. Position builder at start of first MIR basic block
/// 2. For each parameter:
///    - Get the LLVM parameter value
///    - Allocate stack space with the correct type
///    - Store parameter value to stack slot
///    - Track allocation in context for later use
///
/// ## Arguments
///
/// - `ctx` - Mutable code generation context
/// - `mir_func` - MIR function containing parameter information
/// - `llvm_func` - LLVM function to setup parameters for
///
/// ## Returns
///
/// `Ok(())` if parameters were setup successfully.
///
/// ## Errors
///
/// Returns an error if:
///
/// - Parameter not found at expected index
/// - Type translation fails
/// - Stack allocation (`alloca`) fails
/// - Store instruction fails
/// - First basic block not found
///
/// ## Example
///
/// ```rust,ignore
/// setup_parameters(&mut ctx, &mir_func, llvm_func)?;
/// // Now parameters can be loaded/stored like any local variable
/// ```
pub fn setup_parameters<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    mir_func: &MIRFunction,
    llvm_func: FunctionValue<'ctx>,
) -> CodegenResult<()> {
    // Position at the end of the first basic block (which should already be created)
    let first_block_id = mir_func
        .blocks
        .first()
        .ok_or(CodegenError::BlockNotFound(typhon_mir::instr::BasicBlockID(0)))?
        .id;
    let first_bb = ctx.get_block(first_block_id)?;

    // Position at the end of the first block for parameter allocations
    // Since blocks are initially empty, this will add allocas first, then instructions later
    ctx.builder.position_at_end(first_bb);

    // Create stack allocations for each parameter
    for (idx, param) in mir_func.params.iter().enumerate() {
        let llvm_param =
            llvm_func.get_nth_param(idx as u32).ok_or(CodegenError::ParameterNotFound(idx))?;

        let param_type = translate_type(ctx, &param.ty)?;
        let alloca = ctx
            .builder
            .build_alloca(param_type, &param.name)
            .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

        // Store parameter to stack slot
        ctx.builder
            .build_store(alloca, llvm_param)
            .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

        // Track local allocation
        ctx.locals.insert(param.local, alloca);
    }

    Ok(())
}

/// Translates a MIR function signature to LLVM function declaration.
///
/// This function creates the LLVM function type and adds the function to the module.
/// All Typhon object types are represented as pointers in LLVM, while primitive types
/// like booleans use native LLVM types.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `mir_func` - MIR function containing signature information
///
/// ## Returns
///
/// The LLVM function value with the translated signature.
///
/// ## Errors
///
/// Returns an error if:
///
/// - Parameter type translation fails
/// - Return type translation fails
/// - Function creation fails
///
/// ## Type Translation
///
/// - Parameters: Translated using the type translation module
/// - Return type: Void is handled specially, other types are translated normally
/// - Function calling convention: Uses default C ABI
///
/// ## Example
///
/// ```rust,ignore
/// // def add(a: Int, b: Int) -> Int
/// let llvm_func = translate_function_signature(&mut ctx, &mir_func)?;
/// // Creates: define %TyphonInt* @add(%TyphonInt* %a, %TyphonInt* %b)
/// ```
pub fn translate_function_signature<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    mir_func: &MIRFunction,
) -> CodegenResult<FunctionValue<'ctx>> {
    // Translate parameter types
    let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = mir_func
        .params
        .iter()
        .map(|p| translate_type(ctx, &p.ty).map(Into::into))
        .collect::<Result<Vec<_>, _>>()?;

    // Translate return type
    let fn_type = if mir_func.return_type == MIRType::Void {
        ctx.context.void_type().fn_type(&param_types, false)
    } else {
        let ret_ty = translate_type(ctx, &mir_func.return_type)?;
        ret_ty.fn_type(&param_types, false)
    };

    // Add function to module
    let fn_val = ctx.module.add_function(&mir_func.name, fn_type, None);

    Ok(fn_val)
}
