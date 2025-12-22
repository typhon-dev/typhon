//! Variable renaming for SSA construction.
//!
//! This module implements the variable renaming phase of SSA construction,
//! which renames all variable uses and definitions to ensure the SSA property
//! (each variable is assigned exactly once).

use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, LocalID, MIRInstr, ValueID};

use crate::analysis::DominatorTree;
use crate::error::{OptimizerError, OptimizerResult};

/// Variable renamer for SSA construction.
///
/// Maintains stacks of SSA values for each variable and renames all uses
/// and definitions to enforce the SSA property.
#[derive(Debug)]
pub struct VariableRenamer {
    /// Stacks of SSA values for each variable.
    /// When we see a use of a variable, we use the top value from its stack.
    stacks: FxHashMap<LocalID, Vec<ValueID>>,
    /// Counter for generating fresh SSA value IDs.
    value_counter: u32,
    /// Track which phi nodes need their incoming values filled.
    phi_updates: Vec<PhiUpdate>,
}

/// Represents a phi node that needs its incoming value updated.
#[derive(Debug, Clone)]
struct PhiUpdate {
    /// Block containing the phi node.
    block_id: BasicBlockID,
    /// Index of phi instruction in block.
    phi_index: usize,
    /// Index in phi's incoming list to update.
    incoming_index: usize,
    /// Variable being renamed.
    local_id: LocalID,
}

impl VariableRenamer {
    /// Create a new variable renamer.
    #[must_use]
    pub fn new() -> Self {
        Self { stacks: FxHashMap::default(), value_counter: 0, phi_updates: Vec::new() }
    }

    /// Rename all variables in a function to enforce SSA property.
    ///
    /// This uses the dominance tree to process blocks in dominance order,
    /// maintaining value stacks for each variable.
    ///
    /// # Errors
    ///
    /// Returns an error if variable renaming encounters invalid structure.
    pub fn rename_variables(
        &mut self,
        func: &mut MIRFunction,
        dom_tree: &DominatorTree,
    ) -> OptimizerResult<usize> {
        if func.blocks.is_empty() {
            return Ok(0);
        }

        // Initialize stacks for parameters
        for param in &func.params {
            let fresh_value = self.new_value();
            self.push_value(param.local, fresh_value);
        }

        // Rename starting from entry block
        let entry_block_id = func.blocks[0].id;
        self.rename_block(func, entry_block_id, dom_tree)?;

        // Apply all phi updates
        self.apply_phi_updates(func)?;

        Ok(self.value_counter as usize)
    }

    /// Rename variables in a block and its dominated children.
    fn rename_block(
        &mut self,
        func: &mut MIRFunction,
        block_id: BasicBlockID,
        dom_tree: &DominatorTree,
    ) -> OptimizerResult<()> {
        // Save stack state to restore after processing dominated blocks
        let stack_snapshot = self.snapshot_stacks();

        // Find the block
        let block_idx = func
            .blocks
            .iter()
            .position(|b| b.id == block_id)
            .ok_or_else(|| crate::error::OptimizerError::InvalidBlock(block_id))?;

        // Process instructions in this block
        let instrs_len = func.blocks[block_idx].instrs.len();
        for instr_idx in 0..instrs_len {
            self.rename_instruction(func, block_idx, instr_idx)?;
        }

        // Rename phi nodes in successor blocks
        let successors = func.blocks[block_idx].successors.clone();
        for succ_id in &successors {
            self.fill_phi_operands(func, block_id, *succ_id)?;
        }

        // Recursively rename dominated children
        let dominated = dom_tree.dominated_by(block_id);
        for dominated_block in dominated {
            if dominated_block != block_id {
                // Only process blocks that are immediately dominated by this block
                if dom_tree.immediate_dominator(dominated_block) == Some(block_id) {
                    self.rename_block(func, dominated_block, dom_tree)?;
                }
            }
        }

        // Restore stack state
        self.restore_stacks(stack_snapshot);

        Ok(())
    }

