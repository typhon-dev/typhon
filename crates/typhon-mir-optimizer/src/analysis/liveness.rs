//! Liveness analysis for MIR optimization.
//!
//! This module implements backward dataflow analysis to determine which values are live
//! (may be used in the future) at each program point. This analysis is essential for
//! optimizations like dead code elimination and register allocation.
//!
//! ## Integration with Dead Code Elimination
//!
//! The DCE pass can be enhanced by using liveness analysis to more precisely identify dead code:
//!
//! 1. **Current DCE approach** (simple): Marks values as live if they're used by side-effecting
//!    instructions or terminators. This is conservative and may miss some dead code.
//!
//! 2. **Enhanced DCE with liveness** (more precise):
//!    - Run [`LivenessAnalysis::analyze()`] to compute live-in/live-out sets for all blocks
//!    - For each instruction producing a value, check if that value appears in the live-out
//!      set of any successor block
//!    - Remove instructions whose produced values are not live (unless they have side effects)
//!    - This catches more dead code, especially in loops and complex control flow
//!
//! Example integration:
//!
//! ```rust,ignore
//! let liveness = LivenessAnalysis::analyze(func)?;
//! for block in &mut func.blocks {
//!     block.instrs.retain(|instr| {
//!         let value_id = get_produced_value(instr);
//!         // Keep if has side effects OR if value is live anywhere
//!         has_side_effects(instr) ||
//!         func.blocks.iter().any(|b| liveness.is_live_at(value_id, b.id))
//!     });
//! }
//! ```

use std::collections::VecDeque;

use indexmap::{IndexMap, IndexSet};
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, MIRInstr, Terminator, ValueID};

use crate::error::{OptimizerError, OptimizerResult};

/// Liveness analysis result.
///
/// Uses backward dataflow analysis to compute which values are live at each program point.
/// A value is live at a point if it may be used along some path to the function exit.
#[derive(Debug, Clone)]
pub struct LivenessAnalysis {
    /// Values that are live at the entry of each block.
    live_in: IndexMap<BasicBlockID, IndexSet<ValueID>>,
    /// Values that are live at the exit of each block.
    live_out: IndexMap<BasicBlockID, IndexSet<ValueID>>,
}

impl LivenessAnalysis {
    /// Get all values that are live at the exit of a block.
    #[must_use]
    pub fn get_live_out(&self, block: BasicBlockID) -> Option<&IndexSet<ValueID>> {
        self.live_out.get(&block)
    }

    /// Get all values that are live at the entry of a block.
    #[must_use]
    pub fn get_live_values(&self, block: BasicBlockID) -> Option<&IndexSet<ValueID>> {
        self.live_in.get(&block)
    }

    /// Check if a value is live at the entry of a block.
    #[must_use]
    pub fn is_live_at(&self, value: ValueID, block: BasicBlockID) -> bool {
        self.live_in.get(&block).is_some_and(|set| set.contains(&value))
    }

    /// Analyze a function to compute liveness information.
    ///
    /// This implements a backward dataflow analysis:
    ///
    /// 1. Compute use and def sets for each block
    /// 2. Iterate until fixed point:
    ///    - `live_out[B]` = union of `live_in[S]` for all successors S
    ///    - `live_in[B] = use[B] ∪ (live_out[B] - def[B])`
    ///
    /// ## Errors
    ///
    /// Returns an error if the function has an invalid CFG structure.
    pub fn analyze(func: &MIRFunction) -> OptimizerResult<Self> {
        if func.blocks.is_empty() {
            return Err(OptimizerError::CFGMalformed("Function has no basic blocks".to_string()));
        }

        let mut analysis = Self { live_in: IndexMap::new(), live_out: IndexMap::new() };

        // Initialize live sets for all blocks
        for block in &func.blocks {
            analysis.live_in.insert(block.id, IndexSet::new());
            analysis.live_out.insert(block.id, IndexSet::new());
        }

        // Compute use and def sets for each block
        let use_sets = Self::compute_use_sets(func);
        let def_sets = Self::compute_def_sets(func);

        // Fixed-point iteration using worklist algorithm
        let mut worklist: VecDeque<BasicBlockID> = func.blocks.iter().map(|b| b.id).collect();
        let mut changed = true;

        while changed {
            changed = false;

            // Process blocks in reverse postorder for faster convergence
            while let Some(block_id) = worklist.pop_front() {
                let block = func
                    .blocks
                    .iter()
                    .find(|b| b.id == block_id)
                    .ok_or(OptimizerError::InvalidBlock(block_id))?;

                // Compute live_out[B] = union of live_in[S] for all successors S
                let mut new_live_out = IndexSet::new();
                for succ_id in &block.successors {
                    if let Some(succ_live_in) = analysis.live_in.get(succ_id) {
                        new_live_out.extend(succ_live_in.iter().copied());
                    }
                }

                // Compute live_in[B] = use[B] ∪ (live_out[B] - def[B])
                let use_set = use_sets.get(&block_id).cloned().unwrap_or_default();
                let def_set = def_sets.get(&block_id).cloned().unwrap_or_default();

                let mut new_live_in = use_set;
                for value in &new_live_out {
                    if !def_set.contains(value) {
                        new_live_in.insert(*value);
                    }
                }

                // Check if anything changed
                let live_in_changed = analysis.live_in.get(&block_id) != Some(&new_live_in);
                let live_out_changed = analysis.live_out.get(&block_id) != Some(&new_live_out);

                if live_in_changed || live_out_changed {
                    changed = true;
                    analysis.live_in.insert(block_id, new_live_in);
                    analysis.live_out.insert(block_id, new_live_out);

                    // Add predecessors to worklist
                    for pred_id in &block.predecessors {
                        if !worklist.contains(pred_id) {
                            worklist.push_back(*pred_id);
                        }
                    }
                }
            }
        }

        Ok(analysis)
    }

