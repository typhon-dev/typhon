//! Tests for constant folding optimization pass.

use typhon_mir::instr::{BinOpKind, MIRConst, MIRInstr, ValueID};
use typhon_mir::types::MIRType;
use typhon_mir_optimizer::passes::ConstantFolder;

mod fixtures;

use fixtures::{count_const_instrs, create_function_with_instrs};

#[test]
fn test_arithmetic_folding() {
    // Create: 2 + 3 (as BinOp with constant operands)
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(2)),
        MIRInstr::Const(MIRConst::Int(3)),
        MIRInstr::BinOp { op: BinOpKind::Add, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Int },
    ];

    let mut func = create_function_with_instrs("test_arithmetic", instrs);

    let mut folder = ConstantFolder::new();
    let constants_folded = folder.fold_constants(&mut func).expect("folding failed");

    // Should have folded the addition
    assert!(constants_folded > 0, "should fold at least one constant");

    // After folding, we should have constant values
    let const_count = count_const_instrs(&func);

    assert!(const_count >= 2, "should have constant values");
}

#[test]
fn test_comparison_folding() {
    // Create: 5 > 3
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(5)),
        MIRInstr::Const(MIRConst::Int(3)),
        MIRInstr::BinOp { op: BinOpKind::Gt, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Bool },
    ];

    let mut func = create_function_with_instrs("test_comparison", instrs);

    let mut folder = ConstantFolder::new();
    let constants_folded = folder.fold_constants(&mut func).expect("folding failed");

    // Should have folded the comparison
    assert!(constants_folded > 0, "should fold at least one constant");

    // After folding, we should have constant values
    let const_count = count_const_instrs(&func);

    assert!(const_count >= 2, "should have constant values");
}

#[test]
fn test_logical_folding() {
    // Create: true && false
    let instrs = vec![
        MIRInstr::Const(MIRConst::Bool(true)),
        MIRInstr::Const(MIRConst::Bool(false)),
        MIRInstr::BinOp { op: BinOpKind::And, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Bool },
    ];

    let mut func = create_function_with_instrs("test_logical", instrs);

    let mut folder = ConstantFolder::new();
    let constants_folded = folder.fold_constants(&mut func).expect("folding failed");

    // Should have folded the logical operation
    assert!(constants_folded > 0, "should fold at least one constant");

    // After folding, we should have constant values
    let const_count = count_const_instrs(&func);

    assert!(const_count >= 2, "should have constant values");
}

#[test]
fn test_string_concat() {
    // Create: "hello" + " world"
    let instrs = vec![
        MIRInstr::Const(MIRConst::Str("hello".to_string())),
        MIRInstr::Const(MIRConst::Str(" world".to_string())),
        MIRInstr::BinOp { op: BinOpKind::Add, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Str },
    ];

    let mut func = create_function_with_instrs("test_string_concat", instrs);

    let mut folder = ConstantFolder::new();
    let constants_folded = folder.fold_constants(&mut func).expect("folding failed");

    // Should have folded the string concatenation
    assert!(constants_folded > 0, "should fold at least one constant");

    // After folding, we should have constant values
    let const_count = count_const_instrs(&func);

    assert!(const_count >= 2, "should have constant values");
}

#[test]
fn test_constant_propagation() {
    // Create: 10 * 5  -> should become 50
    //         50 / 2  -> should become 25
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(10)),
        MIRInstr::Const(MIRConst::Int(5)),
        MIRInstr::BinOp { op: BinOpKind::Mul, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Int },
        MIRInstr::Const(MIRConst::Int(2)),
        MIRInstr::BinOp {
            op: BinOpKind::Div,
            lhs: ValueID(2), // Result of previous BinOp
            rhs: ValueID(3),
            ty: MIRType::Int,
        },
    ];

    let mut func = create_function_with_instrs("test_propagation", instrs);

    let mut folder = ConstantFolder::new();
    let constants_folded = folder.fold_constants(&mut func).expect("folding failed");

    // Should fold multiple constants through propagation
    assert!(constants_folded > 0, "should fold constants");

    // After folding, we should have more constant values
    let const_count = count_const_instrs(&func);

    assert!(const_count >= 3, "should have propagated constant values");
}
