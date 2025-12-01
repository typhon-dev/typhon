//! Typhon Compiler Library
//!
//! This crate provides the core components of the Typhon compiler, including the
//! lexer, parser, AST, type system, and code generation.

pub mod backend;
pub mod common;
pub mod driver;
pub mod typesystem;

/// Version of the Typhon compiler
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