    /// Compute def sets for all blocks.
    ///
    /// A value is in def[B] if it is defined (assigned) in block B.
    /// We reconstruct the `ValueID` assignment by traversing instructions sequentially,
    /// matching how the `FunctionBuilder` assigns `ValueID`s.
    fn compute_def_sets(func: &MIRFunction) -> IndexMap<BasicBlockID, IndexSet<ValueID>> {
        let mut def_sets = IndexMap::new();
        let mut next_value_id = 0u32;

        for block in &func.blocks {
            let mut def_set = IndexSet::new();

            // Process instructions - each value-producing instruction gets the next ValueID
            for instr in &block.instrs {
                let produces_value = matches!(
                    instr,
                    MIRInstr::AllocObject { .. }
                        | MIRInstr::BinOp { .. }
                        | MIRInstr::Call { .. }
                        | MIRInstr::Cast { .. }
                        | MIRInstr::Const(_)
                        | MIRInstr::CreateClosure { .. }
                        | MIRInstr::GetAttr { .. }
                        | MIRInstr::GetCapture { .. }
                        | MIRInstr::GetClassAttr { .. }
                        | MIRInstr::GetItem { .. }
                        | MIRInstr::InstanceOf { .. }
                        | MIRInstr::Load { .. }
                        | MIRInstr::LoadGlobal { .. }
                        | MIRInstr::MethodCall { .. }
                        | MIRInstr::Phi { .. }
                        | MIRInstr::UnOp { .. }
                );

                if produces_value {
                    def_set.insert(ValueID(next_value_id));
                    next_value_id += 1;
                }
            }

            def_sets.insert(block.id, def_set);
        }

        def_sets
    }

    /// Compute use sets for all blocks.
    ///
    /// A value is in use[B] if it is used in block B before being defined.
    /// We reconstruct the `ValueID` assignment by traversing instructions sequentially.
    fn compute_use_sets(func: &MIRFunction) -> IndexMap<BasicBlockID, IndexSet<ValueID>> {
        let mut use_sets = IndexMap::new();
        let mut next_value_id = 0u32;

        for block in &func.blocks {
            let mut use_set = IndexSet::new();
            let mut local_defs = IndexSet::new();

            // Process instructions in order
            for instr in &block.instrs {
                // Collect used values (that haven't been defined yet in this block)
                let used_values = Self::get_used_values(instr);
                for used in used_values {
                    if !local_defs.contains(&used) {
                        use_set.insert(used);
                    }
                }

                // Track definitions - each value-producing instruction gets the next ValueID
                let produces_value = matches!(
                    instr,
                    MIRInstr::AllocObject { .. }
                        | MIRInstr::BinOp { .. }
                        | MIRInstr::Call { .. }
                        | MIRInstr::Cast { .. }
                        | MIRInstr::Const(_)
                        | MIRInstr::CreateClosure { .. }
                        | MIRInstr::GetAttr { .. }
                        | MIRInstr::GetCapture { .. }
                        | MIRInstr::GetClassAttr { .. }
                        | MIRInstr::GetItem { .. }
                        | MIRInstr::InstanceOf { .. }
                        | MIRInstr::Load { .. }
                        | MIRInstr::LoadGlobal { .. }
                        | MIRInstr::MethodCall { .. }
                        | MIRInstr::Phi { .. }
                        | MIRInstr::UnOp { .. }
                );

                if produces_value {
                    local_defs.insert(ValueID(next_value_id));
                    next_value_id += 1;
                }
            }

            // Add values used in terminator (but not if already defined in this block)
            let term_used = Self::get_terminator_used_values(&block.terminator);
            for used in term_used {
                if !local_defs.contains(&used) {
                    use_set.insert(used);
                }
            }

            use_sets.insert(block.id, use_set);
        }

        use_sets
    }

