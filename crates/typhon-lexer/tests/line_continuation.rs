//! Tests for line-continuation handling.
//!
//! Two flavours are exercised:
//!
//! - **Implicit** continuation inside `(`/`[`/`{`. Newlines and indentation
//!   are suppressed entirely while at least one bracket is open.
//! - **Explicit** continuation via a trailing backslash before a newline.
//!
//! The pure helper [`is_line_continuation`](typhon_lexer::is_line_continuation)
//! is also tested directly because it is part of the crate's public API.

use rstest::rstest;
use typhon_lexer::{TokenKind, is_line_continuation};

mod fixtures;

use fixtures::token_kinds;

// -----------------------------------------------------------------------------
// Implicit continuation inside brackets
// -----------------------------------------------------------------------------

#[rstest]
#[case::parens("f(\n    1,\n    2,\n)")]
#[case::brackets("xs = [\n    1,\n    2,\n]")]
#[case::braces("d = {\n    1: 2,\n    3: 4,\n}")]
fn newlines_inside_brackets_emit_no_newline_token(#[case] source: &str) {
    let kinds = token_kinds(source);

    assert!(
        !kinds.contains(&TokenKind::Newline),
        "newlines inside brackets are implicit continuations and must not emit \
         Newline tokens: {kinds:?}"
    );
}

#[rstest]
#[case::parens("f(\n    1,\n    2,\n)")]
#[case::brackets("xs = [\n    1,\n    2,\n]")]
#[case::braces("d = {\n    1: 2,\n    3: 4,\n}")]
fn newlines_inside_brackets_emit_no_indent_or_dedent(#[case] source: &str) {
    let kinds = token_kinds(source);

    assert!(!kinds.contains(&TokenKind::Indent), "{kinds:?}");
    assert!(!kinds.contains(&TokenKind::Dedent), "{kinds:?}");
}

#[test]
fn closing_bracket_resumes_indentation_tracking() {
    // After the closing `)` the lexer is back outside brackets, so the
    // *next* newline is reported as a Newline token.
    let kinds = token_kinds("f(\n    1,\n)\nx = 1");

    assert!(
        kinds.contains(&TokenKind::Newline),
        "the newline after the closing `)` should re-enable Newline emission: \
         {kinds:?}"
    );
}

#[test]
fn nested_brackets_remain_in_continuation_mode_until_outermost_closes() {
    // `[(...)]` keeps `in_brackets > 0` until the very last `]`.
    let kinds = token_kinds("xs = [(\n    1,\n    2,\n)]");

    assert!(!kinds.contains(&TokenKind::Newline), "{kinds:?}");
    assert!(!kinds.contains(&TokenKind::Indent), "{kinds:?}");
}

// -----------------------------------------------------------------------------
// Explicit backslash continuation
// -----------------------------------------------------------------------------

// The lexer has no `Backslash` token; a stray `\\` outside of a string
// literal is rejected by logos and the iterator records an
// `InvalidToken('\\')` error before terminating. The
// [`is_line_continuation`] helper still reports the case truthfully — see
// [`is_line_continuation_returns_true_for_newline_followed_by_backslash`].
// This test pins the iterator-level outcome so a future "treat backslash
// as a continuation" implementation has to update the contract here too.

#[test]
fn lone_backslash_in_arithmetic_records_invalid_token_error() {
    use fixtures::lex_with_diagnostics;

    let (kinds, errors, _warnings) = lex_with_diagnostics("1 + \\\n2");

    assert!(
        kinds.starts_with(&[TokenKind::IntLiteral, TokenKind::Plus]),
        "expected stream to start with `IntLiteral, Plus`: {kinds:?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, typhon_lexer::LexError::InvalidToken { character: '\\', .. })),
        "expected an InvalidToken('\\\\') error from the stray backslash, got {errors:?}"
    );
}

// -----------------------------------------------------------------------------
// `is_line_continuation` helper
// -----------------------------------------------------------------------------

#[rstest]
#[case::open_parens(TokenKind::Plus, 1, None, true)]
#[case::nested_brackets(TokenKind::Identifier, 3, None, true)]
fn is_line_continuation_returns_true_inside_brackets(
    #[case] kind: TokenKind,
    #[case] in_brackets: usize,
    #[case] next_char: Option<char>,
    #[case] expected: bool,
) {
    assert_eq!(is_line_continuation(kind, in_brackets, next_char), expected);
}

#[test]
fn is_line_continuation_returns_true_for_newline_followed_by_backslash() {
    assert!(is_line_continuation(TokenKind::Newline, 0, Some('\\')));
}

#[test]
fn is_line_continuation_returns_false_for_newline_without_backslash() {
    assert!(!is_line_continuation(TokenKind::Newline, 0, Some('x')));
    assert!(!is_line_continuation(TokenKind::Newline, 0, None));
}

#[test]
fn is_line_continuation_returns_false_for_non_newline_outside_brackets() {
    assert!(!is_line_continuation(TokenKind::Identifier, 0, None));
    assert!(!is_line_continuation(TokenKind::Plus, 0, Some('\\')));
}
