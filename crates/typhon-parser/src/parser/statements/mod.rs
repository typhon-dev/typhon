//! Statement parsing module.
//!
//! This module contains parsers for all Python statement types, organized by category:
//!
//! - [`core`] - Entry points (`parse_statement`, `parse_block`, `parse_statement_list`) and variable declarations
//! - [`keywords`] - Builtin keyword statements (`assert`, `del`, `global`, `nonlocal`, `pass`, `expression_stmt`)
//! - [`assignments`] - Assignment statements (regular, augmented, annotated)
//! - [`control_flow`] - Control flow statements (if, while, for, break, continue, return, async variants)
//! - [`error_handling`] - Error handling statements (try, except, raise)
//! - [`context_managers`] - Context manager statements (with, async with)
//! - [`helpers`] - Helper functions for statement parsing

mod assignments;
mod context_managers;
mod control_flow;
mod core;
mod error_handling;
mod helpers;
mod keywords;
