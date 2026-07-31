//! Helpers that drive a [`Lexer`] to completion and return the collected
//! tokens, kinds, spans, and accumulated diagnostics.
//!
//! Tests should prefer these helpers over hand-rolled loops so that the
//! `take_errors` / `take_warnings` drain pattern is exercised consistently.

use std::ops::Range;

use typhon_lexer::{LexError, LexWarning, Lexer, Token, TokenKind};
use typhon_source::types::FileID;

use super::lexers::TEST_FILE_ID;

/// Run a fresh lexer over `source` and collect the resulting tokens.
///
/// Diagnostics accumulated during lexing are silently discarded; use
/// [`lex_with_diagnostics`] when error/warning coverage matters.
pub fn tokens(source: &str) -> Vec<Token<'_>> {
    let mut lexer = Lexer::new(source, TEST_FILE_ID);
    let collected: Vec<Token<'_>> = (&mut lexer).collect();
    drop(lexer.take_errors());
    drop(lexer.take_warnings());

    collected
}

/// Run a fresh lexer over `source` and project the resulting tokens to their
/// [`TokenKind`] variants.
pub fn token_kinds(source: &str) -> Vec<TokenKind> {
    tokens(source).into_iter().map(|tok| tok.kind).collect()
}

/// Run a fresh lexer over `source` and project the resulting tokens to their
/// byte spans.
pub fn spans(source: &str) -> Vec<Range<usize>> {
    tokens(source).into_iter().map(|tok| tok.span).collect()
}

/// Run a fresh lexer over `source` and return its tokens together with every
/// accumulated [`LexError`] and [`LexWarning`].
///
/// This exercises the new internal-accumulator pattern: the lexer no longer
/// takes a `DiagnosticReporter`; instead it pushes errors and warnings into
/// `Vec`s drained via [`Lexer::take_errors`] / [`Lexer::take_warnings`].
pub fn lex_with_diagnostics(source: &str) -> (Vec<TokenKind>, Vec<LexError>, Vec<LexWarning>) {
    let mut lexer = Lexer::new(source, TEST_FILE_ID);
    let kinds: Vec<TokenKind> = (&mut lexer).map(|t| t.kind).collect();
    let errors = lexer.take_errors();
    let warnings = lexer.take_warnings();

    (kinds, errors, warnings)
}

/// Convenience wrapper that lets tests override the [`FileID`] when the
/// stable default would mask a bug. Currently unused by the suite but kept
/// alongside [`tokens`] so future span tests have a single insertion point.
pub fn tokens_with_file_id(source: &str, file_id: FileID) -> Vec<Token<'_>> {
    let mut lexer = Lexer::new(source, file_id);
    let collected: Vec<Token<'_>> = (&mut lexer).collect();
    drop(lexer.take_errors());
    drop(lexer.take_warnings());

    collected
}
