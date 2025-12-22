//! MIR optimization passes for the Typhon compiler.
//!
//! This crate transforms raw MIR from `typhon-mir-builder` into optimized SSA-form MIR
//! for code generation, achieving significant performance improvements through dataflow
//! analysis and optimization passes.

pub mod analysis;
pub mod config;
pub mod error;
pub mod pass_manager;
pub mod passes;
pub mod validation;

use config::OptimizerConfig;
use error::OptimizerResult;
use pass_manager::PassManager;
use typhon_mir::function::MIRFunction;
use typhon_mir::module::MIRModule;

/// Optimize a module with default configuration (level 2).
///
/// This is a convenience function that creates a pass manager with default settings
/// and runs all enabled optimization passes on the module.
///
/// # Errors
///
/// Returns an error if any optimization pass fails.
pub fn optimize_module(module: &mut MIRModule) -> OptimizerResult<()> {
    let config = OptimizerConfig::default();

    optimize_module_with_config(module, &config)
}

/// Optimize a module with custom configuration.
///
/// # Errors
///
/// Returns an error if any optimization pass fails.
pub fn optimize_module_with_config(
    module: &mut MIRModule,
    config: &OptimizerConfig,
) -> OptimizerResult<()> {
    let mut pass_manager = PassManager::new(*config);

    pass_manager.run_on_module(module)
}

/// Optimize a single function with custom configuration.
///
/// This is primarily used for testing individual optimization passes.
///
/// # Errors
///
/// Returns an error if any optimization pass fails.
pub fn optimize_function(func: &mut MIRFunction, config: &OptimizerConfig) -> OptimizerResult<()> {
    let mut pass_manager = PassManager::new(*config);

    pass_manager.run_on_function(func)
}
