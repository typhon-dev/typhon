//! Code generation module for the Typhon compiler.
//!
//! This module provides code generation from Typhon AST to LLVM IR.
//! The architecture is designed to handle LLVM's complex lifetime requirements by:
//!
//! 1. Separating immutable context from mutable state
//! 2. Using `Arc<Mutex<>>` for thread-safe shared mutable access
//! 3. Carefully managing borrowing patterns to avoid conflicts
//! 4. Ensuring proper lifetimes for LLVM objects
//!
//! The main components are:
//! - `CodeGenContext`: Immutable context for code generation
//! - `CodeGenState`: Mutable state for code generation
//! - `CodeGenerator`: Main code generator that combines context and state
//! - `SymbolTable`: Tracks variables in scope during code generation

mod context;
mod generator;
mod symbol_table;
mod types;
mod visitor;

pub use context::CodeGenContext;
pub use generator::{CodeGenState, CodeGenerator};
pub use symbol_table::{SymbolEntry, SymbolTable};
pub use types::CodeGenValue;
pub use visitor::{DefaultNodeVisitor, NodeVisitor};
