//! Phi node insertion for SSA construction.
//!
//! This module implements the phi node insertion phase of SSA construction,
//! which places phi nodes at dominance frontiers for each variable using
//! the classical Cytron algorithm.

use indexmap::{IndexMap, IndexSet};
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, LocalID, MIRInstr};
use typhon_mir::types::MIRType;

use crate::analysis::DominanceFrontiers;
use crate::error::OptimizerResult;

/// Insert phi nodes at dominance frontiers for all variables.
///
/// This implements the algorithm from Cytron et al., placing phi nodes
/// at join points where multiple definitions of a variable converge.
///
/// # Errors
///
/// Returns an error if the function structure is invalid.
pub fn insert_phi_nodes(
    func: &mut MIRFunction,
    frontiers: &DominanceFrontiers,
) -> OptimizerResult<usize> {
    let mut phi_count = 0;

    // Compute which variables are defined in which blocks
    let definitions = compute_definitions(func);

    // For each variable that has definitions
    for (local_id, def_blocks) in &definitions {
        // Skip if variable is only defined once
        if def_blocks.len() <= 1 {
            continue;
        }

        // Get the type of this local variable
        let local_type = func
            .locals
            .get(local_id.0 as usize)
            .map_or(MIRType::Object { type_id: None }, |l| l.ty.clone());

        // Compute iterated dominance frontier
        let idf = frontiers.iterated_frontier(def_blocks);

        // Insert phi node at each block in the iterated dominance frontier
        for block_id in idf {
            // Find the block and get its predecessors
            if let Some(block) = func.blocks.iter_mut().find(|b| b.id == block_id) {
                let num_preds = block.predecessors.len();

                // Only insert phi if block has multiple predecessors
                if num_preds >= 2 {
                    // Create phi node with placeholders for each predecessor
                    // The actual values will be filled in during variable renaming
                    let incoming = block
                        .predecessors
                        .iter()
                        .map(|&pred_id| (pred_id, typhon_mir::instr::ValueID(0)))
                        .collect();

                    let phi_instr = MIRInstr::Phi { incoming, ty: local_type.clone() };

                    // Insert phi at the beginning of the block
                    block.instrs.insert(0, phi_instr);
                    phi_count += 1;
                }
            }
        }
    }

    Ok(phi_count)
}

/// Compute which variables are defined in which blocks.
///
/// This builds a map from each local variable to the set of basic blocks
/// where that variable is assigned (defined).
#[must_use]
pub fn compute_definitions(func: &MIRFunction) -> IndexMap<LocalID, IndexSet<BasicBlockID>> {
    let mut definitions: IndexMap<LocalID, IndexSet<BasicBlockID>> = IndexMap::new();

    // Walk through all blocks and instructions
    for block in &func.blocks {
        for instr in &block.instrs {
            // Check if this instruction defines a local variable
            if let MIRInstr::Store { local, .. } = instr {
                definitions.entry(*local).or_default().insert(block.id);
            }
        }
    }

    // Also track parameters as being defined in the entry block
    if !func.blocks.is_empty() {
        let entry_block_id = func.blocks[0].id;
        for param in &func.params {
            definitions.entry(param.local).or_default().insert(entry_block_id);
        }
    }

    definitions
}

#[cfg(test)]
mod tests {
    use typhon_mir::block::BasicBlock;
    use typhon_mir::function::{MIRFunction, MIRLocal};
    use typhon_mir::instr::{MIRConst, Terminator, ValueID};
    use typhon_mir::types::MIRType;
    use typhon_source::types::Span;

    use super::*;

    fn create_test_local(name: &str, ty: MIRType) -> MIRLocal {
        MIRLocal { name: Some(name.to_string()), ty, mutable: true }
    }

    #[test]
    fn test_compute_definitions_simple() {
        let local_id = LocalID(0);
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![create_test_local("x", MIRType::Int)],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(1)),
                    MIRInstr::Store { local: local_id, value: ValueID(0) },
                ],
                terminator: Terminator::Return(None),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        let defs = compute_definitions(&func);

        assert_eq!(defs.len(), 1);
        assert!(defs.contains_key(&local_id));
        assert_eq!(defs[&local_id].len(), 1);
        assert!(defs[&local_id].contains(&BasicBlockID(0)));
    }

    #[test]
    fn test_compute_definitions_multiple_blocks() {
        let local_id = LocalID(0);
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MIRType::Int,
            locals: vec![create_test_local("x", MIRType::Int)],
            blocks: vec![
                BasicBlock {
                    id: BasicBlockID(0),
                    instrs: vec![
                        MIRInstr::Const(MIRConst::Int(1)),
                        MIRInstr::Store { local: local_id, value: ValueID(0) },
                    ],
                    terminator: Terminator::Branch(BasicBlockID(1)),
                    landing_pad: None,
                    predecessors: vec![],
                    successors: vec![BasicBlockID(1)],
                },
                BasicBlock {
                    id: BasicBlockID(1),
                    instrs: vec![
                        MIRInstr::Const(MIRConst::Int(2)),
                        MIRInstr::Store { local: local_id, value: ValueID(1) },
                    ],
                    terminator: Terminator::Return(None),
                    landing_pad: None,
                    predecessors: vec![BasicBlockID(0)],
                    successors: vec![],
                },
            ],
            captures: vec![],
            span: Span::default(),
        };

        let defs = compute_definitions(&func);

        assert_eq!(defs.len(), 1);
        assert!(defs.contains_key(&local_id));
        assert_eq!(defs[&local_id].len(), 2);
        assert!(defs[&local_id].contains(&BasicBlockID(0)));
        assert!(defs[&local_id].contains(&BasicBlockID(1)));
    }
}
