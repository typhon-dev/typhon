//! Pass orchestration and statistics tracking.

use typhon_mir::function::MIRFunction;
use typhon_mir::module::MIRModule;

use crate::analysis::LivenessAnalysis;
use crate::config::OptimizerConfig;
use crate::error::OptimizerResult;
use crate::passes::{
    ConstantFolder,
    DeadCodeEliminator,
    EscapeAnalysis,
    FunctionInliner,
    RefCountOptimizer,
};

/// Statistics collected during optimization.
#[derive(Clone, Copy, Debug, Default)]
pub struct OptimizerStats {
    /// Number of functions converted to SSA form.
    pub ssa_constructed: usize,
    /// Number of constants folded.
    pub constants_folded: usize,
    /// Number of dead basic blocks removed.
    pub dead_blocks_removed: usize,
    /// Number of dead instructions removed.
    pub dead_instrs_removed: usize,
    /// Number of functions inlined.
    pub functions_inlined: usize,
    /// Number of call sites inlined.
    pub call_sites_inlined: usize,
    /// Number of reference count operations eliminated (total).
    pub refcounts_eliminated: usize,
    /// Number of incref instructions removed.
    pub incref_removed: usize,
    /// Number of decref instructions removed.
    pub decref_removed: usize,
    /// Number of incref/decref pairs eliminated.
    pub refcount_pairs_eliminated: usize,
    /// Number of phi nodes inserted.
    pub phi_nodes_inserted: usize,
    /// Number of variables renamed.
    pub variables_renamed: usize,
    /// Total objects analyzed for escape.
    pub objects_analyzed: usize,
    /// Objects eligible for stack allocation.
    pub stack_allocatable: usize,
}

/// Orchestrates execution of optimization passes.
#[derive(Clone, Copy, Debug, Default)]
pub struct PassManager {
    config: OptimizerConfig,
    stats: OptimizerStats,
}

impl PassManager {
    /// Create a new pass manager with the given configuration.
    #[must_use]
    pub fn new(config: OptimizerConfig) -> Self {
        Self { config, stats: OptimizerStats::default() }
    }

    /// Run optimization passes on a module.
    ///
    /// # Errors
    ///
    /// Returns an error if any optimization pass fails.
    pub fn run_on_module(&mut self, module: &mut MIRModule) -> OptimizerResult<()> {
        // Phase 1: Run function-local optimization passes
        // SSA → Constant Folding → DCE (iterative until fixed point)
        for func in &mut module.functions {
            self.run_on_function(func)?;
        }

        // Phase 2: Run module-level inlining pass
        // Inlining operates across functions and may expose new optimization opportunities
        if self.config.inlining_enabled() {
            let mut inliner = FunctionInliner::new();
            let call_sites_inlined = inliner.inline_functions(module, &self.config)?;

            self.stats.functions_inlined += inliner.functions_inlined();
            self.stats.call_sites_inlined += call_sites_inlined;

            // If inlining occurred, run another round of local optimizations
            // to clean up and optimize the inlined code
            if call_sites_inlined > 0 {
                for func in &mut module.functions {
                    self.run_on_function(func)?;
                }
            }
        }

        Ok(())
    }

    /// Run optimization passes on a single function.
    ///
    /// # Errors
    ///
    /// Returns an error if any optimization pass fails.
    pub fn run_on_function(&mut self, func: &mut MIRFunction) -> OptimizerResult<()> {
        // Constant Folding + Dead Code Elimination
        // Run iteratively until convergence
        for _ in 0..self.config.max_iterations {
            let mut changed = false;

            // Constant folding pass
            if self.config.constant_folding_enabled() {
                let mut folder = ConstantFolder::new();
                let constants_folded = folder.fold_constants(func)?;

                self.stats.constants_folded += constants_folded;
                if constants_folded > 0 {
                    changed = true;
                }
            }

            // Dead code elimination pass
            if self.config.dead_code_enabled() {
                let mut eliminator = DeadCodeEliminator::new();
                let (blocks_removed, instrs_removed) = eliminator.eliminate_dead_code(func)?;
                self.stats.dead_blocks_removed += blocks_removed;
                self.stats.dead_instrs_removed += instrs_removed;

                if blocks_removed > 0 || instrs_removed > 0 {
                    changed = true;
                }
            }

            // If no changes were made, we've reached a fixed point
            if !changed {
                break;
            }
        }

        // Reference Count Optimization
        // Run after constant folding and DCE to optimize refcount operations
        if self.config.refcount_opt_enabled() {
            // Run liveness analysis for refcount optimization
            let liveness = LivenessAnalysis::analyze(func)?;

            // Run escape analysis for refcount optimization
            let escape = EscapeAnalysis::new();
            // Note: For single-function optimization, escape analysis is limited
            // Full interprocedural analysis requires module-level context

            let mut optimizer = RefCountOptimizer::new(liveness, escape);
            let refcounts_removed = optimizer.optimize_refcounts(func)?;

            self.stats.refcounts_eliminated += refcounts_removed;
            self.stats.incref_removed += optimizer.incref_removed();
            self.stats.decref_removed += optimizer.decref_removed();
            self.stats.refcount_pairs_eliminated += optimizer.pairs_eliminated();
        }

        Ok(())
    }

    /// Get statistics from the last optimization run.
    #[must_use]
    pub const fn stats(&self) -> &OptimizerStats { &self.stats }
}
