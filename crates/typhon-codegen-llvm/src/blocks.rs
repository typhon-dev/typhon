//! Basic block translation.
//!
//! This module handles the translation of MIR basic blocks to LLVM IR.
//! Basic blocks are single-entry, single-exit sequences of instructions with
//! explicit control flow terminators.

use inkwell::values::FunctionValue;
use typhon_mir::block::BasicBlock as MIRBasicBlock;

use crate::context::CodegenContext;
use crate::error::CodegenResult;
use crate::instructions::{translate_instruction, translate_terminator};

/// Creates LLVM basic blocks for all MIR basic blocks.
///
/// This function pre-creates all LLVM basic blocks and stores them in the context
/// before translating any instructions. This two-phase approach is necessary because
/// branch instructions need to reference target blocks that may not have been
/// translated yet.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `mir_blocks` - Slice of MIR basic blocks to create
/// - `llvm_func` - LLVM function to append blocks to
///
/// ## Returns
///
/// `Ok(())` if all blocks were created successfully.
///
/// ## Errors
///
/// Returns an error if LLVM block creation fails.
///
/// ## Example
///
/// ```rust,ignore
/// let mir_blocks = vec![block0, block1, block2];
/// create_basic_blocks(&mut ctx, &mir_blocks, llvm_func)?;
/// // Now all blocks are available for branch targets
/// ```
pub fn create_basic_blocks<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    mir_blocks: &[MIRBasicBlock],
    llvm_func: FunctionValue<'ctx>,
) -> CodegenResult<()> {
    for block in mir_blocks {
        let bb_name = format!("bb{}", block.id.0);
        let llvm_bb = ctx.context.append_basic_block(llvm_func, &bb_name);
        ctx.set_block(block.id, llvm_bb);
    }

    Ok(())
}

/// Translates all MIR basic blocks to LLVM IR.
///
/// For each basic block, this function:
/// 1. Positions the builder at the start of the block
/// 2. Translates all instructions in sequence
/// 3. Translates the terminator instruction
///
/// Instructions that produce values are stored in the context with their `ValueID`.
/// Each instruction in a block is assigned a sequential `ValueID` starting from 0.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `mir_blocks` - Slice of MIR basic blocks to translate
///
/// ## Returns
///
/// `Ok(())` if all blocks were translated successfully.
///
/// ## Errors
///
/// Returns an error if:
///
/// - A block is not found in the context
/// - Instruction translation fails
/// - Terminator translation fails
///
/// ## Example
///
/// ```rust,ignore
/// create_basic_blocks(&mut ctx, &mir_blocks, llvm_func)?;
/// translate_blocks(&mut ctx, &mir_blocks)?;
/// ```
pub fn translate_blocks(
    ctx: &mut CodegenContext<'_>,
    mir_blocks: &[MIRBasicBlock],
) -> CodegenResult<()> {
    // Track the next value ID across all blocks
    let mut next_value_id = 0u32;

    for block in mir_blocks {
        let llvm_bb = ctx.get_block(block.id)?;
        ctx.builder.position_at_end(llvm_bb);

        // Translate instructions and track their value IDs
        for instr in &block.instrs {
            if let Some(value) = translate_instruction(ctx, instr)? {
                // Store the value with its sequential ID
                ctx.set_value(typhon_mir::instr::ValueID(next_value_id), value);
                next_value_id += 1;
            }
        }

        // Translate terminator
        translate_terminator(ctx, &block.terminator)?;
    }

    Ok(())
}
