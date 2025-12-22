//! Optimization passes for MIR.

mod constant_fold;
mod dead_code;
mod escape;
mod inline;
mod refcount;
mod ssa;

pub use constant_fold::*;
pub use dead_code::*;
pub use escape::*;
pub use inline::*;
pub use refcount::*;
pub use ssa::*;
