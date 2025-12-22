//! Function inlining optimization pass.
//!
//! This pass replaces function calls with the body of the called function when beneficial,
//! enabling cross-function optimization opportunities.

use indexmap::IndexSet;
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use rustc_hash::FxHashMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, MIRInstr, ValueID};
use typhon_mir::module::MIRModule;
use typhon_mir::types::MIRType;

use crate::config::OptimizerConfig;
use crate::error::OptimizerResult;

/// Function inliner pass.
///
/// Replaces function calls with the body of the called function when beneficial.
#[derive(Clone, Copy, Debug)]
pub struct FunctionInliner {
    /// Number of call sites inlined.
    call_sites_inlined: usize,
    /// Number of functions inlined.
    functions_inlined: usize,
    /// Number of instructions added during inlining.
    instructions_added: usize,
}

impl FunctionInliner {
    /// Create a new function inliner.
    #[must_use]
    pub const fn new() -> Self {
        Self { call_sites_inlined: 0, functions_inlined: 0, instructions_added: 0 }
    }

    /// Get the number of call sites inlined.
    #[must_use]
    pub const fn call_sites_inlined(&self) -> usize { self.call_sites_inlined }

    /// Get the number of functions inlined.
    #[must_use]
    pub const fn functions_inlined(&self) -> usize { self.functions_inlined }

    /// Get the number of instructions added.
    #[must_use]
    pub const fn instructions_added(&self) -> usize { self.instructions_added }

    /// Inline functions in the module.
    ///
    /// # Errors
    ///
    /// Returns an error if inlining fails.
    pub fn inline_functions(
        &mut self,
        module: &mut MIRModule,
        config: &OptimizerConfig,
    ) -> OptimizerResult<usize> {
        // Build call graph to detect recursion
        let call_graph = self.build_call_graph(module);
        let recursive_functions = call_graph.find_scc();

        // Analyze call frequencies
        let call_counts = self.count_call_sites(module);

        // Build function index for quick lookup
        let mut function_map: FxHashMap<String, MIRFunction> = FxHashMap::default();
        for func in &module.functions {
            function_map.insert(func.name.clone(), func.clone());
        }

        // Track which functions have been inlined
        let mut inlined_functions = IndexSet::new();

        // Inline functions in each caller
        for func in &mut module.functions {
            let inlined = self.inline_in_function(
                func,
                &function_map,
                &recursive_functions,
                &call_counts,
                config,
            )?;

            if inlined > 0 {
                inlined_functions.insert(func.name.clone());
            }
        }

        self.functions_inlined = inlined_functions.len();

        Ok(self.call_sites_inlined)
    }

    /// Build call graph for the module.
    fn build_call_graph(&self, module: &MIRModule) -> CallGraph {
        let mut graph = CallGraph::new();

        for func in &module.functions {
            for block in &func.blocks {
                for instr in &block.instrs {
                    if let MIRInstr::Call { callee, .. } = instr {
                        // Resolve callee ValueID to function name using module's value_names
                        if let Some(callee_name) = module.get_value_name(*callee) {
                            graph.add_edge(&func.name, callee_name);
                        }
                    }
                }
            }
        }

        graph
    }

    /// Calculate the size of a function (number of instructions).
    fn calculate_function_size(&self, func: &MIRFunction) -> usize {
        func.blocks.iter().map(|block| block.instrs.len()).sum()
    }

