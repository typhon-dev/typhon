//! Basic IR construction tests
//!
//! Tests for constructing simple MIR programs using the builder API.

use typhon_mir::builder::FunctionBuilder;
use typhon_mir::instr::{BinOpKind, LocalID, MIRConst, MIRInstr, Terminator, UnOpKind};
use typhon_mir::module::MIRModule;
use typhon_mir::types::MIRType;

#[test]
fn test_simple_arithmetic() {
    // Build: def add(a: Int, b: Int) -> Int: return a + b
    let mut builder = FunctionBuilder::new(
        "add".to_string(),
        vec![("a".to_string(), MIRType::Int), ("b".to_string(), MIRType::Int)],
        MIRType::Int,
    );

    let entry = builder.create_block();
    builder.switch_to_block(entry);

    // Load parameters
    let a_val = builder.add_instruction(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });

    let b_val = builder.add_instruction(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });

    // Add them
    let result = builder.add_instruction(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: a_val,
        rhs: b_val,
        ty: MIRType::Int,
    });

    // Return result
    builder.set_terminator(Terminator::Return(Some(result)));

    let function = builder.build();

    // Verify structure
    assert_eq!(function.name, "add");
    assert_eq!(function.params.len(), 2);
    assert_eq!(function.return_type, MIRType::Int);
    assert_eq!(function.blocks.len(), 1);
    assert_eq!(function.blocks[0].instrs.len(), 3);
}

#[test]
fn test_constant_expression() {
    // Build: def get_answer() -> Int: return 42
    let mut builder = FunctionBuilder::new("get_answer".to_string(), vec![], MIRType::Int);

    let entry = builder.create_block();
    builder.switch_to_block(entry);

    let const_val = builder.add_instruction(MIRInstr::Const(MIRConst::Int(42)));
    builder.set_terminator(Terminator::Return(Some(const_val)));

    let function = builder.build();

    assert_eq!(function.name, "get_answer");
    assert_eq!(function.params.len(), 0);
    assert_eq!(function.blocks[0].instrs.len(), 1);
}

#[test]
fn test_control_flow() {
    // Build: def abs(x: Int) -> Int:
    //     if x < 0:
    //         return -x
    //     return x
    let mut builder = FunctionBuilder::new(
        "abs".to_string(),
        vec![("x".to_string(), MIRType::Int)],
        MIRType::Int,
    );

    let entry = builder.create_block();
    let then_block = builder.create_block();
    let else_block = builder.create_block();

    // Entry block: load x and compare
    builder.switch_to_block(entry);
    let x_val = builder.add_instruction(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    let zero = builder.add_instruction(MIRInstr::Const(MIRConst::Int(0)));
    let cond = builder.add_instruction(MIRInstr::BinOp {
        op: BinOpKind::Lt,
        lhs: x_val,
        rhs: zero,
        ty: MIRType::Bool,
    });
    builder.set_terminator(Terminator::CondBranch { condition: cond, then_block, else_block });

    // Then block: return -x
    builder.switch_to_block(then_block);
    let x_val2 = builder.add_instruction(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    let neg_x = builder.add_instruction(MIRInstr::UnOp {
        op: UnOpKind::Neg,
        operand: x_val2,
        ty: MIRType::Int,
    });
    builder.set_terminator(Terminator::Return(Some(neg_x)));

    // Else block: return x
    builder.switch_to_block(else_block);
    let x_val3 = builder.add_instruction(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    builder.set_terminator(Terminator::Return(Some(x_val3)));

    let function = builder.build();

    assert_eq!(function.blocks.len(), 3);
    assert_eq!(function.name, "abs");
}

#[test]
fn test_module_creation() {
    let module = MIRModule {
        name: "test_module".to_string(),
        functions: vec![],
        globals: vec![],
        types: vec![],
    };

    assert_eq!(module.name, "test_module");
    assert!(module.functions.is_empty());
}

#[test]
fn test_pretty_print() {
    // Build simple function and test pretty printing
    let mut builder = FunctionBuilder::new(
        "add".to_string(),
        vec![("a".to_string(), MIRType::Int), ("b".to_string(), MIRType::Int)],
        MIRType::Int,
    );

    let entry = builder.create_block();
    builder.switch_to_block(entry);

    let a_val = builder.add_instruction(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });

    let b_val = builder.add_instruction(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });

    let result = builder.add_instruction(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: a_val,
        rhs: b_val,
        ty: MIRType::Int,
    });

    builder.set_terminator(Terminator::Return(Some(result)));

    let function = builder.build();
    let output = format!("{function}");

    // Verify output contains expected elements
    assert!(output.contains("func @add"));
    assert!(output.contains("%a: Int"));
    assert!(output.contains("%b: Int"));
    assert!(output.contains("-> Int"));
    assert!(output.contains("bb0:"));
    assert!(output.contains("load"));
    assert!(output.contains("binop"));
    assert!(output.contains("ret"));
}
