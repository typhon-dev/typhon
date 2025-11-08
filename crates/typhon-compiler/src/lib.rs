//! Typhon Compiler Library
//!
//! This is the main compiler library for the Typhon programming language.

pub mod ast;
pub mod parser;
pub mod analyzer;
pub mod codegen;
pub mod diagnostics;
pub mod types;
pub mod ir;
pub mod transforms;
pub mod utils;

/// Version of the Typhon compiler
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Entry point for the compilation process
pub fn compile(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement compilation pipeline
    todo!("Implement compilation pipeline")
}

/// Create an error diagnostic
pub fn diagnostic(message: &str) -> diagnostics::Diagnostic {
    diagnostics::Diagnostic::new(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
