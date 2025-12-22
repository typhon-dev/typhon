//! Test fixtures and utilities.

mod builders;
mod helpers;
mod validators;

pub use builders::{
    create_function_with_instrs,
    create_function_with_instrs_and_terminator,
    create_module_with_function,
};
pub use helpers::{create_empty_function, create_test_function};
pub use validators::{count_binop_instrs, count_blocks, count_const_instrs, count_instrs};