    /// Extract all values used by a terminator instruction.
    fn get_terminator_used_values(terminator: &Terminator) -> Vec<ValueID> {
        match terminator {
            Terminator::CondBranch { condition, .. } => vec![*condition],
            Terminator::Invoke { callee, args, .. } => {
                let mut values = vec![*callee];
                values.extend(args.iter().copied());
                values
            }
            Terminator::Raise(value) | Terminator::Return(Some(value)) => vec![*value],
            Terminator::Branch(_) | Terminator::Unreachable | Terminator::Return(None) => vec![],
        }
    }

    /// Extract all values used by an instruction.
    fn get_used_values(instr: &MIRInstr) -> Vec<ValueID> {
        match instr {
            MIRInstr::BinOp { lhs, rhs, .. } => vec![*lhs, *rhs],
            MIRInstr::Call { callee, args, .. } => {
                let mut values = vec![*callee];
                values.extend(args.iter().copied());
                values
            }
            MIRInstr::Cast { value, .. } | MIRInstr::DecRef(value) | MIRInstr::IncRef(value) => {
                vec![*value]
            }
            MIRInstr::CreateClosure { captures, .. } => captures.clone(),
            MIRInstr::GetAttr { object, .. }
            | MIRInstr::GetCapture { closure: object, .. }
            | MIRInstr::InstanceOf { object, .. }
            | MIRInstr::UnOp { operand: object, .. } => vec![*object],
            MIRInstr::GetItem { object, key, .. } => vec![*object, *key],
            MIRInstr::MethodCall { object, args, .. } => {
                let mut values = vec![*object];
                values.extend(args.iter().copied());
                values
            }
            MIRInstr::Phi { incoming, .. } => incoming.iter().map(|(_, v)| *v).collect(),
            MIRInstr::SetAttr { object, value, .. }
            | MIRInstr::SetCapture { closure: object, value, .. } => vec![*object, *value],
            MIRInstr::SetItem { object, key, value, .. } => vec![*object, *key, *value],
            MIRInstr::Store { value, .. } | MIRInstr::StoreGlobal { value, .. } => vec![*value],
            // Instructions that don't use values
            MIRInstr::AllocObject { .. }
            | MIRInstr::Const(_)
            | MIRInstr::GetClassAttr { .. }
            | MIRInstr::Load { .. }
            | MIRInstr::LoadGlobal { .. } => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use typhon_mir::block::BasicBlock;
    use typhon_mir::instr::{BinOpKind, MIRConst};
    use typhon_mir::types::MIRType;
    use typhon_source::types::Span;

    use super::*;

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

    #[test]
    fn test_simple_liveness() {
        // Single block with use-def:
        // %0 = const 1
        // %1 = const 2
        // %2 = add %0, %1
        // return %2
        let block = BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![
                MIRInstr::Const(MIRConst::Int(1)),
                MIRInstr::Const(MIRConst::Int(2)),
                MIRInstr::BinOp {
                    op: BinOpKind::Add,
                    lhs: ValueID(0),
                    rhs: ValueID(1),
                    ty: MIRType::Int,
                },
            ],
            terminator: Terminator::Return(Some(ValueID(2))),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        };

        let func = create_test_function(vec![block]);
        let analysis = LivenessAnalysis::analyze(&func).unwrap();

        // All values are defined and used within the block, so nothing is live at entry
        let live_in = analysis.get_live_values(BasicBlockID(0)).unwrap();

        assert!(live_in.is_empty());

        // No successors, so nothing is live at exit
        let live_out = analysis.get_live_out(BasicBlockID(0)).unwrap();

        assert!(live_out.is_empty());
    }

    #[test]
    fn test_if_else_liveness() {
        // Block 0: %0 = const 1; branch %1 ? block1 : block2
        // Block 1: %1 = const 2; branch block3
        // Block 2: %2 = const 3; branch block3
        // Block 3: %3 = phi [block1: %1], [block2: %2]; return %3
        let blocks = vec![
            BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![MIRInstr::Const(MIRConst::Int(1))],
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
                instrs: vec![MIRInstr::Const(MIRConst::Int(2))],
                terminator: Terminator::Branch(BasicBlockID(3)),
                landing_pad: None,
                predecessors: vec![BasicBlockID(0)],
                successors: vec![BasicBlockID(3)],
            },
            BasicBlock {
                id: BasicBlockID(2),
                instrs: vec![MIRInstr::Const(MIRConst::Int(3))],
                terminator: Terminator::Branch(BasicBlockID(3)),
                landing_pad: None,
                predecessors: vec![BasicBlockID(0)],
                successors: vec![BasicBlockID(3)],
            },
            BasicBlock {
                id: BasicBlockID(3),
                instrs: vec![MIRInstr::Phi {
                    incoming: vec![
                        (BasicBlockID(1), ValueID(10001)),
                        (BasicBlockID(2), ValueID(20002)),
                    ],
                    ty: MIRType::Int,
                }],
                terminator: Terminator::Return(Some(ValueID(30003))),
                landing_pad: None,
                predecessors: vec![BasicBlockID(1), BasicBlockID(2)],
                successors: vec![],
            },
        ];

        let func = create_test_function(blocks);
        let analysis = LivenessAnalysis::analyze(&func).unwrap();

        // Values from block 1 and 2 should be live at their exits (used in phi)
        let live_out_1 = analysis.get_live_out(BasicBlockID(1)).unwrap();
        assert!(live_out_1.contains(&ValueID(10001)));

        let live_out_2 = analysis.get_live_out(BasicBlockID(2)).unwrap();
        assert!(live_out_2.contains(&ValueID(20002)));
    }

