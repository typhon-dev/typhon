//! Dominance analysis for SSA construction.
//!
//! This module implements dominance tree and dominance frontier computation
//! using the Lengauer-Tarjan algorithm via petgraph.

use indexmap::{IndexMap, IndexSet};
use petgraph::algo::dominators::simple_fast;
use petgraph::graph::{DiGraph, NodeIndex};
use rustc_hash::FxHashMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::BasicBlockID;

/// Dominance tree computed using Lengauer-Tarjan algorithm.
#[derive(Debug, Clone)]
pub struct DominatorTree {
    /// Immediate dominators for each block.
    /// `idom[block]` is the immediate dominator of `block`, or `None` for the entry block.
    idom: IndexMap<BasicBlockID, Option<BasicBlockID>>,
    /// All dominators for each block (transitive closure).
    all_doms: IndexMap<BasicBlockID, IndexSet<BasicBlockID>>,
}

impl DominatorTree {
    /// Get all blocks that this block dominates (including itself).
    #[must_use]
    pub fn dominated_by(&self, block: BasicBlockID) -> IndexSet<BasicBlockID> {
        let mut dominated = IndexSet::new();
        for (bid, doms) in &self.all_doms {
            if doms.contains(&block) {
                dominated.insert(*bid);
            }
        }

        dominated
    }

    /// Check if block `a` dominates block `b`.
    ///
    /// A block dominates another if all paths from entry to the dominated block
    /// pass through the dominating block.
    #[must_use]
    pub fn dominates(&self, a: BasicBlockID, b: BasicBlockID) -> bool {
        self.all_doms.get(&b).is_some_and(|doms| doms.contains(&a))
    }

    /// Get the immediate dominator of a block.
    ///
    /// Returns None for the entry block.
    #[must_use]
    pub fn immediate_dominator(&self, block: BasicBlockID) -> Option<BasicBlockID> {
        self.idom.get(&block).copied().flatten()
    }

    /// Compute dominance tree for a function.
    ///
    /// Uses the Lengauer-Tarjan algorithm via petgraph for efficient dominance computation.
    #[must_use]
    pub fn compute(func: &MIRFunction) -> Self {
        if func.blocks.is_empty() {
            return Self { idom: IndexMap::new(), all_doms: IndexMap::new() };
        }

        // Build CFG graph for petgraph
        let mut graph = DiGraph::new();
        let mut block_to_node: FxHashMap<BasicBlockID, NodeIndex> = FxHashMap::default();
        let mut node_to_block: FxHashMap<NodeIndex, BasicBlockID> = FxHashMap::default();

        // Create nodes for all blocks
        for block in &func.blocks {
            let node = graph.add_node(block.id);
            block_to_node.insert(block.id, node);
            node_to_block.insert(node, block.id);
        }

        // Add edges for control flow
        for block in &func.blocks {
            let from_node = block_to_node[&block.id];
            for &succ_id in &block.successors {
                if let Some(&to_node) = block_to_node.get(&succ_id) {
                    graph.add_edge(from_node, to_node, ());
                }
            }
        }

        // Entry block is always blocks[0]
        let entry_node = block_to_node[&func.blocks[0].id];

        // Compute dominators using petgraph
        let dominators = simple_fast(&graph, entry_node);

        // Build immediate dominator map
        let mut idom = IndexMap::new();
        let mut all_doms = IndexMap::new();

        for block in &func.blocks {
            let node = block_to_node[&block.id];

            // Get immediate dominator
            if let Some(idom_node) = dominators.immediate_dominator(node) {
                if idom_node == node {
                    // Entry block dominates itself but has no idom
                    idom.insert(block.id, None);
                } else {
                    idom.insert(block.id, Some(node_to_block[&idom_node]));
                }
            } else {
                // Entry block
                idom.insert(block.id, None);
            }

            // Compute all dominators (walk up the idom chain)
            let mut doms = IndexSet::new();
            let mut current = block.id;
            doms.insert(current); // Block dominates itself

            while let Some(Some(dom_id)) = idom.get(&current) {
                doms.insert(*dom_id);
                current = *dom_id;
            }

            all_doms.insert(block.id, doms);
        }

        Self { idom, all_doms }
    }
}

/// Dominance frontiers for SSA construction.
#[derive(Debug, Clone)]
pub struct DominanceFrontiers {
    /// Dominance frontier for each block.
    /// `frontiers[block]` is the set of blocks in the dominance frontier of `block`.
    frontiers: IndexMap<BasicBlockID, IndexSet<BasicBlockID>>,
}

impl DominanceFrontiers {
    /// Get the dominance frontier of a block.
    ///
    /// Returns an empty set if the block has no dominance frontier.
    #[must_use]
    pub fn frontier(&self, block: BasicBlockID) -> IndexSet<BasicBlockID> {
        self.frontiers.get(&block).cloned().unwrap_or_default()
    }

    /// Get iterated dominance frontier for a set of blocks.
    ///
    /// This is the fixpoint of repeatedly taking dominance frontiers.
    #[must_use]
    pub fn iterated_frontier(&self, blocks: &IndexSet<BasicBlockID>) -> IndexSet<BasicBlockID> {
        let mut result = IndexSet::new();
        let mut worklist: Vec<BasicBlockID> = blocks.iter().copied().collect();

        while let Some(block) = worklist.pop() {
            for &frontier_block in &self.frontier(block) {
                if result.insert(frontier_block) {
                    worklist.push(frontier_block);
                }
            }
        }

        result
    }

    /// Compute dominance frontiers from a dominance tree.
    ///
    /// The dominance frontier of a block X is the set of blocks Y such that:
    /// - X dominates a predecessor of Y
    /// - X does not strictly dominate Y
    #[must_use]
    pub fn compute(func: &MIRFunction, dom_tree: &DominatorTree) -> Self {
        let mut frontiers: IndexMap<BasicBlockID, IndexSet<BasicBlockID>> = IndexMap::new();

        // Initialize empty frontier for each block
        for block in &func.blocks {
            frontiers.insert(block.id, IndexSet::new());
        }

        // For each block Y with predecessors
        for block in &func.blocks {
            let y = block.id;

            // If Y has multiple predecessors, it's a join point
            if block.predecessors.len() >= 2 {
                // For each predecessor P of Y
                for &pred_id in &block.predecessors {
                    let mut runner = pred_id;

                    // Walk up the dominator tree from P
                    // Add Y to the frontier of all blocks from P up to (but not including) Y's idom
                    loop {
                        // Add Y to runner's dominance frontier
                        frontiers.entry(runner).or_default().insert(y);

                        // If runner is Y's immediate dominator, stop
                        if let Some(y_idom) = dom_tree.immediate_dominator(y) {
                            if runner == y_idom {
                                break;
                            }
                        } else {
                            // Y is entry block, stop
                            break;
                        }

                        // Move up to runner's immediate dominator
                        if let Some(runner_idom) = dom_tree.immediate_dominator(runner) {
                            runner = runner_idom;
                        } else {
                            // Reached entry block
                            break;
                        }
                    }
                }
            }
        }

        Self { frontiers }
    }
}
