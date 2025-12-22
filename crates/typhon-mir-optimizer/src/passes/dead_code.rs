//! Dead Code Elimination Pass
//!
//! This pass removes instructions whose results are never used and unreachable basic blocks.
//! It works in three phases:
//! 1. Mark reachable blocks via DFS from entry
//! 2. Mark live instructions via backward dataflow
//! 3. Remove dead instructions and blocks

use indexmap::IndexSet;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, MIRInstr, Terminator, ValueID};

use crate::error::OptimizerResult;

/// Dead code elimination optimization pass.
///
/// This pass removes:
/// - Unreachable basic blocks
/// - Instructions whose results are never used
/// - Instructions with no side effects
///
/// Instructions with side effects are always kept:
/// - Function calls
/// - Method calls
/// - Store operations
/// - Reference count operations
/// - Raises and returns
#[derive(Clone, Copy, Debug)]
pub struct DeadCodeEliminator {
    /// Number of dead blocks removed.
    removed_blocks: usize,
    /// Number of dead instructions removed.
    removed_instrs: usize,
}

impl DeadCodeEliminator {
    /// Create a new dead code eliminator.
    #[must_use]
    pub const fn new() -> Self { Self { removed_blocks: 0, removed_instrs: 0 } }

    /// Eliminate dead code from the function.
    ///
    /// Returns `(blocks_removed, instrs_removed)`.
    ///
    /// # Errors
    ///
    /// Returns an error if DCE analysis fails.
    pub fn eliminate_dead_code(
        &mut self,
        func: &mut MIRFunction,
    ) -> OptimizerResult<(usize, usize)> {
        self.removed_blocks = 0;
        self.removed_instrs = 0;

        // Phase 1: Mark reachable blocks
        let reachable = self.find_reachable_blocks(func);

        // Phase 2: Build use-def chains to find live values
        let live_values = self.find_live_values(func, &reachable);

        // Phase 3: Remove dead instructions and blocks
        self.remove_dead_code(func, &reachable, &live_values);

        Ok((self.removed_blocks, self.removed_instrs))
    }

    /// Find all live values using backward dataflow analysis.
    fn find_live_values(
        &self,
        func: &MIRFunction,
        reachable: &IndexSet<BasicBlockID>,
    ) -> IndexSet<ValueID> {
        let mut live = IndexSet::new();

        // Mark all values used in side-effecting instructions as live
        for block in &func.blocks {
            if !reachable.contains(&block.id) {
                continue;
            }

            for instr in &block.instrs {
                // Check if instruction has side effects
                if self.has_side_effects(instr) {
                    // Mark all used values as live
                    self.mark_used_values(instr, &mut live);
                }
            }

            // Terminator uses are always live
            self.mark_terminator_uses(&block.terminator, &mut live);
        }

        live
    }

    /// Find all reachable blocks using depth-first search from entry.
    fn find_reachable_blocks(&self, func: &MIRFunction) -> IndexSet<BasicBlockID> {
        let mut reachable = IndexSet::new();
        let mut worklist = Vec::new();

        if func.blocks.is_empty() {
            return reachable;
        }

        // Start from entry block (always blocks[0])
        let entry_id = func.blocks[0].id;
        worklist.push(entry_id);
        reachable.insert(entry_id);

        while let Some(block_id) = worklist.pop() {
            // Find the block
            if let Some(block) = func.blocks.iter().find(|b| b.id == block_id) {
                // Add successors to worklist
                for &succ_id in &block.successors {
                    if reachable.insert(succ_id) {
                        worklist.push(succ_id);
                    }
                }
            }
        }

        reachable
    }

    /// Check if an instruction has side effects and must be kept.
    const fn has_side_effects(&self, instr: &MIRInstr) -> bool {
        matches!(
            instr,
            MIRInstr::Call { .. }
                | MIRInstr::MethodCall { .. }
                | MIRInstr::Store { .. }
                | MIRInstr::StoreGlobal { .. }
                | MIRInstr::SetItem { .. }
                | MIRInstr::SetAttr { .. }
                | MIRInstr::SetCapture { .. }
                | MIRInstr::IncRef(_)
                | MIRInstr::DecRef(_)
        )
    }

