//! Integration tests for the MIR optimizer.
//!
//! These tests verify that optimization passes work correctly end-to-end.

use typhon_mir::block::BasicBlock;
use typhon_mir::function::{MIRFunction, MIRLocal, MIRParam};
use typhon_mir::instr::{
    BasicBlockID,
    BinOpKind,
    LocalID,
    MIRConst,
    MIRInstr,
    Terminator,
    ValueID,
};
use typhon_mir::types::MIRType;
use typhon_mir_optimizer::analysis::{ControlFlowGraph, DominatorTree};
use typhon_mir_optimizer::pass_manager::PassManager;
use typhon_mir_optimizer::validation::{CFGValidator, SSAValidator};
use typhon_source::types::Span;

/// Helper to create a simple function for testing.
fn create_test_function() -> MIRFunction {
    MIRFunction {
        name: "test_func".to_string(),
        params: vec![MIRParam { name: "x".to_string(), local: LocalID(0), ty: MIRType::Int }],
        return_type: MIRType::Int,
        locals: vec![
            MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: false },
            MIRLocal { name: Some("y".to_string()), ty: MIRType::Int, mutable: false },
            MIRLocal { name: Some("z".to_string()), ty: MIRType::Int, mutable: false },
        ],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![
                MIRInstr::Const(MIRConst::Int(10)),
                MIRInstr::Store { local: LocalID(1), value: ValueID(0) },
                MIRInstr::Const(MIRConst::Int(20)),
                MIRInstr::Store { local: LocalID(2), value: ValueID(1) },
                MIRInstr::Load { local: LocalID(1), ty: MIRType::Int },
                MIRInstr::Load { local: LocalID(2), ty: MIRType::Int },
                MIRInstr::BinOp {
                    op: BinOpKind::Add,
                    lhs: ValueID(2),
                    rhs: ValueID(3),
                    ty: MIRType::Int,
                },
            ],
            terminator: Terminator::Return(Some(ValueID(4))),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    }
}

#[test]
fn test_cfg_validation() {
    let func = create_test_function();
    assert!(CFGValidator::validate(&func).is_ok());
}

#[test]
fn test_ssa_validation() {
    let func = create_test_function();
    assert!(SSAValidator::validate(&func).is_ok());
}

#[test]
fn test_cfg_construction() {
    let func = create_test_function();
    let cfg = ControlFlowGraph::compute(&func);

    assert_eq!(cfg.predecessors(BasicBlockID(0)).len(), 0);
    assert_eq!(cfg.successors(BasicBlockID(0)).len(), 0);
}

#[test]
fn test_dominance_tree() {
    let func = create_test_function();
    let cfg = ControlFlowGraph::compute(&func);
    let dom_tree = DominatorTree::compute(&func);

    // Entry block dominates itself
    assert!(dom_tree.dominates(BasicBlockID(0), BasicBlockID(0)));
}

#[test]
fn test_constant_folding() {
    let mut func = MIRFunction {
        name: "constant_fold_test".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![MIRLocal {
            name: Some("result".to_string()),
            ty: MIRType::Int,
            mutable: false,
        }],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![
                MIRInstr::Const(MIRConst::Int(2)),
                MIRInstr::Const(MIRConst::Int(3)),
                MIRInstr::BinOp {
                    op: BinOpKind::Mul,
                    lhs: ValueID(0),
                    rhs: ValueID(1),
                    ty: MIRType::Int,
                },
                MIRInstr::Store { local: LocalID(0), value: ValueID(2) },
            ],
            terminator: Terminator::Return(Some(ValueID(2))),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    };

    let pass = ConstantFolding;
    let result = pass.run(&mut func);

    assert!(result.is_ok());
    // After constant folding, the multiplication should be replaced with a constant
}