    #[test]
    fn test_loop_liveness() {
        // Block 0: %0 = const 0; branch block1
        // Block 1: %1 = phi [block0: %0], [block1: %2]; %2 = add %1, 1; branch block1
        let blocks = vec![
            BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![MIRInstr::Const(MIRConst::Int(0))],
                terminator: Terminator::Branch(BasicBlockID(1)),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![BasicBlockID(1)],
            },
            BasicBlock {
                id: BasicBlockID(1),
                instrs: vec![
                    MIRInstr::Phi {
                        incoming: vec![
                            (BasicBlockID(0), ValueID(0)),
                            (BasicBlockID(1), ValueID(10001)),
                        ],
                        ty: MIRType::Int,
                    },
                    MIRInstr::BinOp {
                        op: BinOpKind::Add,
                        lhs: ValueID(10000),
                        rhs: ValueID(1),
                        ty: MIRType::Int,
                    },
                ],
                terminator: Terminator::Branch(BasicBlockID(1)),
                landing_pad: None,
                predecessors: vec![BasicBlockID(0), BasicBlockID(1)],
                successors: vec![BasicBlockID(1)],
            },
        ];

        let func = create_test_function(blocks);
        let analysis = LivenessAnalysis::analyze(&func).unwrap();

        // The phi result should be live at block 1 entry
        assert!(analysis.is_live_at(ValueID(10000), BasicBlockID(1)));
    }

    #[test]
    fn test_phi_liveness() {
        // Block 0: %0 = const 1; %1 = const 2; branch %0 ? block1 : block2
        // Block 1: branch block3
        // Block 2: branch block3
        // Block 3: %2 = phi [block1: %0], [block2: %1]; return %2
        let blocks = vec![
            BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![MIRInstr::Const(MIRConst::Int(1)), MIRInstr::Const(MIRConst::Int(2))],
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
                instrs: vec![MIRInstr::Phi {
                    incoming: vec![(BasicBlockID(1), ValueID(0)), (BasicBlockID(2), ValueID(1))],
                    ty: MIRType::Int,
                }],
                terminator: Terminator::Return(Some(ValueID(30000))),
                landing_pad: None,
                predecessors: vec![BasicBlockID(1), BasicBlockID(2)],
                successors: vec![],
            },
        ];

        let func = create_test_function(blocks);
        let analysis = LivenessAnalysis::analyze(&func).unwrap();

        // Both phi inputs should be live at their respective block exits
        let live_out_1 = analysis.get_live_out(BasicBlockID(1)).unwrap();
        assert!(live_out_1.contains(&ValueID(0)));

        let live_out_2 = analysis.get_live_out(BasicBlockID(2)).unwrap();
        assert!(live_out_2.contains(&ValueID(1)));
    }

    #[test]
    fn test_dead_value() {
        // Block 0: %0 = const 1; %1 = const 2; return %0
        // %1 is never used, so it should not be live
        let block = BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![MIRInstr::Const(MIRConst::Int(1)), MIRInstr::Const(MIRConst::Int(2))],
            terminator: Terminator::Return(Some(ValueID(0))),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        };

        let func = create_test_function(vec![block]);
        let analysis = LivenessAnalysis::analyze(&func).unwrap();

        // Both values are defined in the block
        // %0 is used, %1 is not, but both are local to the block
        // Neither should be live at entry or exit (no successors)
        let live_in = analysis.get_live_values(BasicBlockID(0)).unwrap();
        assert!(live_in.is_empty());

        let live_out = analysis.get_live_out(BasicBlockID(0)).unwrap();
        assert!(live_out.is_empty());
    }
}
