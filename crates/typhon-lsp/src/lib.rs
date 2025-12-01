//! Typhon Language Server Protocol implementation.
//!
//! This crate provides a Language Server Protocol (LSP) implementation for the
//! Typhon programming language. It uses the tower-lsp framework for implementing
//! the LSP protocol and integrates with the Typhon compiler for language analysis.

pub mod capabilities;
pub mod document;
pub mod handlers;
pub mod server;
pub mod utils;

#[cfg(test)]
pub mod tests;

// Re-export server struct for convenience
pub use server::TyphonLanguageServer;
