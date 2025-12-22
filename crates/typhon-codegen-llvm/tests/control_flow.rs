//! Control flow translation tests.
//!
//! Tests for MIR basic block and terminator translation to LLVM IR.

use inkwell::context::Context;
use typhon_codegen_llvm::blocks::{create_basic_blocks, translate_blocks};
use typhon_codegen_llvm::context::CodegenContext;
use typhon_codegen_llvm::error::CodegenError;
use typhon_codegen_llvm::instructions::{translate_instruction, translate_terminator};
use typhon_codegen_llvm::runtime::declare_runtime_functions;
use typhon_mir::block::BasicBlock;
use typhon_mir::instr::{BasicBlockID, MIRConst, MIRInstr, Terminator, ValueID};
use typhon_mir::types::MIRType;

/// Helper to create a test context with runtime functions declared.
#[allow(unsafe_code)]
fn create_test_context() -> (Context, CodegenContext<'static>) {
    let context = Box::leak(Box::new(Context::create()));
    let module = context.create_module("test");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(context, module, builder);

    declare_runtime_functions(&mut ctx).unwrap();

    // SAFETY: We're creating a copy of the context for test purposes.
    // The original context is leaked and remains valid for 'static lifetime.
    (unsafe { std::ptr::read(&raw const *context) }, ctx)
}

/// Helper to create a simple LLVM function for testing.
fn create_test_function<'ctx>(
    ctx: &CodegenContext<'ctx>,
    name: &str,
) -> inkwell::values::FunctionValue<'ctx> {
    let void_type = ctx.context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    ctx.module.add_function(name, fn_type, None)
}

#[test]
fn test_create_basic_blocks_single() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");
    let block = BasicBlock {
        id: BasicBlockID(0),
        instrs: vec![],
        terminator: Terminator::Return(None),
        landing_pad: None,
        predecessors: vec![],
        successors: vec![],
    };

    let result = create_basic_blocks(&mut ctx, &[block], func);

    assert!(result.is_ok());

    // Verify block was created
    let bb = ctx.get_block(BasicBlockID(0));

    assert!(bb.is_ok());
}

#[test]
fn test_create_basic_blocks_multiple() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");
    let blocks = vec![
        BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![],
            terminator: Terminator::Branch(BasicBlockID(1)),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![BasicBlockID(1)],
        },
        BasicBlock {
            id: BasicBlockID(1),
            instrs: vec![],
            terminator: Terminator::Return(None),
            landing_pad: None,
            predecessors: vec![BasicBlockID(0)],
            successors: vec![],
        },
    ];

    let result = create_basic_blocks(&mut ctx, &blocks, func);

    assert!(result.is_ok());

    // Verify all blocks were created
    assert!(ctx.get_block(BasicBlockID(0)).is_ok());
    assert!(ctx.get_block(BasicBlockID(1)).is_ok());
}

