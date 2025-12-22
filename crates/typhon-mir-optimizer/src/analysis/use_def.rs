//! Use-def chain analysis.
//!
//! This module computes use-def chains, which map each use of a variable
//! to the set of definitions that may reach it.

use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, LocalID, MIRInstr, ValueID};

/// Use-def chains for a function.
///
/// Maps each variable use to the set of definitions that may reach it.
#[derive(Debug, Clone)]
pub struct UseDefChains {
    /// Maps (block, instruction index, local) to set of defining instructions.
    uses: FxHashMap<(BasicBlockID, usize, LocalID), IndexSet<DefSite>>,
}

/// A definition site in the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefSite {
    /// Block containing the definition.
    pub block: BasicBlockID,
    /// Instruction index within the block.
    pub instr_index: usize,
    /// The variable being defined.
    pub local: LocalID,
}

impl UseDefChains {
    /// Compute use-def chains for a function.
    ///
    /// This performs a simple intraprocedural analysis, tracking definitions
    /// and uses within each block.
    #[must_use]
    pub fn compute(func: &MIRFunction) -> Self {
        let mut uses: FxHashMap<(BasicBlockID, usize, LocalID), IndexSet<DefSite>> =
            FxHashMap::default();

        // For each block, track which definitions reach each use
        for block in &func.blocks {
            let mut current_defs: FxHashMap<LocalID, DefSite> = FxHashMap::default();

            // Initialize with parameter definitions (entry block only)
            if block.id == func.blocks[0].id {
                for param in &func.params {
                    current_defs.insert(
                        param.local,
                        DefSite { block: block.id, instr_index: 0, local: param.local },
                    );
                }
            }

            // Process instructions in order
            for (instr_idx, instr) in block.instrs.iter().enumerate() {
                match instr {
                    MIRInstr::Store { local, value } => {
                        // Record use of the value being stored
                        if let Some(use_local) = value_to_local(*value, func)
                            && let Some(&def_site) = current_defs.get(&use_local)
                        {
                            uses.entry((block.id, instr_idx, use_local))
                                .or_default()
                                .insert(def_site);
                        }

                        // This instruction defines the local
                        current_defs.insert(
                            *local,
                            DefSite { block: block.id, instr_index: instr_idx, local: *local },
                        );
                    }
                    MIRInstr::Load { local, .. } => {
                        // Record use of the local being loaded
                        if let Some(&def_site) = current_defs.get(local) {
                            uses.entry((block.id, instr_idx, *local)).or_default().insert(def_site);
                        }
                    }
                    MIRInstr::BinOp { lhs, rhs, .. } => {
                        // Record uses of operands
                        for &value_id in &[*lhs, *rhs] {
                            if let Some(use_local) = value_to_local(value_id, func)
                                && let Some(&def_site) = current_defs.get(&use_local)
                            {
                                uses.entry((block.id, instr_idx, use_local))
                                    .or_default()
                                    .insert(def_site);
                            }
                        }
                    }
                    MIRInstr::UnOp { operand, .. } => {
                        // Record use of operand
                        if let Some(use_local) = value_to_local(*operand, func)
                            && let Some(&def_site) = current_defs.get(&use_local)
                        {
                            uses.entry((block.id, instr_idx, use_local))
                                .or_default()
                                .insert(def_site);
                        }
                    }
                    MIRInstr::Call { callee, args, .. } => {
                        // Record use of callee
                        if let Some(use_local) = value_to_local(*callee, func)
                            && let Some(&def_site) = current_defs.get(&use_local)
                        {
                            uses.entry((block.id, instr_idx, use_local))
                                .or_default()
                                .insert(def_site);
                        }

                        // Record uses of arguments
                        for &arg in args {
                            if let Some(use_local) = value_to_local(arg, func)
                                && let Some(&def_site) = current_defs.get(&use_local)
                            {
                                uses.entry((block.id, instr_idx, use_local))
                                    .or_default()
                                    .insert(def_site);
                            }
                        }
                    }
                    _ => {
                        // Other instructions don't use or define locals
                    }
                }
            }
        }

        Self { uses }
    }

    /// Get the definitions that reach a use.
    ///
    /// Returns the set of definition sites that may reach the given use.
    #[must_use]
    pub fn reaching_defs(
        &self,
        block: BasicBlockID,
        instr_index: usize,
        local: LocalID,
    ) -> IndexSet<DefSite> {
        self.uses.get(&(block, instr_index, local)).cloned().unwrap_or_default()
    }

    /// Check if a variable has exactly one definition reaching a use.
    ///
    /// This is useful for optimization passes that require single definitions.
    #[must_use]
    pub fn has_single_def(&self, block: BasicBlockID, instr_index: usize, local: LocalID) -> bool {
        self.reaching_defs(block, instr_index, local).len() == 1
    }

    /// Get the unique definition reaching a use, if there is exactly one.
    #[must_use]
    pub fn unique_def(
        &self,
        block: BasicBlockID,
        instr_index: usize,
        local: LocalID,
    ) -> Option<DefSite> {
        let defs = self.reaching_defs(block, instr_index, local);
        if defs.len() == 1 { defs.into_iter().next() } else { None }
    }
}

/// Helper to extract a `LocalID` from a `ValueID` if it represents a Load.
///
/// Returns None if the `ValueID` doesn't correspond to a local variable.
fn value_to_local(_value_id: ValueID, _func: &MIRFunction) -> Option<LocalID> {
    // This is a simplified implementation.
    // A full implementation would maintain a mapping from ValueID to its source instruction,
    // then check if that instruction is a Load of a local variable.
    // For now, we return None as this requires additional infrastructure.
    None
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
    fn test_use_def_simple() {
        let local = LocalID(0);
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![MIRParam { name: "x".to_string(), local, ty: MIRType::Int }],
            return_type: MIRType::Int,
            locals: vec![MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: true }],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![
                    MIRInstr::Const(MIRConst::Int(42)),
                    MIRInstr::Store { local, value: ValueID(0) },
                    MIRInstr::Load { local, ty: MIRType::Int },
                ],
                terminator: Terminator::Return(Some(ValueID(1))),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        let chains = UseDefChains::compute(&func);

        // The Load at instruction 2 should see the Store at instruction 1
        let defs = chains.reaching_defs(BasicBlockID(0), 2, local);
        assert_eq!(defs.len(), 1);
        assert_eq!(
            defs.into_iter().next(),
            Some(DefSite { block: BasicBlockID(0), instr_index: 1, local })
        );
    }

    #[test]
    fn test_use_def_parameter() {
        let local = LocalID(0);
        let func = MIRFunction {
            name: "test".to_string(),
            params: vec![MIRParam { name: "x".to_string(), local, ty: MIRType::Int }],
            return_type: MIRType::Int,
            locals: vec![MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: true }],
            blocks: vec![BasicBlock {
                id: BasicBlockID(0),
                instrs: vec![MIRInstr::Load { local, ty: MIRType::Int }],
                terminator: Terminator::Return(Some(ValueID(0))),
                landing_pad: None,
                predecessors: vec![],
                successors: vec![],
            }],
            captures: vec![],
            span: Span::default(),
        };

        let chains = UseDefChains::compute(&func);

        // The Load should see the parameter definition
        let defs = chains.reaching_defs(BasicBlockID(0), 0, local);
        assert_eq!(defs.len(), 1);
        let def = defs.into_iter().next().unwrap();
        assert_eq!(def.block, BasicBlockID(0));
        assert_eq!(def.instr_index, 0);
        assert_eq!(def.local, local);
    }
}
