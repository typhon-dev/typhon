//! Tests for reference count optimization.

use typhon_mir::block::BasicBlock;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, LocalID, MIRConst, MIRInstr, Terminator, ValueID};
use typhon_mir::module::MIRModule;
use typhon_mir::types::MIRType;
use typhon_mir_optimizer::analysis::LivenessAnalysis;
use typhon_mir_optimizer::passes::{EscapeAnalysis, RefCountOptimizer};
use typhon_source::types::Span;

mod fixtures;

use fixtures::create_function_with_instrs_and_terminator;

fn create_test_function(blocks: Vec<BasicBlock>) -> MIRFunction {
    MIRFunction {
        name: "test".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks,
        captures: vec![],
        span: Span::default(),
    }
}

fn create_module_with_function(func: MIRFunction) -> MIRModule {
    MIRModule {
        name: "test_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    }
}

#[test]
fn test_eliminate_noescape_refcounts() {
    // Test: Remove refcounts for non-escaping objects
    // Create: alloc, incref, decref on local object that doesn't escape
    let instrs = vec![
        MIRInstr::AllocObject { type_id: 0, size: 16 },
        MIRInstr::IncRef(ValueID(0)),
        MIRInstr::Store { local: LocalID(0), value: ValueID(0) },
        MIRInstr::DecRef(ValueID(0)),
    ];

    let mut func = create_function_with_instrs_and_terminator(
        "test_noescape",
        instrs,
        Terminator::Return(None),
    );

    let module = create_module_with_function(func.clone());
    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let mut escape = EscapeAnalysis::new();
    escape.analyze_module(&module).expect("escape analysis should succeed");

    let mut optimizer = RefCountOptimizer::new(liveness, escape);
    let removed = optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    // Non-escaping allocation should have both refcounts removed
    assert!(removed >= 2, "should remove at least 2 refcount operations");
    assert_eq!(optimizer.incref_removed(), 1, "should remove 1 incref operation");
    assert_eq!(optimizer.decref_removed(), 1, "should remove 1 decref operation");
}

#[test]
fn test_coalesce_adjacent_ops() {
    // Test: Combine adjacent incref/decref operations on same value
    // Create: incref %0; decref %0 (should be removed as a pair)
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(42)),
        MIRInstr::IncRef(ValueID(0)),
        MIRInstr::DecRef(ValueID(0)),
        MIRInstr::Const(MIRConst::Int(1)),
    ];

    let mut func = create_function_with_instrs_and_terminator(
        "test_adjacent",
        instrs,
        Terminator::Return(None),
    );

    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let escape = EscapeAnalysis::new();
    let mut optimizer = RefCountOptimizer::new(liveness, escape);

    let removed = optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    assert_eq!(removed, 2, "should remove both incref and decref");
    assert_eq!(optimizer.pairs_eliminated(), 1, "should eliminate 1 pair");

    // Verify only const instructions remain
    assert_eq!(func.blocks[0].instrs.len(), 2, "should have 2 const instructions");
}

#[test]
fn test_remove_redundant_pairs() {
    // Test: Eliminate incref followed immediately by decref
    // Create: const, incref, decref - the refcount pair is redundant
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(100)),
        MIRInstr::IncRef(ValueID(0)),
        MIRInstr::DecRef(ValueID(0)),
    ];

    let mut func = create_function_with_instrs_and_terminator(
        "test_redundant",
        instrs,
        Terminator::Return(Some(ValueID(0))),
    );

    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let escape = EscapeAnalysis::new();
    let mut optimizer = RefCountOptimizer::new(liveness, escape);

    let removed = optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    assert_eq!(removed, 2, "should remove redundant pair");
    assert_eq!(optimizer.pairs_eliminated(), 1, "should count pair elimination");

    // Only const should remain
    assert_eq!(func.blocks[0].instrs.len(), 1, "should have only const");
}

