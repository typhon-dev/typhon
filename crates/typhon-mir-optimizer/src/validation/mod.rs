//! MIR validation infrastructure.
//!
//! This module provides validators to check the correctness and consistency
//! of MIR after transformations.

mod cfg_validator;
mod ssa_validator;

pub use cfg_validator::*;
pub use ssa_validator::*;