#[test]
fn test_return_terminator_with_value() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create a value to return
    let value_id = ValueID(0);
    let bool_val = ctx.context.bool_type().const_int(1, false);

    ctx.set_value(value_id, bool_val.into());

    // Create block
    let bb = ctx.context.append_basic_block(func, "bb0");

    ctx.builder.position_at_end(bb);

    let terminator = Terminator::Return(Some(value_id));
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_return_terminator_void() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create block
    let bb = ctx.context.append_basic_block(func, "bb0");

    ctx.builder.position_at_end(bb);

    let terminator = Terminator::Return(None);
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_branch_terminator() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create two blocks
    let bb0 = ctx.context.append_basic_block(func, "bb0");
    let bb1 = ctx.context.append_basic_block(func, "bb1");

    ctx.set_block(BasicBlockID(0), bb0);
    ctx.set_block(BasicBlockID(1), bb1);
    ctx.builder.position_at_end(bb0);

    let terminator = Terminator::Branch(BasicBlockID(1));
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_cond_branch_with_i1() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create condition value (i1)
    let value_id = ValueID(0);
    let bool_val = ctx.context.bool_type().const_int(1, false);

    ctx.set_value(value_id, bool_val.into());

    // Create blocks
    let bb0 = ctx.context.append_basic_block(func, "bb0");
    let bb1 = ctx.context.append_basic_block(func, "bb1");
    let bb2 = ctx.context.append_basic_block(func, "bb2");

    ctx.set_block(BasicBlockID(0), bb0);
    ctx.set_block(BasicBlockID(1), bb1);
    ctx.set_block(BasicBlockID(2), bb2);
    ctx.builder.position_at_end(bb0);

    let terminator = Terminator::CondBranch {
        condition: value_id,
        then_block: BasicBlockID(1),
        else_block: BasicBlockID(2),
    };
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_cond_branch_with_object() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create condition value (pointer - object)
    let value_id = ValueID(0);
    let ptr_type = ctx.context.ptr_type(inkwell::AddressSpace::default());
    let ptr_val = ptr_type.const_null();

    ctx.set_value(value_id, ptr_val.into());

    // Create blocks
    let bb0 = ctx.context.append_basic_block(func, "bb0");
    let bb1 = ctx.context.append_basic_block(func, "bb1");
    let bb2 = ctx.context.append_basic_block(func, "bb2");

    ctx.set_block(BasicBlockID(0), bb0);
    ctx.set_block(BasicBlockID(1), bb1);
    ctx.set_block(BasicBlockID(2), bb2);
    ctx.builder.position_at_end(bb0);

    let terminator = Terminator::CondBranch {
        condition: value_id,
        then_block: BasicBlockID(1),
        else_block: BasicBlockID(2),
    };
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_phi_node() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create values from different blocks
    let value1 = ValueID(0);
    let value2 = ValueID(1);
    let bool_val1 = ctx.context.bool_type().const_int(1, false);
    let bool_val2 = ctx.context.bool_type().const_int(0, false);

    ctx.set_value(value1, bool_val1.into());
    ctx.set_value(value2, bool_val2.into());

    // Create blocks
    let bb0 = ctx.context.append_basic_block(func, "bb0");
    let bb1 = ctx.context.append_basic_block(func, "bb1");
    let bb2 = ctx.context.append_basic_block(func, "bb2");

    ctx.set_block(BasicBlockID(0), bb0);
    ctx.set_block(BasicBlockID(1), bb1);
    ctx.set_block(BasicBlockID(2), bb2);
    ctx.builder.position_at_end(bb2);

    let phi_instr = MIRInstr::Phi {
        incoming: vec![(BasicBlockID(0), value1), (BasicBlockID(1), value2)],
        ty: MIRType::Bool,
    };
    let result = translate_instruction(&mut ctx, &phi_instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_unreachable_terminator() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create block
    let bb = ctx.context.append_basic_block(func, "bb0");

    ctx.builder.position_at_end(bb);

    let terminator = Terminator::Unreachable;
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_raise_terminator() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create exception value
    let value_id = ValueID(0);
    let ptr_type = ctx.context.ptr_type(inkwell::AddressSpace::default());
    let ptr_val = ptr_type.const_null();

    ctx.set_value(value_id, ptr_val.into());

    // Create block
    let bb = ctx.context.append_basic_block(func, "bb0");

    ctx.builder.position_at_end(bb);

    let terminator = Terminator::Raise(value_id);
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_ok());
}

