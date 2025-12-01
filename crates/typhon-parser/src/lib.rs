//! # Typhon Parser
//!
//! A parser for the Typhon programming language, a statically typed language based on Python 3.
//!
//! This crate provides a comprehensive parser for the Typhon language, including lexing,
//! parsing, error reporting, and AST generation. The parser is designed to be memory
//! efficient, robust with error recovery, and easy to integrate into language tools.
//!
//! ## Key Features
//!
//! - **Memory Efficiency**: Uses lifetimes to avoid unnecessary string allocations.
//! - **Error Recovery**: Continues parsing after errors to report multiple issues in a single pass.
//! - **Python-Style Indentation**: Handles Python's indentation-based block structure.
//! - **Visitor Pattern**: Generic visitor for flexible AST traversal.
//! - **Type-Safe AST**: Uses traits and generics for a type-safe AST.
//! - **Rich Error Reporting**: Provides detailed error messages with context.
//!
//! ## Example
//!
//! ```rust,ignore
//! use typhon_parser::{Parser, Parse, Module, SourceManager};
//! use std::sync::Arc;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let source_code = "def greet(name: str) -> str:\n    return f\"Hello, {name}!\"";
//!     let source_manager = Arc::new(SourceManager::new());
//!     let file_id = source_manager.clone().add_file("example.ty".to_string(), source_code.to_string());
//!
//!     let mut parser = Parser::new(source_code, file_id, source_manager);
//!     let module = parser.parse::<Module>()?;
//!
//!     println!("Successfully parsed!");
//!
//!     Ok(())
//! }
//! ```

pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod utils;