#[test]
fn test_dead_code_elimination() {
    let mut func = MIRFunction {
        name: "dce_test".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![
            MIRLocal { name: Some("used".to_string()), ty: MIRType::Int, mutable: false },
            MIRLocal { name: Some("unused".to_string()), ty: MIRType::Int, mutable: false },
        ],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![
                MIRInstr::Const(MIRConst::Int(42)),
                MIRInstr::Store { local: LocalID(0), value: ValueID(0) },
                MIRInstr::Const(MIRConst::Int(99)), // Dead code
                MIRInstr::Store { local: LocalID(1), value: ValueID(1) }, // Dead code
                MIRInstr::Load { local: LocalID(0), ty: MIRType::Int },
            ],
            terminator: Terminator::Return(Some(ValueID(2))),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    };

    let pass = DeadCodeElimination;
    let result = pass.run(&mut func);

    assert!(result.is_ok());
    // After DCE, unused instructions should be removed
}

#[test]
fn test_pass_manager() {
    let mut func = create_test_function();
    let mut manager = PassManager::default();

    // Add passes to the manager
    manager.add_pass(Box::new(ConstantFolding));
    manager.add_pass(Box::new(DeadCodeElimination));

    // Run all passes
    let result = manager.run(&mut func);
    assert!(result.is_ok());
}

#[test]
fn test_multi_block_cfg() {
    let func = MIRFunction {
        name: "multi_block".to_string(),
        params: vec![MIRParam { name: "cond".to_string(), local: LocalID(0), ty: MIRType::Bool }],
        return_type: MIRType::Int,
        locals: vec![
            MIRLocal { name: Some("cond".to_string()), ty: MIRType::Bool, mutable: false },
            MIRLocal { name: Some("result".to_string()), ty: MIRType::Int, mutable: false },
        ],
        blocks: vec![
            BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![MIRInstr::Load { local: LocalID(0), ty: MIRType::Bool }],
                terminator: Terminator::CondBranch {
                    condition: ValueID(0),
                    then_block: BasicBlockID(1),
                    else_block: BasicBlockID(2),
                },
                landing_pad: None,
                predecessors: vec![],
                successors: vec![BasicBlockID(1), BasicBlockID(2)],
            },
            BasicBlock {
                id: BasicBlockID(1),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(1)),
                    MIRInstr::Store { local: LocalID(1), value: ValueID(1) },
                ],
                terminator: Terminator::Branch(BasicBlockID(3)),
                landing_pad: None,
                predecessors: vec![BasicBlockID(0)],
                successors: vec![BasicBlockID(3)],
            },
            BasicBlock {
                id: BasicBlockID(2),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(0)),
                    MIRInstr::Store { local: LocalID(1), value: ValueID(2) },
                ],
                terminator: Terminator::Branch(BasicBlockID(3)),
                landing_pad: None,
                predecessors: vec![BasicBlockID(0)],
                successors: vec![BasicBlockID(3)],
            },
            BasicBlock {
                id: BasicBlockID(3),
                instrs: vec![MIRInstr::Load { local: LocalID(1), ty: MIRType::Int }],
                terminator: Terminator::Return(Some(ValueID(3))),
                landing_pad: None,
                predecessors: vec![BasicBlockID(1), BasicBlockID(2)],
                successors: vec![],
            },
        ],
        captures: vec![],
        span: Span::default(),
    };

    // Test CFG construction
    let cfg = ControlFlowGraph::compute(&func);
    assert_eq!(cfg.predecessors(BasicBlockID(3)).len(), 2);
    assert_eq!(cfg.successors(BasicBlockID(0)).len(), 2);

    // Test dominance
    let dom_tree = DominatorTree::compute(&func);
    assert!(dom_tree.dominates(BasicBlockID(0), BasicBlockID(1)));
    assert!(dom_tree.dominates(BasicBlockID(0), BasicBlockID(2)));
    assert!(dom_tree.dominates(BasicBlockID(0), BasicBlockID(3)));

    // Test validation
    assert!(CFGValidator::validate(&func).is_ok());
}
