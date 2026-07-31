//! Tests for [`LexWarning`] variants produced by the lexer.
//!
//! [`LexWarning`] currently has a single variant, `Message`, used by the
//! lexer to flag mixed tabs and spaces in indentation. The variant is open
//! for future warnings (per the doc-comment in
//! [`crates/typhon-lexer/src/error.rs`](../../src/error.rs)).
//!
//! The tests here cover:
//!
//! - The `Message` variant fires for tab characters in indentation.
//! - Multiple offending tabs accumulate independent warnings.
//! - A tab used *outside* of indentation (for example, between tokens) does
//!   not produce a warning because logos skips it as whitespace.
//! - The constructor [`LexWarning::message`] preserves its inputs.

use typhon_lexer::LexWarning;
use typhon_source::types::{FileID, Position, SourceSpan};

mod fixtures;

use fixtures::lex_with_diagnostics;

// -----------------------------------------------------------------------------
// `LexWarning::Message` produced by the lexer
// -----------------------------------------------------------------------------

#[test]
fn tab_in_indentation_produces_message_warning() {
    // Tab character in indentation triggers the mixed-indent warning.
    let source = "if True:\n\tpass";
    let (_kinds, _errors, warnings) = lex_with_diagnostics(source);

    assert!(
        warnings.iter().any(|w| matches!(w, LexWarning::Message { message, .. } if message.contains("mixing tabs and spaces"))),
        "expected at least one mixed-indent warning, got {warnings:?}"
    );
}

#[test]
fn multiple_tab_characters_accumulate_separate_warnings() {
    // Two indented lines, each starting with a tab, should produce two
    // distinct warnings (one per offending tab).
    let source = "if True:\n\tpass\n\tx = 1";
    let (_kinds, _errors, warnings) = lex_with_diagnostics(source);

    assert_eq!(
        warnings.len(),
        2,
        "expected exactly two mixed-indent warnings (one per indented line), got {warnings:?}"
    );
}

#[test]
fn pure_space_indentation_produces_no_warning() {
    let (_kinds, _errors, warnings) = lex_with_diagnostics("if True:\n    pass");

    assert!(warnings.is_empty(), "pure-space indent should not warn, got {warnings:?}");
}

#[test]
fn tab_between_tokens_is_not_warned_about() {
    // A tab between non-indent positions is treated as ordinary whitespace
    // by logos and should not produce a warning.
    let (_kinds, _errors, warnings) = lex_with_diagnostics("x\t=\t1");

    assert!(
        warnings.is_empty(),
        "a tab between tokens (not in indentation) must not warn: {warnings:?}"
    );
}

// -----------------------------------------------------------------------------
// `LexWarning::message` constructor
// -----------------------------------------------------------------------------

#[test]
fn message_constructor_preserves_message_and_span() {
    let span = SourceSpan::new(Position::new(2, 1, 9), Position::new(2, 1, 9), FileID::new(1));
    let warn = LexWarning::message("hi", span);

    match warn {
        LexWarning::Message { message, span: got_span } => {
            assert_eq!(message, "hi");
            assert_eq!(got_span, span);
        }
    }
}
