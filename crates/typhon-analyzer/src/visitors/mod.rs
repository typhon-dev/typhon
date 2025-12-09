//! AST visitors for semantic analysis.
//!
//! This module contains visitor implementations that traverse the AST
//! to perform various semantic analysis passes.

mod name_resolver;
mod semantic_validator;
mod symbol_collector;
mod type_checker;

pub use name_resolver::*;
pub use semantic_validator::*;
pub use symbol_collector::*;
pub use type_checker::*;