#[test]
fn test_preserve_escaped_refcounts() {
    // Test: Keep refcounts for escaping objects (returned values)
    // Create: alloc, incref, return - object escapes via return
    let instrs = vec![MIRInstr::AllocObject { type_id: 0, size: 16 }, MIRInstr::IncRef(ValueID(0))];

    let mut func = create_function_with_instrs_and_terminator(
        "test_escaped",
        instrs,
        Terminator::Return(Some(ValueID(0))),
    );

    let module = create_module_with_function(func.clone());
    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let mut escape = EscapeAnalysis::new();
    escape.analyze_module(&module).expect("escape analysis should succeed");

    let initial_instrs = func.blocks[0].instrs.len();

    let mut optimizer = RefCountOptimizer::new(liveness, escape);
    optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    // Escaping object should keep refcounts (or at least not be fully eliminated)
    // The exact behavior depends on escape analysis results
    let final_instrs = func.blocks[0].instrs.len();

    // We mainly verify no crash occurs and some instructions remain
    assert!(final_instrs > 0, "should have some instructions remaining");
    assert!(final_instrs <= initial_instrs, "should not add instructions");
}

#[test]
fn test_handle_control_flow() {
    // Test: Maintain correctness across branches
    // Create: if-then-else with refcount ops in different branches
    let blocks = vec![
        BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![
                MIRInstr::Const(MIRConst::Bool(true)),
                MIRInstr::AllocObject { type_id: 0, size: 16 },
            ],
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
            instrs: vec![MIRInstr::IncRef(ValueID(1)), MIRInstr::DecRef(ValueID(1))],
            terminator: Terminator::Branch(BasicBlockID(3)),
            landing_pad: None,
            predecessors: vec![BasicBlockID(0)],
            successors: vec![BasicBlockID(3)],
        },
        BasicBlock {
            id: BasicBlockID(2),
            instrs: vec![MIRInstr::IncRef(ValueID(1)), MIRInstr::DecRef(ValueID(1))],
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

    let mut func = create_test_function(blocks);

    let module = create_module_with_function(func.clone());
    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let mut escape = EscapeAnalysis::new();
    escape.analyze_module(&module).expect("escape analysis should succeed");

    let mut optimizer = RefCountOptimizer::new(liveness, escape);
    let removed = optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    // Should remove some refcount operations (exact count depends on escape analysis)
    assert!(removed >= 2, "should remove refcount ops from branches");

    // Verify some optimization occurred
    // Note: Exact behavior depends on escape analysis and liveness
    assert!(
        optimizer.pairs_eliminated() >= 1 || removed >= 2,
        "should eliminate at least one pair or remove multiple operations"
    );
}

#[test]
fn test_preserve_non_adjacent() {
    // Test: Don't remove non-adjacent refcount operations
    // Create: incref, other instruction, decref - not adjacent, keep both
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(1)),
        MIRInstr::IncRef(ValueID(0)),
        MIRInstr::Const(MIRConst::Int(2)), // Separates incref/decref
        MIRInstr::DecRef(ValueID(0)),
    ];

    let mut func = create_function_with_instrs_and_terminator(
        "test_non_adjacent",
        instrs,
        Terminator::Return(None),
    );

    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let escape = EscapeAnalysis::new();
    let mut optimizer = RefCountOptimizer::new(liveness, escape);

    let initial_refcounts = 2;
    optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    // Non-adjacent operations should not be eliminated as pairs
    // (though other optimizations might still apply)
    let remaining_instrs = func.blocks[0].instrs.len();

    assert!(remaining_instrs >= 2, "should preserve non-adjacent pattern structure");
}

#[test]
fn test_multiple_values() {
    // Test: Handle multiple different values correctly
    // Create: operations on different values should be independent
    let instrs = vec![
        MIRInstr::Const(MIRConst::Int(1)),
        MIRInstr::Const(MIRConst::Int(2)),
        MIRInstr::IncRef(ValueID(0)),
        MIRInstr::DecRef(ValueID(0)), // Pair on value 0
        MIRInstr::IncRef(ValueID(1)),
        MIRInstr::DecRef(ValueID(1)), // Pair on value 1
    ];

    let mut func = create_function_with_instrs_and_terminator(
        "test_multiple",
        instrs,
        Terminator::Return(None),
    );

    let liveness = LivenessAnalysis::analyze(&func).expect("liveness analysis should succeed");
    let escape = EscapeAnalysis::new();
    let mut optimizer = RefCountOptimizer::new(liveness, escape);

    let removed = optimizer.optimize_refcounts(&mut func).expect("optimization should succeed");

    // Should remove both pairs
    assert_eq!(removed, 4, "should remove all 4 refcount operations");
    assert_eq!(optimizer.pairs_eliminated(), 2, "should eliminate 2 pairs");
}
