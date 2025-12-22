//! Control Flow Graph (CFG) analysis utilities.
//!
//! This module provides utilities for analyzing the control flow graph
//! of MIR functions, including predecessor/successor relationships,
//! reverse postorder traversal, and other graph properties.

use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::BasicBlockID;

/// Control Flow Graph representation.
///
/// Provides efficient access to CFG properties like predecessors,
/// successors, and various traversal orders.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Entry block ID
    pub entry: BasicBlockID,
    /// Map from block ID to its predecessors
    pub predecessors: FxHashMap<BasicBlockID, Vec<BasicBlockID>>,
    /// Map from block ID to its successors
    pub successors: FxHashMap<BasicBlockID, Vec<BasicBlockID>>,
    /// Blocks in reverse postorder (RPO)
    pub reverse_postorder: Vec<BasicBlockID>,
}

impl ControlFlowGraph {
    /// Check if a block has multiple successors (is a branch point).
    #[must_use]
    pub fn is_branch_point(&self, block: BasicBlockID) -> bool { self.successors(block).len() >= 2 }

    /// Check if a block has multiple predecessors (is a join point).
    #[must_use]
    pub fn is_join_point(&self, block: BasicBlockID) -> bool { self.predecessors(block).len() >= 2 }

    /// Get blocks in postorder.
    ///
    /// Postorder is useful for backward dataflow analysis.
    #[must_use]
    pub fn postorder(&self) -> Vec<BasicBlockID> {
        let mut po = self.reverse_postorder.clone();
        po.reverse();
        po
    }

    /// Get predecessors of a block.
    #[must_use]
    pub fn predecessors(&self, block: BasicBlockID) -> &[BasicBlockID] {
        self.predecessors.get(&block).map_or(&[], |v| v.as_slice())
    }

    /// Get blocks in reverse postorder.
    ///
    /// RPO is useful for forward dataflow analysis as it processes
    /// a block after most of its predecessors.
    #[must_use]
    pub fn reverse_postorder(&self) -> &[BasicBlockID] { &self.reverse_postorder }

    /// Get successors of a block.
    #[must_use]
    pub fn successors(&self, block: BasicBlockID) -> &[BasicBlockID] {
        self.successors.get(&block).map_or(&[], |v| v.as_slice())
    }

    /// Compute CFG properties for a function.
    #[must_use]
    pub fn compute(func: &MIRFunction) -> Self {
        if func.blocks.is_empty() {
            return Self {
                entry: BasicBlockID(0),
                predecessors: FxHashMap::default(),
                successors: FxHashMap::default(),
                reverse_postorder: Vec::new(),
            };
        }

        let entry = func.blocks[0].id;

        // Build predecessor and successor maps from the function's blocks
        let mut predecessors = FxHashMap::default();
        let mut successors = FxHashMap::default();

        for block in &func.blocks {
            // Copy successor list from block
            successors.insert(block.id, block.successors.clone());

            // Copy predecessor list from block
            predecessors.insert(block.id, block.predecessors.clone());
        }

        // Compute reverse postorder
        let reverse_postorder = compute_reverse_postorder(entry, &successors);

        Self { entry, predecessors, successors, reverse_postorder }
    }
}

/// Compute reverse postorder traversal of the CFG.
///
/// Reverse postorder visits blocks in an order such that each block
/// is visited after most of its predecessors, making it ideal for
/// forward dataflow analysis.
fn compute_reverse_postorder(
    entry: BasicBlockID,
    successors: &FxHashMap<BasicBlockID, Vec<BasicBlockID>>,
) -> Vec<BasicBlockID> {
    /// Recursive depth-first traversal for postorder computation.
    fn dfs(
        block: BasicBlockID,
        successors: &FxHashMap<BasicBlockID, Vec<BasicBlockID>>,
        visited: &mut IndexSet<BasicBlockID>,
        postorder: &mut Vec<BasicBlockID>,
    ) {
        if !visited.insert(block) {
            return;
        }

        // Visit all successors first
        if let Some(succs) = successors.get(&block) {
            for &succ in succs {
                dfs(succ, successors, visited, postorder);
            }
        }

        // Add this block after visiting successors (postorder)
        postorder.push(block);
    }

    let mut visited = IndexSet::new();
    let mut postorder = Vec::new();

    dfs(entry, successors, &mut visited, &mut postorder);

    // Reverse to get reverse postorder
    postorder.reverse();

    postorder
}

