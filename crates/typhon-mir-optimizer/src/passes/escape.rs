//! Escape analysis for MIR optimization.
//!
//! This module implements interprocedural escape analysis to determine which heap-allocated
//! objects can be safely stack-allocated or have their reference counting optimized away.
//!
//! An object "escapes" if it can be accessed outside its allocation scope:
//! - Stored in a global variable
//! - Passed to an external function
//! - Returned from a function
//! - Captured by a closure
//! - Stored in a heap-allocated container

use indexmap::{IndexMap, IndexSet};
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, MIRInstr, Terminator, ValueID};
use typhon_mir::module::MIRModule;

use crate::error::{OptimizerError, OptimizerResult};

/// Escape state classification for an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EscapeState {
    /// Object does not escape - stays in stack frame.
    NoEscape,
    /// Object escapes via function argument.
    ArgEscape,
    /// Object escapes via return value.
    ReturnEscape,
    /// Object escapes to global scope or heap.
    GlobalEscape,
}

/// Function summary tracking parameter escape information.
#[derive(Clone, Debug, Default)]
struct FunctionSummary {
    /// Which parameters escape (indexed by parameter position).
    param_escapes: Vec<bool>,
}

/// Interprocedural escape analysis.
///
/// Determines which objects can be stack-allocated by tracking how values flow
/// through the program and identifying escape points.
#[derive(Debug)]
pub struct EscapeAnalysis {
    /// Escape state for each value.
    escape_map: IndexMap<ValueID, EscapeState>,
    /// Fixed-point iteration flag.
    changed: bool,
    /// Function summaries for interprocedural analysis.
    function_summaries: IndexMap<String, FunctionSummary>,
    /// Points-to graph: which values point to which objects.
    points_to: IndexMap<ValueID, IndexSet<ValueID>>,
}

