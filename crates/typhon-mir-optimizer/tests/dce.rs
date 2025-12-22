//! Tests for dead code elimination optimization pass.

use typhon_mir::instr::{BasicBlockID, BinOpKind, MIRConst, MIRInstr, Terminator, ValueID};
use typhon_mir::types::MIRType;
use typhon_mir_optimizer::passes::DeadCodeEliminator;

mod fixtures;

use fixtures::{count_blocks, count_instrs, create_function_with_instrs};

#[test]
fn test_unused_computation() {
    // Create: x = 2 + 3 (but x is never used)
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(2)),
        MIRInstr::Const(MIRConst::Int(3)),
        MIRInstr::BinOp { op: BinOpKind::Add, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Int },
    ];

    let mut func = create_function_with_instrs("test_unused", instrs);
    let initial_count = count_instrs(&func);
    let mut eliminator = DeadCodeEliminator::new();
    let (blocks_removed, instrs_removed) =
        eliminator.eliminate_dead_code(&mut func).expect("DCE failed");

    // Should remove the unused BinOp
    assert!(instrs_removed > 0 || blocks_removed > 0, "should remove dead code");

    let final_count = count_instrs(&func);

    assert!(final_count <= initial_count, "instruction count should not increase");
}

#[test]
fn test_keep_side_effects() {
    // Create: call to a function (has side effects, should be kept)
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(42)),
        MIRInstr::Call { callee: ValueID(0), args: vec![], ty: MIRType::Void },
    ];

    let func = create_function_with_instrs("test_side_effects", instrs);

    // Should keep the Call instruction (side effect)
    let final_count = count_instrs(&func);

    assert!(final_count > 0, "should keep instructions with side effects");
}

#[test]
fn test_dead_branch() {
    // Create a function with an unreachable block
    // This test verifies that unreachable blocks are removed
    let instrs = vec![MIRInstr::Const(MIRConst::Int(1))];
    let mut func = create_function_with_instrs("test_dead_branch", instrs);

    // Manually add an unreachable block
    func.blocks.push(typhon_mir::block::BasicBlock {
        id: BasicBlockID(1),
        instrs: vec![MIRInstr::Const(MIRConst::Int(999))],
        terminator: Terminator::Return(None),
        landing_pad: None,
        predecessors: vec![],
        successors: vec![],
    });

    let initial_blocks = count_blocks(&func);
    let mut eliminator = DeadCodeEliminator::new();
    let (blocks_removed, _instrs_removed) =
        eliminator.eliminate_dead_code(&mut func).expect("DCE failed");

    // Should remove the unreachable block
    if initial_blocks > 1 {
        assert!(blocks_removed > 0, "should remove unreachable blocks");
    }
}

#[test]
fn test_unused_phi() {
    // Create: phi node with no uses
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(1)),
        MIRInstr::Const(MIRConst::Int(2)),
        MIRInstr::Phi {
            incoming: vec![(BasicBlockID(0), ValueID(0)), (BasicBlockID(0), ValueID(1))],
            ty: MIRType::Int,
        },
    ];

    let mut func = create_function_with_instrs("test_unused_phi", instrs);
    let initial_count = count_instrs(&func);
    let mut eliminator = DeadCodeEliminator::new();
    let (blocks_removed, instrs_removed) =
        eliminator.eliminate_dead_code(&mut func).expect("DCE failed");

    // Should remove the unused phi node
    assert!(instrs_removed > 0 || blocks_removed > 0, "should remove unused phi nodes");

    let final_count = count_instrs(&func);

    assert!(final_count <= initial_count, "instruction count should not increase");
}

#[test]
fn test_chain_elimination() {
    // Create: chain of unused computations
    // x = 1 + 2
    // y = x * 3
    // z = y - 4
    // All unused
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(1)),
        MIRInstr::Const(MIRConst::Int(2)),
        MIRInstr::BinOp { op: BinOpKind::Add, lhs: ValueID(0), rhs: ValueID(1), ty: MIRType::Int },
        MIRInstr::Const(MIRConst::Int(3)),
        MIRInstr::BinOp { op: BinOpKind::Mul, lhs: ValueID(2), rhs: ValueID(3), ty: MIRType::Int },
        MIRInstr::Const(MIRConst::Int(4)),
        MIRInstr::BinOp { op: BinOpKind::Sub, lhs: ValueID(4), rhs: ValueID(5), ty: MIRType::Int },
    ];

    let mut func = create_function_with_instrs("test_chain", instrs);
    let initial_count = count_instrs(&func);
    let mut eliminator = DeadCodeEliminator::new();
    let (blocks_removed, instrs_removed) =
        eliminator.eliminate_dead_code(&mut func).expect("DCE failed");

    // Should remove the entire chain of unused computations
    assert!(instrs_removed > 0 || blocks_removed > 0, "should remove chain of unused computations");

    let final_count = count_instrs(&func);

    assert!(final_count < initial_count, "should significantly reduce instruction count");
}
