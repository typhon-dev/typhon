//! Test fixtures and utilities for the typhon-lexer test suite.
//!
//! The fixtures are organised into three concerns matching the
//! [`typhon-mir-optimizer/tests/fixtures/`] layout:
//!
//! - [`lexers`]: `#[fixture]` constructors for [`Lexer`] instances.
//! - [`runners`]: helper functions that drive a [`Lexer`] to completion and
//!   surface the resulting tokens, errors, and warnings.
//! - [`sources`]: canned Typhon source snippets used by multiple test files.
//!
//! Each test file shares this module by declaring `mod fixtures;` at the top;
//! Cargo treats the `fixtures/` subdirectory as a module (not a separate test
//! target).
//!
//! [`typhon-mir-optimizer/tests/fixtures/`]: ../../../typhon-mir-optimizer/tests/fixtures/
//! [`Lexer`]: typhon_lexer::Lexer

#![allow(dead_code)]
#![allow(unreachable_pub)]
#![allow(unused_imports)]

// Pull in the lib's `[dependencies]` (`logos`, `rustc-hash`, `thiserror`)
// as anonymous imports so the test target's `unused_crate_dependencies`
// lint doesn't fire in every test file. The lib re-exports types from
// these crates indirectly, so keeping them in scope is harmless.
use {logos as _, rustc_hash as _, thiserror as _};

pub mod lexers;
pub mod runners;
pub mod sources;

pub use lexers::lexer;
pub use runners::{lex_with_diagnostics, spans, token_kinds, tokens};
