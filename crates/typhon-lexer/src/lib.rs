//! # Typhon Lexer
//!
//! Lexical analysis for the Typhon programming language.
//!
//! This crate converts Typhon source code into a stream of [`Token`]s. It uses the
//! [`logos`](https://crates.io/crates/logos) crate for fast tokenization and adds Python-style
//! indentation handling on top: [`TokenKind::Indent`] / [`TokenKind::Dedent`] generation,
//! implicit and explicit line continuations, implicit string concatenation, soft keywords,
//! and tab/space warnings.
//!
//! Diagnostics ([`LexError`], [`LexWarning`]) are accumulated locally on the [`Lexer`] and
//! drained by the consumer (typically the parser) between tokens. This keeps the lexer free
//! of any dependency on the parser's diagnostics module so the dependency graph stays
//! acyclic.
//!
//! ## Example
//!
//! ```rust,ignore
//! use typhon_lexer::{Lexer, TokenKind};
//! use typhon_source::types::FileID;
//!
//! let mut lexer = Lexer::new("def greet(): ...", FileID::new(1));
//! let kinds: Vec<TokenKind> = (&mut lexer).map(|t| t.kind).collect();
//! let errors = lexer.take_errors();
//! let warnings = lexer.take_warnings();
//! ```

#[cfg(test)]
use rstest as _;

mod error;
mod lexer;
mod rules;
mod token;

pub use error::{LexError, LexErrorBuilder, LexErrorKind, LexWarning};
pub use lexer::Lexer;
pub use rules::{
    calculate_indentation,
    check_soft_keyword,
    extract_string_content,
    is_in_template_string_context,
    is_line_continuation,
    is_string_literal,
    join_string_literals,
    soft_keywords,
};
pub use token::{BracketType, Token, TokenKind};
