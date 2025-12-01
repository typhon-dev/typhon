//! The backend module handles code generation from AST to LLVM IR.
//!
//! This module is responsible for:
//! - Mapping Typhon types to LLVM types
//! - Converting AST nodes to LLVM IR
//! - Optimizing the generated code
//! - Error handling during code generation

pub mod codegen;
pub mod error;
pub mod llvm;

#[cfg(test)]
mod tests;

pub use codegen::{
    CodeGenContext,
    CodeGenState,
    CodeGenValue,
    CodeGenerator,
    SymbolEntry,
    SymbolTable,
};
pub use error::{CodeGenError, CodeGenResult};
pub use llvm::LLVMContext;
