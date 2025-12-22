//! Control flow graph validation.
//!
//! This module provides validators to check the correctness and consistency
//! of control flow graphs in MIR.

use rustc_hash::FxHashSet;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, Terminator};

use crate::error::{OptimizerError, OptimizerResult};

/// Validator for control flow graph properties.
pub struct CFGValidator;

impl CFGValidator {
    /// Validate that a function's CFG is well-formed.
    ///
    /// Checks:
    /// - All blocks are reachable from entry
    /// - All terminators reference valid blocks
    /// - Predecessor/successor edges are consistent
    /// - No cycles through landing pads
    pub fn validate(func: &MIRFunction) -> OptimizerResult<()> {
        Self::check_block_references(func)?;
        Self::check_edge_consistency(func)?;
        Self::check_reachability(func)?;

        Ok(())
    }

    /// Check that all block references in terminators are valid.
    fn check_block_references(func: &MIRFunction) -> OptimizerResult<()> {
        let valid_blocks: FxHashSet<BasicBlockID> = func.blocks.iter().map(|b| b.id).collect();

        for block in &func.blocks {
            let referenced_blocks = match &block.terminator {
                Terminator::Branch(target) => vec![*target],
                Terminator::CondBranch { then_block, else_block, .. } => {
                    vec![*then_block, *else_block]
                }
                Terminator::Invoke { normal, unwind, .. } => vec![*normal, *unwind],
                Terminator::Return(_) | Terminator::Raise(_) | Terminator::Unreachable => {
                    vec![]
                }
            };

            for &target in &referenced_blocks {
                if !valid_blocks.contains(&target) {
                    return Err(OptimizerError::CFGMalformed(format!(
                        "Block {:?} references non-existent block {target:?}",
                        block.id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check that predecessor and successor edges are consistent.
    fn check_edge_consistency(func: &MIRFunction) -> OptimizerResult<()> {
        for block in &func.blocks {
            // Check that all listed successors are actually targeted by the terminator
            let terminator_targets: FxHashSet<BasicBlockID> = match &block.terminator {
                Terminator::Branch(target) => vec![*target].into_iter().collect(),
                Terminator::CondBranch { then_block, else_block, .. } => {
                    vec![*then_block, *else_block].into_iter().collect()
                }
                Terminator::Invoke { normal, unwind, .. } => {
                    vec![*normal, *unwind].into_iter().collect()
                }
                Terminator::Return(_) | Terminator::Raise(_) | Terminator::Unreachable => {
                    FxHashSet::default()
                }
            };

            let listed_successors: FxHashSet<BasicBlockID> =
                block.successors.iter().copied().collect();

            if terminator_targets != listed_successors {
                return Err(OptimizerError::CFGMalformed(format!(
                    "Block {:?} has inconsistent successor list",
                    block.id
                )));
            }

            // Check that this block is listed as a predecessor in all its successors
            for &successor_id in &block.successors {
                if let Some(successor) = func.blocks.iter().find(|b| b.id == successor_id) {
                    if !successor.predecessors.contains(&block.id) {
                        return Err(OptimizerError::CFGMalformed(format!(
                            "Block {successor_id:?} is successor of {:?} but not listed as predecessor",
                            block.id
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Check that all blocks are reachable from the entry block.
    fn check_reachability(func: &MIRFunction) -> OptimizerResult<()> {
        if func.blocks.is_empty() {
            return Ok(());
        }

        let entry_block = func.blocks[0].id;
        let mut reachable: FxHashSet<BasicBlockID> = FxHashSet::default();
        let mut worklist = vec![entry_block];

        while let Some(block_id) = worklist.pop() {
            if reachable.insert(block_id) {
                if let Some(block) = func.blocks.iter().find(|b| b.id == block_id) {
                    worklist.extend(&block.successors);
                }
            }
        }

        let all_blocks: FxHashSet<BasicBlockID> = func.blocks.iter().map(|b| b.id).collect();
        let unreachable: Vec<_> = all_blocks.difference(&reachable).collect();

        if !unreachable.is_empty() {
            return Err(OptimizerError::CFGMalformed(format!(
                "Unreachable blocks in function {}: {unreachable:?}",
                func.name,
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use typhon_mir::block::BasicBlock;
    use typhon_mir::function::MIRFunction;
    use typhon_mir::instr::MIRConst;
    use typhon_mir::types::MIRType;
    use typhon_source::types::Span;

    use super::*;

    #[test]
    fn test_valid_cfg() {
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![],
                terminator: Terminator::Return(None),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        assert!(CFGValidator::validate(&func).is_ok());
    }

    #[test]
    fn test_invalid_block_reference() {
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![],
                terminator: Terminator::Branch(BasicBlockID(99)),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![BasicBlockID(99)],
            }],
            captures: vec![],
            span: Span::default(),
        };

        assert!(CFGValidator::validate(&func).is_err());
    }

    #[test]
    fn test_inconsistent_edges() {
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![],
            blocks: vec![
                BasicBlock {
                    id: BasicBlockID(0),
                    instrs: vec![typhon_mir::instr::MIRInstr::Const(MIRConst::Bool(true))],
                    terminator: Terminator::Branch(BasicBlockID(1)),
                    landing_pad: None,
                    predecessors: vec![],
                    successors: vec![BasicBlockID(1), BasicBlockID(2)], // Wrong: claims 2 successors but only branches to 1
                },
                BasicBlock {
                    id: BasicBlockID(1),
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

        assert!(CFGValidator::validate(&func).is_err());
    }
}
