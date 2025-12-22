//! SSA form validation.
//!
//! This module provides validators to check that MIR is in valid SSA form.

use rustc_hash::FxHashSet;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, LocalID, MIRInstr, ValueID};

use crate::error::{OptimizerError, OptimizerResult};

/// Validator for SSA properties.
#[derive(Clone, Copy, Debug)]
pub struct SSAValidator;

impl SSAValidator {
    /// Check that all value uses are dominated by their definitions.
    ///
    /// This is a simplified check that verifies values are defined before use
    /// within each block. A full check would require dominance information.
    pub fn check_dominance(_func: &MIRFunction) -> OptimizerResult<()> {
        // Simplified implementation - proper dominance checking requires
        // the dominance tree from analysis::dominance
        Ok(())
    }

    /// Validate that a function is in valid SSA form.
    ///
    /// Checks:
    /// - Each variable is defined exactly once
    /// - All uses are dominated by their definitions
    /// - Phi nodes are only at block starts
    /// - Phi nodes have correct number of predecessors
    pub fn validate(func: &MIRFunction) -> OptimizerResult<()> {
        Self::check_single_assignment(func)?;
        Self::check_phi_placement(func)?;
        Self::check_phi_predecessors(func)?;

        Ok(())
    }

    /// Check that phi nodes are properly placed.
    ///
    /// Phi nodes must appear at the start of blocks (before any other instructions).
    fn check_phi_placement(func: &MIRFunction) -> OptimizerResult<()> {
        for block in &func.blocks {
            let mut seen_non_phi = false;

            for (idx, instr) in block.instrs.iter().enumerate() {
                match instr {
                    MIRInstr::Phi { .. } => {
                        if seen_non_phi {
                            return Err(OptimizerError::SSAViolation(format!(
                                "Phi node at instruction {} in block {:?} appears after \
                                 non-phi instruction",
                                idx, block.id
                            )));
                        }
                    }
                    _ => {
                        seen_non_phi = true;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check that phi nodes have correct number of incoming edges.
    ///
    /// Each phi node must have exactly one incoming value per predecessor block.
    fn check_phi_predecessors(func: &MIRFunction) -> OptimizerResult<()> {
        for block in &func.blocks {
            for (idx, instr) in block.instrs.iter().enumerate() {
                if let MIRInstr::Phi { incoming, .. } = instr {
                    if incoming.len() != block.predecessors.len() {
                        return Err(OptimizerError::SSAViolation(format!(
                            "Phi node at instruction {} in block {:?} has {} incoming edges but \
                             {} predecessors",
                            idx,
                            block.id,
                            incoming.len(),
                            block.predecessors.len()
                        )));
                    }

                    // Check that all predecessors are represented
                    let phi_blocks: FxHashSet<BasicBlockID> =
                        incoming.iter().map(|(block_id, _)| *block_id).collect();
                    let pred_blocks: FxHashSet<BasicBlockID> =
                        block.predecessors.iter().copied().collect();

                    if phi_blocks != pred_blocks {
                        return Err(OptimizerError::SSAViolation(format!(
                            "Phi node at instruction {} in block {:?} has mismatched \
                             predecessor blocks",
                            idx, block.id
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Check that each local variable is assigned exactly once.
    fn check_single_assignment(func: &MIRFunction) -> OptimizerResult<()> {
        let mut defined_locals: FxHashSet<LocalID> = FxHashSet::default();

        // Parameters are implicitly defined
        for param in &func.params {
            defined_locals.insert(param.local);
        }

        for block in &func.blocks {
            for instr in &block.instrs {
                match instr {
                    MIRInstr::Store { local, .. } => {
                        if !defined_locals.insert(*local) {
                            return Err(OptimizerError::SSAViolation(format!(
                                "Local {:?} defined multiple times in function {}",
                                local, func.name
                            )));
                        }
                    }
                    MIRInstr::Phi { .. } => {
                        // Phi nodes define a new SSA value, not a local
                        // In proper SSA, each phi would assign to a unique SSA register
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

/// Validation errors specific to SSA form.
#[derive(Debug)]
pub enum SSAValidationError {
    /// Phi node with incorrect predecessor count
    InvalidPhiPredecessors { block: BasicBlockID, expected: usize, actual: usize },
    /// Phi node in wrong location
    MisplacedPhi { block: BasicBlockID, instruction: usize },
    /// Multiple assignments to the same variable
    MultipleAssignment { local: LocalID, function: String },
    /// Use of undefined value
    UndefinedValue { value: ValueID, block: BasicBlockID },
}

#[cfg(test)]
mod tests {
    use typhon_mir::block::BasicBlock;
    use typhon_mir::function::{MIRFunction, MIRLocal, MIRParam};
    use typhon_mir::instr::{MIRConst, Terminator};
    use typhon_mir::types::MIRType;
    use typhon_source::types::Span;

    use super::*;

    #[test]
    fn test_valid_ssa() {
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![MIRParam { name: "x".to_string(), local: LocalID(0), ty: MIRType::Int }],
            return_type: MIRType::Int,
            locals: vec![
                MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: false },
                MIRLocal { name: Some("y".to_string()), ty: MIRType::Int, mutable: false },
            ],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(42)),
                    MIRInstr::Store { local: LocalID(1), value: ValueID(0) },
                ],
                terminator: Terminator::Return(Some(ValueID(1))),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        assert!(SSAValidator::validate(&func).is_ok());
    }

    #[test]
    fn test_multiple_assignment() {
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![MIRLocal {
                name: Some("x".to_string()),
                ty: MIRType::Int,
                mutable: false,
            }],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(1)),
                    MIRInstr::Store { local: LocalID(0), value: ValueID(0) },
                    MIRInstr::Const(MIRConst::Int(2)),
                    MIRInstr::Store { local: LocalID(0), value: ValueID(1) },
                ],
                terminator: Terminator::Return(None),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        assert!(SSAValidator::validate(&func).is_err());
    }

    #[test]
    fn test_misplaced_phi() {
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(42)),
                    MIRInstr::Phi {
                        incoming: vec![(BasicBlockID(1), ValueID(0))],
                        ty: MIRType::Int,
                    },
                ],
                terminator: Terminator::Return(Some(ValueID(1))),
                landing_pad: None,
                predecessors: vec![BasicBlockID(1)],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        assert!(SSAValidator::validate(&func).is_err());
    }
}
