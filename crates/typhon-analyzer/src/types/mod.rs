//! Type system for semantic analysis.
//!
//! This module provides the type representation and type environment infrastructure
//! for the Typhon type system. It includes:
//!
//! - [`Type`]: Core type representation
//! - [`TypeEnvironment`]: Type environment for tracking type information

mod constraints;
mod environment;
mod ty;

pub use constraints::*;
pub use environment::*;
pub use ty::*;
