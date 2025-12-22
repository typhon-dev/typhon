//! Tests for MIR instruction translation to LLVM IR.

use std::f64;

use inkwell::AddressSpace;
use inkwell::context::Context;
use typhon_codegen_llvm::context::CodegenContext;
use typhon_codegen_llvm::instructions::translate_instruction;
use typhon_codegen_llvm::runtime::declare_runtime_functions;
use typhon_mir::instr::{BinOpKind, LocalID, MIRConst, MIRInstr, ValueID};
use typhon_mir::types::MIRType;

/// Helper function to set up a test context with runtime functions declared.
fn setup_context() -> CodegenContext<'static> {
    let context = Box::leak(Box::new(Context::create()));
    let module = context.create_module("test");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(context, module, builder);

    // Declare runtime functions
    declare_runtime_functions(&mut ctx).expect("Failed to declare runtime functions");

    // Create a test function with a basic block so the builder has somewhere to emit instructions
    let void_type = ctx.context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let function = ctx.module.add_function("test_fn", fn_type, None);
    let basic_block = ctx.context.append_basic_block(function, "entry");

    ctx.builder.position_at_end(basic_block);

    ctx
}

// ===== Constant Translation Tests =====

#[test]
fn test_translate_const_bool_true() {
    let mut ctx = setup_context();
    let instr = MIRInstr::Const(MIRConst::Bool(true));
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_some());

    let llvm_value = value.unwrap();
    assert!(llvm_value.is_int_value());
}

#[test]
fn test_translate_const_bool_false() {
    let mut ctx = setup_context();
    let instr = MIRInstr::Const(MIRConst::Bool(false));
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());

    let llvm_value = value.unwrap();

    assert!(llvm_value.is_int_value());
}

#[test]
fn test_translate_const_int() {
    let mut ctx = setup_context();
    let instr = MIRInstr::Const(MIRConst::Int(42));
    let result = translate_instruction(&mut ctx, &instr);

    if let Err(e) = &result {
        eprintln!("Error: {e:?}");
    }

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());

    // Should be a pointer to a boxed int object
    let llvm_value = value.unwrap();

    assert!(llvm_value.is_pointer_value());
}

#[test]
fn test_translate_const_float() {
    let mut ctx = setup_context();
    let instr = MIRInstr::Const(MIRConst::Float(f64::consts::PI));
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());

    // Should be a pointer to a boxed float object
    let llvm_value = value.unwrap();

    assert!(llvm_value.is_pointer_value());
}

#[test]
fn test_translate_const_string() {
    let mut ctx = setup_context();
    let instr = MIRInstr::Const(MIRConst::Str("hello".to_string()));
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());

    // Should be a pointer to a boxed string object
    let llvm_value = value.unwrap();

    assert!(llvm_value.is_pointer_value());
}

#[test]
fn test_translate_const_none() {
    let mut ctx = setup_context();
    let instr = MIRInstr::Const(MIRConst::None);
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());

    // Should be a pointer to the None singleton
    let llvm_value = value.unwrap();

    assert!(llvm_value.is_pointer_value());
}

// ===== Memory Operations Tests =====

#[test]
fn test_translate_store() {
    let mut ctx = setup_context();

    // Create a local variable (alloca)
    let local_id = LocalID(0);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let alloca = ctx.builder.build_alloca(ptr_type, "local").unwrap();
    ctx.locals.insert(local_id, alloca);

    // Create a value to store
    let value_id = ValueID(0);
    let int_val = ctx.context.i64_type().const_int(42, false);
    ctx.set_value(value_id, int_val.into());

    // Translate store instruction
    let instr = MIRInstr::Store { local: local_id, value: value_id };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    // Store returns None
    assert!(value.is_none());
}

#[test]
fn test_translate_load() {
    let mut ctx = setup_context();

    // Create a local variable (alloca) and store a value in it
    let local_id = LocalID(0);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let alloca = ctx.builder.build_alloca(ptr_type, "local").unwrap();

    ctx.locals.insert(local_id, alloca);

    // Store a value
    let int_val = ctx.context.i64_type().const_int(42, false);

    ctx.builder.build_store(alloca, int_val).unwrap();

    // Translate load instruction
    let instr = MIRInstr::Load { local: local_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());
}

// ===== Arithmetic Operations Tests =====

#[test]
fn test_translate_binop_add() {
    let mut ctx = setup_context();

    // Create two values
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    // Translate add instruction
    let instr = MIRInstr::BinOp { op: BinOpKind::Add, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    assert!(value.is_some());
}

#[test]
fn test_translate_binop_sub() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Sub, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_mul() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Mul, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_div() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Div, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_mod() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Mod, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_pow() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Pow, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

// ===== Comparison Operations Tests =====

#[test]
fn test_translate_binop_eq() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Eq, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Bool };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_ne() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Ne, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Bool };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_lt() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Lt, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Bool };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_le() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Le, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Bool };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_gt() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Gt, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Bool };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_translate_binop_ge() {
    let mut ctx = setup_context();
    let lhs_id = ValueID(0);
    let rhs_id = ValueID(1);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let lhs_val = ptr_type.const_null();
    let rhs_val = ptr_type.const_null();

    ctx.set_value(lhs_id, lhs_val.into());
    ctx.set_value(rhs_id, rhs_val.into());

    let instr = MIRInstr::BinOp { op: BinOpKind::Ge, lhs: lhs_id, rhs: rhs_id, ty: MIRType::Bool };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

// ===== Memory Management Tests =====

#[test]
fn test_translate_incref() {
    let mut ctx = setup_context();
    // Create a value
    let value_id = ValueID(0);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let val = ptr_type.const_null();

    ctx.set_value(value_id, val.into());

    // Translate incref instruction
    let instr = MIRInstr::IncRef(value_id);
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    // IncRef returns None
    assert!(value.is_none());
}

#[test]
fn test_translate_decref() {
    let mut ctx = setup_context();
    // Create a value
    let value_id = ValueID(0);
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());
    let val = ptr_type.const_null();

    ctx.set_value(value_id, val.into());

    // Translate decref instruction
    let instr = MIRInstr::DecRef(value_id);
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_ok());

    let value = result.unwrap();

    // DecRef returns None
    assert!(value.is_none());
}

// ===== Error Handling Tests =====

#[test]
fn test_unsupported_instruction() {
    let mut ctx = setup_context();

    // Try to translate an unsupported instruction (Call is not implemented yet)
    let instr = MIRInstr::Call { callee: ValueID(0), args: vec![], ty: MIRType::Void };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_err());
}

#[test]
fn test_missing_value_error() {
    let mut ctx = setup_context();

    // Try to use a value that doesn't exist
    let instr = MIRInstr::IncRef(ValueID(999));
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_err());
}

#[test]
fn test_missing_local_error() {
    let mut ctx = setup_context();

    // Try to load from a local that doesn't exist
    let instr = MIRInstr::Load { local: LocalID(999), ty: MIRType::Int };
    let result = translate_instruction(&mut ctx, &instr);

    assert!(result.is_err());
}