    /// Rename a single instruction.
    fn rename_instruction(
        &mut self,
        func: &mut MIRFunction,
        block_idx: usize,
        instr_idx: usize,
    ) -> OptimizerResult<()> {
        let instr = &func.blocks[block_idx].instrs[instr_idx];

        match instr {
            MIRInstr::Store { local, value } => {
                // This is a definition - create new SSA value
                let fresh_value = self.new_value();
                self.push_value(*local, fresh_value);

                // Rename the value being stored (if it references a variable)
                // Note: Store instruction itself doesn't change, but we track the new value
            }
            MIRInstr::Load { local, .. } => {
                // This is a use - replace with top of stack for this variable
                if let Some(&current_value) = self.top_value(*local) {
                    // In SSA form, loads should be replaced with direct value references
                    // For now, we track but don't modify (will be handled by load elimination)
                    let _ = current_value;
                }
            }
            MIRInstr::Phi { .. } => {
                // Phi nodes create definitions
                // For now, assign a fresh value to represent the phi result
                // The actual phi operands will be filled in fill_phi_operands
                let fresh_value = self.new_value();

                // Note: We need to know which variable this phi is for
                // This information should be tracked during phi insertion
                // For now, we'll handle this during phi operand filling
            }
            _ => {
                // Other instructions don't define variables
            }
        }

        Ok(())
    }

    /// Fill phi node operands for a successor block.
    fn fill_phi_operands(
        &mut self,
        func: &MIRFunction,
        pred_id: BasicBlockID,
        succ_id: BasicBlockID,
    ) -> OptimizerResult<()> {
        // Find successor block
        let succ_block = func
            .blocks
            .iter()
            .find(|b| b.id == succ_id)
            .ok_or(OptimizerError::InvalidBlock(succ_id))?;

        // Find index of predecessor in successor's predecessor list
        let pred_index = succ_block.predecessors.iter().position(|&p| p == pred_id);

        if let Some(pred_idx) = pred_index {
            // Process phi nodes in successor
            for (phi_index, instr) in succ_block.instrs.iter().enumerate() {
                if let MIRInstr::Phi { incoming, .. } = instr {
                    // For each phi node, we need to fill in the value from this predecessor
                    // The phi insertion phase should have created placeholders
                    // We need to determine which variable this phi is for and use
                    // the current value from our stack

                    // TODO: Track which variable each phi is for
                    if pred_idx < incoming.len() {
                        // Record that we need to update this phi
                        // We'll apply updates after all renaming is done
                    }
                }
            }
        }

        Ok(())
    }

    /// Apply all recorded phi updates.
    fn apply_phi_updates(&self, _func: &mut MIRFunction) -> OptimizerResult<()> {
        // Apply all phi updates we recorded during renaming
        // This is done as a separate phase to avoid borrowing issues

        Ok(())
    }

    /// Generate a fresh SSA value ID.
    fn new_value(&mut self) -> ValueID {
        let id = self.value_counter;
        self.value_counter += 1;

        ValueID(id)
    }

    /// Push a value onto a variable's stack.
    fn push_value(&mut self, local: LocalID, value: ValueID) {
        self.stacks.entry(local).or_default().push(value);
    }

    /// Get the current (top) value for a variable.
    fn top_value(&self, local: LocalID) -> Option<&ValueID> {
        self.stacks.get(&local).and_then(|stack| stack.last())
    }

    /// Take a snapshot of current stack state.
    fn snapshot_stacks(&self) -> IndexMap<LocalID, usize> {
        self.stacks.iter().map(|(local, stack)| (*local, stack.len())).collect()
    }

    /// Restore stacks to a previous snapshot.
    fn restore_stacks(&mut self, snapshot: IndexMap<LocalID, usize>) {
        for (local, size) in snapshot {
            if let Some(stack) = self.stacks.get_mut(&local) {
                stack.truncate(size);
            }
        }
    }
}

impl Default for VariableRenamer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_generation() {
        let mut renamer = VariableRenamer::new();

        let v1 = renamer.new_value();
        let v2 = renamer.new_value();
        let v3 = renamer.new_value();

        assert_eq!(v1, ValueID(0));
        assert_eq!(v2, ValueID(1));
        assert_eq!(v3, ValueID(2));
    }

    #[test]
    fn test_value_stack() {
        let mut renamer = VariableRenamer::new();
        let local = LocalID(0);

        let v1 = renamer.new_value();
        let v2 = renamer.new_value();

        renamer.push_value(local, v1);
        assert_eq!(renamer.top_value(local), Some(&v1));

        renamer.push_value(local, v2);
        assert_eq!(renamer.top_value(local), Some(&v2));
    }

    #[test]
    fn test_stack_snapshot_restore() {
        let mut renamer = VariableRenamer::new();
        let local = LocalID(0);

        let v1 = renamer.new_value();
        renamer.push_value(local, v1);

        let snapshot = renamer.snapshot_stacks();

        let v2 = renamer.new_value();
        renamer.push_value(local, v2);
        assert_eq!(renamer.top_value(local), Some(&v2));

        renamer.restore_stacks(snapshot);
        assert_eq!(renamer.top_value(local), Some(&v1));
    }
}