    /// Mark all values used by a terminator as live.
    fn mark_terminator_uses(&self, terminator: &Terminator, live: &mut IndexSet<ValueID>) {
        match terminator {
            Terminator::CondBranch { condition, .. } => {
                live.insert(*condition);
            }
            Terminator::Return(Some(value)) | Terminator::Raise(value) => {
                live.insert(*value);
            }
            Terminator::Invoke { callee, args, .. } => {
                live.insert(*callee);
                for arg in args {
                    live.insert(*arg);
                }
            }
            Terminator::Branch(_) | Terminator::Return(None) | Terminator::Unreachable => {
                // No values used
            }
        }
    }

    /// Mark all values used by an instruction as live.
    fn mark_used_values(&self, instr: &MIRInstr, live: &mut IndexSet<ValueID>) {
        match instr {
            MIRInstr::BinOp { lhs, rhs, .. } => {
                live.insert(*lhs);
                live.insert(*rhs);
            }
            MIRInstr::UnOp { operand, .. } => {
                live.insert(*operand);
            }
            MIRInstr::Call { callee, args, .. } => {
                live.insert(*callee);
                for arg in args {
                    live.insert(*arg);
                }
            }
            MIRInstr::MethodCall { object, args, .. } => {
                live.insert(*object);
                for arg in args {
                    live.insert(*arg);
                }
            }
            MIRInstr::Store { value, .. }
            | MIRInstr::StoreGlobal { value, .. }
            | MIRInstr::Cast { value, .. } => {
                live.insert(*value);
            }
            MIRInstr::GetItem { object, key, .. } => {
                live.insert(*object);
                live.insert(*key);
            }
            MIRInstr::SetItem { object, key, value } => {
                live.insert(*object);
                live.insert(*key);
                live.insert(*value);
            }
            MIRInstr::GetAttr { object, .. } | MIRInstr::InstanceOf { object, .. } => {
                live.insert(*object);
            }
            MIRInstr::SetAttr { object, value, .. } => {
                live.insert(*object);
                live.insert(*value);
            }
            MIRInstr::CreateClosure { captures, .. } => {
                for cap in captures {
                    live.insert(*cap);
                }
            }
            MIRInstr::GetCapture { closure, .. } => {
                live.insert(*closure);
            }
            MIRInstr::SetCapture { closure, value, .. } => {
                live.insert(*closure);
                live.insert(*value);
            }
            MIRInstr::Phi { incoming, .. } => {
                for (_, value) in incoming {
                    live.insert(*value);
                }
            }
            MIRInstr::IncRef(val) | MIRInstr::DecRef(val) => {
                live.insert(*val);
            }
            MIRInstr::Const(_)
            | MIRInstr::Load { .. }
            | MIRInstr::LoadGlobal { .. }
            | MIRInstr::AllocObject { .. }
            | MIRInstr::GetClassAttr { .. } => {
                // These don't use values
            }
        }
    }

    /// Remove dead instructions and blocks.
    fn remove_dead_code(
        &mut self,
        func: &mut MIRFunction,
        reachable: &IndexSet<BasicBlockID>,
        live: &IndexSet<ValueID>,
    ) {
        let mut value_counter = 0u32;

        // Remove dead instructions from reachable blocks
        for block in &mut func.blocks {
            if !reachable.contains(&block.id) {
                continue;
            }

            block.instrs.retain(|instr| {
                let value_id = ValueID(value_counter);
                value_counter += 1;

                // Keep if it has side effects or if its value is used
                let keep = self.has_side_effects(instr) || live.contains(&value_id);
                if !keep {
                    self.removed_instrs += 1;
                }
                keep
            });
        }

        // Remove unreachable blocks
        func.blocks.retain(|block| {
            let keep = reachable.contains(&block.id);
            if !keep {
                self.removed_blocks += 1;
            }
            keep
        });
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self { Self::new() }
}
