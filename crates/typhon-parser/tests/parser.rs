//! Comprehensive parser tests mirroring the src/parser structure.
//!
//! This test suite validates all parser functionality including:
//! - Expression parsing (literals, operators, comprehensions, etc.)
//! - Statement parsing (assignments, control flow, error handling, etc.)
//! - Declaration parsing (functions, classes, types, variables)
//! - Pattern matching
//! - Type annotations
//! - Module parsing
//! - Edge cases and error recovery

mod declaration;
mod expression;
mod identifier;
mod lexer;
mod module;
mod pattern;
mod statement;
mod types;