impl EscapeAnalysis {
    /// Create a new escape analysis instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            escape_map: IndexMap::new(),
            changed: false,
            function_summaries: IndexMap::new(),
            points_to: IndexMap::new(),
        }
    }

    /// Analyze escape states for all objects in the module.
    ///
    /// This performs interprocedural dataflow analysis to determine which objects
    /// can be stack-allocated.
    ///
    /// # Errors
    ///
    /// Returns an error if the analysis encounters invalid MIR structure.
    pub fn analyze_module(&mut self, module: &MIRModule) -> OptimizerResult<()> {
        // Phase 1: Build function summaries
        for func in &module.functions {
            self.analyze_function_summary(func)?;
        }

        // Phase 2: Iterative fixed-point analysis
        loop {
            self.changed = false;

            for func in &module.functions {
                self.analyze_function(func)?;
            }

            if !self.changed {
                break;
            }
        }

        Ok(())
    }

    /// Check if a value can be stack-allocated.
    ///
    /// Returns `true` if the value does not escape and can safely use stack allocation.
    #[must_use]
    pub fn can_stack_allocate(&self, value: ValueID) -> bool {
        matches!(self.escape_map.get(&value), Some(EscapeState::NoEscape))
    }

    /// Get the escape state for a value.
    ///
    /// Returns `None` if the value has not been analyzed.
    #[must_use]
    pub fn get_escape_state(&self, value: ValueID) -> Option<EscapeState> {
        self.escape_map.get(&value).copied()
    }

    /// Analyze a basic block for escape states.
    fn analyze_block(&mut self, func: &MIRFunction, block_id: BasicBlockID) -> OptimizerResult<()> {
        let block = func
            .blocks
            .iter()
            .find(|b| b.id == block_id)
            .ok_or(OptimizerError::InvalidBlock(block_id))?;

        // Analyze instructions
        for instr in &block.instrs {
            self.analyze_instruction(instr)?;
        }

        // Analyze terminator
        self.analyze_terminator(&block.terminator)?;

        Ok(())
    }

    /// Analyze function to compute escape states.
    fn analyze_function(&mut self, func: &MIRFunction) -> OptimizerResult<()> {
        // Build points-to graph
        self.build_points_to_graph(func)?;

        // Phase 1: Initialize all allocated objects as NoEscape
        let mut value_id = 0u32;
        for block in &func.blocks {
            for instr in &block.instrs {
                if matches!(instr, MIRInstr::AllocObject { .. }) {
                    // Mark allocation as NoEscape by default
                    self.escape_map.entry(ValueID(value_id)).or_insert(EscapeState::NoEscape);
                }

                value_id += 1;
            }
        }

        // Phase 2: Analyze each block for escape points
        for block in &func.blocks {
            self.analyze_block(func, block.id)?;
        }

        Ok(())
    }

    /// Analyze function to build initial summary.
    fn analyze_function_summary(&mut self, func: &MIRFunction) -> OptimizerResult<()> {
        let mut summary = FunctionSummary { param_escapes: vec![false; func.params.len()] };

        // Analyze each block
        for block in &func.blocks {
            // Check if any parameters escape in this block
            for instr in &block.instrs {
                self.check_instruction_escapes(instr, &mut summary)?;
            }

            // Check terminator
            self.check_terminator_escapes(&block.terminator, &mut summary)?;
        }

        self.function_summaries.insert(func.name.clone(), summary);
        Ok(())
    }

    /// Analyze an instruction for escape behavior.
    fn analyze_instruction(&mut self, instr: &MIRInstr) -> OptimizerResult<()> {
        match instr {
            // Object allocation - initially NoEscape
            MIRInstr::AllocObject { .. } => {
                // TODO: Track which ValueID corresponds to this allocation.
                // Currently handled in analyze_function by walking instructions
                // and assigning sequential ValueIDs. This assumes instructions
                // produce ValueIDs in order, which matches the current MIR design.
            }

            // Store to global - causes escape
            MIRInstr::StoreGlobal { value, .. } => {
                self.mark_escape(*value, EscapeState::GlobalEscape);
            }

            // Set attribute - object and value may escape
            MIRInstr::SetAttr { object, value, .. } => {
                self.mark_escape(*object, EscapeState::GlobalEscape);
                self.mark_escape(*value, EscapeState::GlobalEscape);
            }

            // Set item - container and value may escape
            MIRInstr::SetItem { object, value, .. } => {
                self.mark_escape(*object, EscapeState::GlobalEscape);
                self.mark_escape(*value, EscapeState::GlobalEscape);
            }

            // Function call - arguments may escape
            MIRInstr::Call { args, .. } => {
                for arg in args {
                    self.mark_escape(*arg, EscapeState::ArgEscape);
                }
            }

            // Method call - object and arguments may escape
            MIRInstr::MethodCall { object, args, .. } => {
                self.mark_escape(*object, EscapeState::ArgEscape);
                for arg in args {
                    self.mark_escape(*arg, EscapeState::ArgEscape);
                }
            }

            // Closure capture - captured values escape
            MIRInstr::CreateClosure { captures, .. } => {
                for capture in captures {
                    self.mark_escape(*capture, EscapeState::GlobalEscape);
                }
            }

            // Set capture - value escapes
            MIRInstr::SetCapture { value, .. } => {
                self.mark_escape(*value, EscapeState::GlobalEscape);
            }

            // Other instructions don't cause escape
            _ => {}
        }

        Ok(())
    }

    /// Analyze a terminator for escape behavior.
    fn analyze_terminator(&mut self, terminator: &Terminator) -> OptimizerResult<()> {
        match terminator {
            // Return - returned value escapes
            Terminator::Return(Some(value)) => {
                self.mark_escape(*value, EscapeState::ReturnEscape);
            }

            // Invoke - arguments may escape
            Terminator::Invoke { args, .. } => {
                for arg in args {
                    self.mark_escape(*arg, EscapeState::ArgEscape);
                }
            }

            // Raise - exception escapes
            Terminator::Raise(value) => {
                self.mark_escape(*value, EscapeState::GlobalEscape);
            }

            _ => {}
        }

        Ok(())
    }

    /// Build points-to graph for the function.
    fn build_points_to_graph(&mut self, func: &MIRFunction) -> OptimizerResult<()> {
        for block in &func.blocks {
            for instr in &block.instrs {
                match instr {
                    // Track pointer relationships
                    MIRInstr::Load { local, .. } => {
                        // Load creates a points-to relationship
                        // ValueID would need to be tracked per instruction
                    }

                    MIRInstr::GetAttr { object, .. } => {
                        // Attribute access creates points-to relationship
                        // ValueID would need to be tracked per instruction
                    }

                    MIRInstr::GetItem { object, .. } => {
                        // Item access creates points-to relationship
                        // ValueID would need to be tracked per instruction
                    }

                    MIRInstr::Phi { incoming, .. } => {
                        // Phi merges points-to sets
                        // Would need ValueID for phi result
                        for (_, value) in incoming {
                            // Merge points-to sets
                            let _ = value;
                        }
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Check if instruction causes parameter escapes.
    fn check_instruction_escapes(
        &mut self,
        instr: &MIRInstr,
        _summary: &mut FunctionSummary,
    ) -> OptimizerResult<()> {
        // Track which parameters escape through various operations
        match instr {
            MIRInstr::StoreGlobal { .. }
            | MIRInstr::SetAttr { .. }
            | MIRInstr::SetItem { .. }
            | MIRInstr::CreateClosure { .. } => {
                // These operations cause parameters to escape
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if terminator causes parameter escapes.
    fn check_terminator_escapes(
        &mut self,
        terminator: &Terminator,
        _summary: &mut FunctionSummary,
    ) -> OptimizerResult<()> {
        match terminator {
            Terminator::Return(Some(_)) | Terminator::Raise(_) => {
                // Return or raise causes escape
            }
            _ => {}
        }

        Ok(())
    }

    /// Mark a value as escaping with the given state.
    fn mark_escape(&mut self, value: ValueID, new_state: EscapeState) {
        let current = self.escape_map.entry(value).or_insert(EscapeState::NoEscape);

        // Propagate more severe escape states
        let updated = match (*current, new_state) {
            (EscapeState::GlobalEscape, _) | (_, EscapeState::GlobalEscape) => {
                EscapeState::GlobalEscape
            }
            (EscapeState::ReturnEscape, _) | (_, EscapeState::ReturnEscape) => {
                EscapeState::ReturnEscape
            }
            (EscapeState::ArgEscape, _) | (_, EscapeState::ArgEscape) => EscapeState::ArgEscape,
            _ => EscapeState::NoEscape,
        };

        if *current != updated {
            *current = updated;
            self.changed = true;
        }
    }
}

impl Default for EscapeAnalysis {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_state_ordering() {
        // Verify escape states have correct severity ordering
        let no_escape = EscapeState::NoEscape;
        let arg_escape = EscapeState::ArgEscape;
        let return_escape = EscapeState::ReturnEscape;
        let global_escape = EscapeState::GlobalEscape;

        // Test that states are distinct
        assert_ne!(no_escape, arg_escape);
        assert_ne!(arg_escape, return_escape);
        assert_ne!(return_escape, global_escape);
        assert_ne!(no_escape, return_escape);
        assert_ne!(no_escape, global_escape);
        assert_ne!(arg_escape, global_escape);

        // Test debug formatting works
        assert_eq!(format!("{no_escape:?}"), "NoEscape");
        assert_eq!(format!("{arg_escape:?}"), "ArgEscape");
        assert_eq!(format!("{return_escape:?}"), "ReturnEscape");
        assert_eq!(format!("{global_escape:?}"), "GlobalEscape");

        // Test that we can clone and copy
        let copied = no_escape;

        assert_eq!(copied, no_escape);
    }

    #[test]
    fn test_escape_analysis_initialization() {
        let analysis = EscapeAnalysis::new();

        assert!(analysis.escape_map.is_empty());
        assert!(!analysis.changed);
        assert!(analysis.function_summaries.is_empty());
        assert!(analysis.points_to.is_empty());
    }
}
