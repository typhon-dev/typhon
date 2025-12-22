//! SSA construction pass.
//!
//! Transforms raw MIR into Static Single Assignment (SSA) form using the classical
//! dominance-based algorithm (Cytron et al.).

mod phi_insertion;
mod renaming;

pub use phi_insertion::*;
pub use renaming::*;
use typhon_mir::function::MIRFunction;

use crate::analysis::{DominanceFrontiers, DominatorTree};
use crate::error::OptimizerResult;

/// Construct SSA form for a function.
///
/// This is a three-phase process:
///
/// 1. Compute dominance tree and dominance frontiers
/// 2. Insert phi nodes at dominance frontiers
/// 3. Rename variables to enforce SSA property
///
/// # Errors
///
/// Returns an error if SSA construction fails.
pub fn construct_ssa(func: &mut MIRFunction) -> OptimizerResult<()> {
    // Phase 1: Compute dominance information
    let dom_tree = DominatorTree::compute(func);
    let frontiers = DominanceFrontiers::compute(func, &dom_tree);

    // Phase 2: Insert phi nodes at dominance frontiers
    let phi_count = insert_phi_nodes(func, &frontiers)?;

    // Phase 3: Rename variables to enforce SSA property
    let mut renamer = VariableRenamer::new();
    let _value_count = renamer.rename_variables(func, &dom_tree)?;

    // SSA construction complete (inserted {phi_count} phi nodes)
    let _ = phi_count; // Suppress unused variable warning

    Ok(())
}
