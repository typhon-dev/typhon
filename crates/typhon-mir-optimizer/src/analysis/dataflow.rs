//! Generic dataflow analysis framework.
//!
//! This module provides a generic framework for implementing dataflow analyses
//! such as reaching definitions, available expressions, and live variable analysis.

use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::BasicBlockID;

use crate::analysis::ControlFlowGraph;
use crate::error::OptimizerResult;

/// Direction of dataflow analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataflowDirection {
    /// Forward analysis (entry to exit).
    Forward,
    /// Backward analysis (exit to entry).
    Backward,
}

/// Generic dataflow analysis trait.
///
/// Implement this trait to define a specific dataflow analysis.
pub trait DataflowAnalysis {
    /// The type of dataflow facts being computed.
    type Fact: Clone + Eq;

    /// Direction of the analysis.
    fn direction(&self) -> DataflowDirection;

    /// Initial fact for the entry/exit block.
    fn initial_fact(&self) -> Self::Fact;

    /// Bottom value (⊥) for the lattice.
    fn bottom(&self) -> Self::Fact;

    /// Transfer function: compute output fact from input fact for a block.
    fn transfer(
        &self,
        block_id: BasicBlockID,
        func: &MIRFunction,
        input: &Self::Fact,
    ) -> Self::Fact;

    /// Meet operator: combine facts from multiple predecessors/successors.
    fn meet(&self, facts: &[Self::Fact]) -> Self::Fact;
}

/// Dataflow analysis results.
///
/// Contains both entry and exit facts for each block.
#[derive(Debug, Clone)]
pub struct DataflowResults<F> {
    /// Facts at block entry (for forward) or exit (for backward).
    pub entry_facts: FxHashMap<BasicBlockID, F>,
    /// Facts at block exit (for forward) or entry (for backward).
    pub exit_facts: FxHashMap<BasicBlockID, F>,
}

/// Run a dataflow analysis to fixpoint.
///
/// Returns the dataflow facts at entry and exit of each block.
pub fn run_dataflow<A: DataflowAnalysis>(
    func: &MIRFunction,
    cfg: &ControlFlowGraph,
    analysis: &A,
) -> OptimizerResult<DataflowResults<A::Fact>> {
    const MAX_ITERATIONS: usize = 1000;

    let mut entry_facts: FxHashMap<BasicBlockID, A::Fact> = FxHashMap::default();
    let mut exit_facts: FxHashMap<BasicBlockID, A::Fact> = FxHashMap::default();

    // Initialize all blocks with bottom value
    for block in &func.blocks {
        entry_facts.insert(block.id, analysis.bottom());
        exit_facts.insert(block.id, analysis.bottom());
    }

    // Set initial fact for entry/exit block
    if !func.blocks.is_empty() {
        let entry_block = func.blocks[0].id;
        match analysis.direction() {
            DataflowDirection::Forward => {
                entry_facts.insert(entry_block, analysis.initial_fact());
            }
            DataflowDirection::Backward => {
                exit_facts.insert(entry_block, analysis.initial_fact());
            }
        }
    }

    // Worklist algorithm
    let mut worklist: IndexSet<BasicBlockID> = func.blocks.iter().map(|b| b.id).collect();
    let mut iteration_count = 0;

    while let Some(block_id) = worklist.pop() {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            return Err(crate::error::OptimizerError::AnalysisFailed(
                "Dataflow analysis did not converge".to_string(),
            ));
        }

        let old_entry = entry_facts[&block_id].clone();
        let old_exit = exit_facts[&block_id].clone();

        match analysis.direction() {
            DataflowDirection::Forward => {
                // Compute entry fact by meeting predecessor exit facts
                let pred_facts: Vec<A::Fact> = cfg
                    .predecessors(block_id)
                    .iter()
                    .filter_map(|&pred_id| exit_facts.get(&pred_id).cloned())
                    .collect();

                let new_entry = if pred_facts.is_empty() {
                    entry_facts[&block_id].clone()
                } else {
                    analysis.meet(&pred_facts)
                };

                entry_facts.insert(block_id, new_entry.clone());

                // Apply transfer function
                let new_exit = analysis.transfer(block_id, func, &new_entry);
                exit_facts.insert(block_id, new_exit.clone());

                // If exit fact changed, add successors to worklist
                if new_exit != old_exit {
                    for &succ_id in cfg.successors(block_id) {
                        worklist.insert(succ_id);
                    }
                }
            }
            DataflowDirection::Backward => {
                // Compute exit fact by meeting successor entry facts
                let succ_facts: Vec<A::Fact> = cfg
                    .successors(block_id)
                    .iter()
                    .filter_map(|&succ_id| entry_facts.get(&succ_id).cloned())
                    .collect();

                let new_exit = if succ_facts.is_empty() {
                    exit_facts[&block_id].clone()
                } else {
                    analysis.meet(&succ_facts)
                };

                exit_facts.insert(block_id, new_exit.clone());

                // Apply transfer function
                let new_entry = analysis.transfer(block_id, func, &new_exit);
                entry_facts.insert(block_id, new_entry.clone());

                // If entry fact changed, add predecessors to worklist
                if new_entry != old_entry {
                    for &pred_id in cfg.predecessors(block_id) {
                        worklist.insert(pred_id);
                    }
                }
            }
        }
    }

    Ok(DataflowResults { entry_facts, exit_facts })
}

#[cfg(test)]
mod tests {
    use typhon_mir::block::BasicBlock;
    use typhon_mir::instr::Terminator;
    use typhon_mir::types::MIRType;
    use typhon_source::types::Span;

    use super::*;

    /// Simple constant propagation analysis for testing.
    struct ConstantAnalysis;

    impl DataflowAnalysis for ConstantAnalysis {
        type Fact = bool;

        fn direction(&self) -> DataflowDirection { DataflowDirection::Forward }

        fn initial_fact(&self) -> Self::Fact { true }

        fn bottom(&self) -> Self::Fact { false }

        fn transfer(
            &self,
            _block_id: BasicBlockID,
            _func: &MIRFunction,
            input: &Self::Fact,
        ) -> Self::Fact {
            *input
        }

        fn meet(&self, facts: &[Self::Fact]) -> Self::Fact { facts.iter().all(|&f| f) }
    }

    #[test]
    fn test_dataflow_linear() {
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
        let analysis = ConstantAnalysis;
        let results = run_dataflow(&func, &cfg, &analysis).unwrap();

        assert_eq!(results.entry_facts[&BasicBlockID(0)], true);
        assert_eq!(results.entry_facts[&BasicBlockID(1)], true);
    }
}