#[test]
fn test_invoke_terminator_not_implemented() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create callee value
    let value_id = ValueID(0);
    let ptr_type = ctx.context.ptr_type(inkwell::AddressSpace::default());
    let ptr_val = ptr_type.const_null();

    ctx.set_value(value_id, ptr_val.into());

    // Create blocks
    let bb0 = ctx.context.append_basic_block(func, "bb0");
    let bb1 = ctx.context.append_basic_block(func, "bb1");
    let bb2 = ctx.context.append_basic_block(func, "bb2");

    ctx.set_block(BasicBlockID(0), bb0);
    ctx.set_block(BasicBlockID(1), bb1);
    ctx.set_block(BasicBlockID(2), bb2);
    ctx.builder.position_at_end(bb0);

    let terminator = Terminator::Invoke {
        callee: value_id,
        args: vec![],
        normal: BasicBlockID(1),
        unwind: BasicBlockID(2),
        ty: MIRType::None,
    };
    let result = translate_terminator(&mut ctx, &terminator);

    assert!(result.is_err());
    assert!(matches!(result, Err(CodegenError::InstructionTranslationError(_))));
}

#[test]
fn test_complete_if_else_cfg() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create condition value
    let cond_id = ValueID(0);
    let bool_val = ctx.context.bool_type().const_int(1, false);

    ctx.set_value(cond_id, bool_val.into());

    // Create blocks representing: if (condition) { block1 } else { block2 } return;
    let blocks = vec![
        BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![MIRInstr::Const(MIRConst::Bool(true))],
            terminator: Terminator::CondBranch {
                condition: cond_id,
                then_block: BasicBlockID(1),
                else_block: BasicBlockID(2),
            },
            landing_pad: None,
            predecessors: vec![],
            successors: vec![BasicBlockID(1), BasicBlockID(2)],
        },
        BasicBlock {
            id: BasicBlockID(1),
            instrs: vec![],
            terminator: Terminator::Branch(BasicBlockID(3)),
            landing_pad: None,
            predecessors: vec![BasicBlockID(0)],
            successors: vec![BasicBlockID(3)],
        },
        BasicBlock {
            id: BasicBlockID(2),
            instrs: vec![],
            terminator: Terminator::Branch(BasicBlockID(3)),
            landing_pad: None,
            predecessors: vec![BasicBlockID(0)],
            successors: vec![BasicBlockID(3)],
        },
        BasicBlock {
            id: BasicBlockID(3),
            instrs: vec![],
            terminator: Terminator::Return(None),
            landing_pad: None,
            predecessors: vec![BasicBlockID(1), BasicBlockID(2)],
            successors: vec![],
        },
    ];
    let result = create_basic_blocks(&mut ctx, &blocks, func);

    assert!(result.is_ok());

    let result = translate_blocks(&mut ctx, &blocks);

    assert!(result.is_ok());
}

#[test]
fn test_complete_loop_cfg() {
    let (_context, mut ctx) = create_test_context();
    let func = create_test_function(&ctx, "test_func");

    // Create condition value
    let cond_id = ValueID(0);
    let bool_val = ctx.context.bool_type().const_int(1, false);

    ctx.set_value(cond_id, bool_val.into());

    // Create blocks representing: while (condition) { body } return;
    let blocks = vec![
        BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![MIRInstr::Const(MIRConst::Bool(true))],
            terminator: Terminator::CondBranch {
                condition: cond_id,
                then_block: BasicBlockID(1),
                else_block: BasicBlockID(2),
            },
            landing_pad: None,
            predecessors: vec![BasicBlockID(1)],
            successors: vec![BasicBlockID(1), BasicBlockID(2)],
        },
        BasicBlock {
            id: BasicBlockID(1),
            instrs: vec![],
            terminator: Terminator::Branch(BasicBlockID(0)),
            landing_pad: None,
            predecessors: vec![BasicBlockID(0)],
            successors: vec![BasicBlockID(0)],
        },
        BasicBlock {
            id: BasicBlockID(2),
            instrs: vec![],
            terminator: Terminator::Return(None),
            landing_pad: None,
            predecessors: vec![BasicBlockID(0)],
            successors: vec![],
        },
    ];
    let result = create_basic_blocks(&mut ctx, &blocks, func);

    assert!(result.is_ok());

    let result = translate_blocks(&mut ctx, &blocks);

    assert!(result.is_ok());
}