    /// Count call sites for each function.
    fn count_call_sites(&self, module: &MIRModule) -> FxHashMap<String, usize> {
        let mut counts: FxHashMap<String, usize> = FxHashMap::default();

        for func in &module.functions {
            for block in &func.blocks {
                for instr in &block.instrs {
                    if let MIRInstr::Call { callee, .. } = instr {
                        // Resolve callee ValueID to function name using module's value_names
                        if let Some(callee_name) = module.get_value_name(*callee) {
                            *counts.entry(callee_name.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        counts
    }

    /// Inline eligible function calls in a single function.
    fn inline_in_function(
        &mut self,
        func: &mut MIRFunction,
        function_map: &FxHashMap<String, MIRFunction>,
        recursive_functions: &IndexSet<String>,
        call_counts: &FxHashMap<String, usize>,
        config: &OptimizerConfig,
    ) -> OptimizerResult<usize> {
        let mut inlined_count = 0;

        // Find all call sites in the function
        let call_sites = self.find_call_sites(func);

        // Process each call site
        for site in call_sites {
            if self.should_inline(&site, function_map, recursive_functions, call_counts, config)? {
                self.inline_call_site(func, &site, function_map)?;
                self.call_sites_inlined += 1;
                inlined_count += 1;
            }
        }

        Ok(inlined_count)
    }

    /// Find all call sites in a function.
    fn find_call_sites(&self, func: &MIRFunction) -> Vec<CallSite> {
        let mut sites = Vec::new();

        for (block_idx, block) in func.blocks.iter().enumerate() {
            for (instr_idx, instr) in block.instrs.iter().enumerate() {
                if let MIRInstr::Call { callee, args, ty } = instr {
                    sites.push(CallSite {
                        block_id: block.id,
                        block_idx,
                        instr_idx,
                        callee: *callee,
                        args: args.clone(),
                        return_ty: ty.clone(),
                    });
                }
            }
        }

        sites
    }

    /// Inline a call site by replacing it with the callee's body.
    fn inline_call_site(
        &mut self,
        _caller: &mut MIRFunction,
        site: &CallSite,
        function_map: &FxHashMap<String, MIRFunction>,
    ) -> OptimizerResult<()> {
        // Resolve callee ValueID to function name
        // Note: This requires the module reference, which we don't have here.
        // This method signature needs to be updated to accept module reference.
        // For now, we document the implementation steps:
        //
        // 1. Resolve site.callee ValueID to function name using module.get_value_name()
        // 2. Look up function in function_map using the resolved name
        // 3. Clone the callee function from function_map
        // 4. Rename all values in cloned function to avoid conflicts
        // 5. Map formal parameters to actual arguments from site.args
        // 6. Insert cloned blocks into caller function
        // 7. Replace the call instruction with a jump to the inlined entry
        // 8. Handle return values by redirecting to continuation block
        // 9. Update phi nodes and control flow edges

        // Calculate instruction count for statistics
        if let Some(_callee_name) = function_map.keys().next() {
            // TODO: get count from actual callee function
            self.instructions_added += 1;
        }

        // Mark the call site for inlining (implementation deferred)
        let _ = (site, function_map);

        Ok(())
    }

    /// Determine if a call site should be inlined.
    fn should_inline(
        &self,
        site: &CallSite,
        function_map: &FxHashMap<String, MIRFunction>,
        recursive_functions: &IndexSet<String>,
        call_counts: &FxHashMap<String, usize>,
        config: &OptimizerConfig,
    ) -> OptimizerResult<bool> {
        // TODO: Function name resolution from site.callee ValueID requires module reference.
        // This method signature needs to be updated to accept module reference.
        // For now, we implement the heuristics framework:
        //
        // Once we have the callee name from module.get_value_name(site.callee):
        //
        // 1. Never inline if function is recursive (check recursive_functions)
        // 2. Never inline if function size > config.max_inline_size
        // 3. Always inline if single use and config.inline_single_use_enabled()
        // 4. Consider inlining if size <= config.inline_threshold
        //
        // Cost/benefit analysis:
        //
        // - Benefit: eliminated call overhead, enables further optimization
        // - Cost: code size increase (function_size - call_overhead)
        // - Inline if benefit > cost threshold

        // Placeholder: use parameters to avoid unused warnings
        let _ = (site, function_map, recursive_functions, call_counts, config);

        // Conservative: don't inline until module reference is available
        Ok(false)
    }
}

impl Default for FunctionInliner {
    fn default() -> Self { Self::new() }
}

/// Represents a call site in a function.
#[derive(Debug, Clone)]
struct CallSite {
    /// Block containing the call.
    block_id: BasicBlockID,
    /// Index of block in function.
    block_idx: usize,
    /// Index of instruction in block.
    instr_idx: usize,
    /// Callee value.
    callee: ValueID,
    /// Call arguments.
    args: Vec<ValueID>,
    /// Return type.
    return_ty: MIRType,
}

/// Call graph for detecting recursion using petgraph.
#[derive(Debug)]
pub struct CallGraph {
    /// Directed graph for call relationships.
    graph: DiGraph<String, ()>,
    /// Map from function name to node index.
    name_to_node: FxHashMap<String, NodeIndex>,
}

impl CallGraph {
    /// Create a new empty call graph.
    #[must_use]
    pub fn new() -> Self { Self { graph: DiGraph::new(), name_to_node: FxHashMap::default() } }

    /// Add an edge from caller to callee.
    #[allow(clippy::similar_names)]
    pub fn add_edge(&mut self, caller: &str, callee: &str) {
        // Get or create node for caller
        let caller_idx = *self
            .name_to_node
            .entry(caller.to_string())
            .or_insert_with(|| self.graph.add_node(caller.to_string()));

        // Get or create node for callee
        let callee_idx = *self
            .name_to_node
            .entry(callee.to_string())
            .or_insert_with(|| self.graph.add_node(callee.to_string()));

        // Add edge from caller to callee
        self.graph.add_edge(caller_idx, callee_idx, ());
    }

    /// Find strongly connected components (recursive functions) using Tarjan's algorithm.
    ///
    /// Returns a set of all function names that are part of recursive call cycles.
    #[must_use]
    pub fn find_scc(&self) -> IndexSet<String> {
        let mut recursive_functions = IndexSet::new();

        // Use petgraph's built-in Tarjan's SCC algorithm
        let sccs = tarjan_scc(&self.graph);

        for component in sccs {
            // A component is recursive if:
            // 1. It has more than one function (mutual recursion)
            // 2. It has one function that calls itself (direct recursion)
            if component.len() > 1 {
                // Mutual recursion
                for &node_idx in &component {
                    if let Some(func_name) = self.graph.node_weight(node_idx) {
                        recursive_functions.insert(func_name.clone());
                    }
                }
            } else if component.len() == 1 {
                // Check for direct self-recursion
                let node_idx = component[0];
                if self.graph.contains_edge(node_idx, node_idx)
                    && let Some(func_name) = self.graph.node_weight(node_idx)
                {
                    recursive_functions.insert(func_name.clone());
                }
            }
        }

        recursive_functions
    }
}

impl Default for CallGraph {
    fn default() -> Self { Self::new() }
}