#[cfg(test)]
mod tests {
    use typhon_mir::block::BasicBlock;
    use typhon_mir::function::MIRFunction;
    use typhon_mir::instr::{BasicBlockID, Terminator, ValueID};
    use typhon_mir::types::MIRType;
    use typhon_source::types::Span;

    use super::*;

    #[test]
    fn test_cfg_branch() {
        // Branch CFG: 0 -> {1, 2}
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Void,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    id: BasicBlockID(0),
                    instrs: vec![],
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
                    terminator: Terminator::Return(None),
                    landing_pad: None,
                    predecessors: vec![BasicBlockID(0)],
                    successors: vec![],
                },
                BasicBlock {
                    id: BasicBlockID(2),
                    instrs: vec![],
                    terminator: Terminator::Return(None),
                    landing_pad: None,
                    predecessors: vec![BasicBlockID(0)],
                    successors: vec![],
                },
            ],
            captures: vec![],
            span: Span::default(),
        };

        let cfg = ControlFlowGraph::compute(&func);

        assert!(cfg.is_branch_point(BasicBlockID(0)));
        assert!(!cfg.is_join_point(BasicBlockID(1)));
        assert!(!cfg.is_join_point(BasicBlockID(2)));
    }

    #[test]
    fn test_cfg_join() {
        // Join CFG: {0, 1} -> 2
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Void,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    id: BasicBlockID(0),
                    instrs: vec![],
                    terminator: Terminator::Branch(BasicBlockID(2)),
                    landing_pad: None,
                    predecessors: vec![],
                    successors: vec![BasicBlockID(2)],
                },
                BasicBlock {
                    id: BasicBlockID(1),
                    instrs: vec![],
                    terminator: Terminator::Branch(BasicBlockID(2)),
                    landing_pad: None,
                    predecessors: vec![],
                    successors: vec![BasicBlockID(2)],
                },
                BasicBlock {
                    id: BasicBlockID(2),
                    instrs: vec![],
                    terminator: Terminator::Return(None),
                    landing_pad: None,
                    predecessors: vec![BasicBlockID(0), BasicBlockID(1)],
                    successors: vec![],
                },
            ],
            captures: vec![],
            span: Span::default(),
        };

        let cfg = ControlFlowGraph::compute(&func);

        assert!(cfg.is_join_point(BasicBlockID(2)));
        assert_eq!(cfg.predecessors(BasicBlockID(2)).len(), 2);
    }

    #[test]
    fn test_cfg_linear() {
        // Linear CFG: 0 -> 1 -> 2
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Void,
            locals: vec![],
            blocks: vec![
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
                    terminator: Terminator::Branch(BasicBlockID(2)),
                    landing_pad: None,
                    predecessors: vec![BasicBlockID(0)],
                    successors: vec![BasicBlockID(2)],
                },
                BasicBlock {
                    id: BasicBlockID(2),
                    instrs: vec![],
                    terminator: Terminator::Return(None),
                    landing_pad: None,
                    predecessors: vec![BasicBlockID(1)],
                    successors: vec![],
                },
            ],
            captures: vec![],
            span: Span::default(),
        };

        let cfg = ControlFlowGraph::compute(&func);

        assert_eq!(cfg.entry, BasicBlockID(0));
        assert_eq!(cfg.predecessors(BasicBlockID(0)), &[]);
        assert_eq!(cfg.predecessors(BasicBlockID(1)), &[BasicBlockID(0)]);
        assert_eq!(cfg.predecessors(BasicBlockID(2)), &[BasicBlockID(1)]);
        assert_eq!(cfg.successors(BasicBlockID(0)), &[BasicBlockID(1)]);
        assert_eq!(cfg.successors(BasicBlockID(1)), &[BasicBlockID(2)]);
        assert_eq!(cfg.successors(BasicBlockID(2)), &[]);
    }
}
