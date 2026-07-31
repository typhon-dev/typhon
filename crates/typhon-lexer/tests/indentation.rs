//! Tests for indentation tracking: synthetic
//! [`Indent`](typhon_lexer::TokenKind::Indent) and
//! [`Dedent`](typhon_lexer::TokenKind::Dedent) generation.
//!
//! Covers:
//!
//! - Simple INDENT after a colon-introduced block
//! - Multi-level nested INDENT/DEDENT pairing
//! - DEDENT generation at end of file when the indent stack still has
//!   pending levels (no trailing newline)
//! - Blank lines and comment-only lines do *not* affect the indent stack
//! - Indentation tokens are suppressed inside brackets (implicit
//!   continuation)
//! - The [`calculate_indentation`](typhon_lexer::calculate_indentation) helper
//!   reports tab-vs-space mixing

use typhon_lexer::{TokenKind, calculate_indentation};

mod fixtures;

use fixtures::{token_kinds, tokens};

// -----------------------------------------------------------------------------
// Synthetic INDENT / DEDENT tokens
// -----------------------------------------------------------------------------

#[test]
fn indented_block_emits_indent_token() {
    let kinds = token_kinds("if True:\n    pass");

    assert!(
        kinds.windows(2).any(|w| w == [TokenKind::Newline, TokenKind::Indent]),
        "expected an INDENT token after the introductory newline, got {kinds:?}"
    );
    assert!(kinds.contains(&TokenKind::Pass));
}

#[test]
fn nested_block_emits_two_indent_tokens() {
    let kinds = token_kinds("if True:\n    if False:\n        pass\n");
    let indent_count = kinds.iter().filter(|k| **k == TokenKind::Indent).count();

    assert_eq!(indent_count, 2, "two-level nesting should emit two INDENT tokens, got {kinds:?}");
}

#[test]
fn dedent_returns_to_outer_block() {
    let kinds = token_kinds("if True:\n    pass\noutside");
    let dedent_count = kinds.iter().filter(|k| **k == TokenKind::Dedent).count();

    assert!(dedent_count >= 1, "expected at least one DEDENT, got {kinds:?}");
    assert!(kinds.contains(&TokenKind::Identifier));
}

#[test]
fn eof_terminated_block_emits_dedent() {
    // Source ends inside an indented block; the lexer must emit a DEDENT
    // when the iterator is drained instead of leaving the indent stack
    // dangling.
    let kinds = token_kinds("if True:\n    pass");

    assert!(
        kinds.contains(&TokenKind::Dedent),
        "EOF inside an indented block should emit a DEDENT, got {kinds:?}"
    );
}

#[test]
fn multi_level_dedent_emits_one_dedent_per_level() {
    let source = "if True:\n    if False:\n        pass\n    x = 1\ny = 2";
    let kinds = token_kinds(source);
    let dedent_count = kinds.iter().filter(|k| **k == TokenKind::Dedent).count();

    // Two levels of nesting → at least two DEDENT tokens by EOF.
    assert!(
        dedent_count >= 2,
        "expected ≥2 DEDENT tokens for two-level nesting, got {dedent_count} in {kinds:?}"
    );
}

// -----------------------------------------------------------------------------
// Blank lines & comments inside an indented block
// -----------------------------------------------------------------------------
//
// The current lexer treats blank and comment-only lines as `is_blank_or_comment`
// for *indent processing on that line*, but the empty-line newline still flips
// `at_line_start = true`, so the *next* non-blank line re-runs the indent
// comparison. The block continues uninterrupted: every token of the indented
// content reappears after the blank/comment line. The tests below pin that
// behaviour: the `pass` body and the `x = 1` body are both inside the same
// `INDENT` level and produce the same identifier/keyword tokens.

#[test]
fn blank_line_inside_block_preserves_inner_tokens() {
    let kinds = token_kinds("if True:\n    pass\n\n    x = 1");

    assert!(
        kinds.contains(&TokenKind::Pass),
        "first body token (pass) must survive the blank line: {kinds:?}"
    );
    assert!(
        kinds.contains(&TokenKind::Assign),
        "second body token (x = 1) must survive the blank line: {kinds:?}"
    );
}

#[test]
fn comment_only_line_preserves_inner_tokens_and_emits_no_comment_token() {
    let kinds = token_kinds("if True:\n    pass\n# top-level comment\n    x = 1");

    assert!(
        kinds.contains(&TokenKind::Pass),
        "first body token must survive the comment line: {kinds:?}"
    );
    assert!(
        kinds.contains(&TokenKind::Assign),
        "second body token must survive the comment line: {kinds:?}"
    );
    assert!(
        !kinds.contains(&TokenKind::Comment),
        "comments use logos::skip and should never appear in the stream: {kinds:?}"
    );
}

// -----------------------------------------------------------------------------
// Brackets suppress INDENT
// -----------------------------------------------------------------------------

#[test]
fn newline_inside_brackets_does_not_emit_indent() {
    let kinds = token_kinds("f(\n    1,\n    2,\n)");

    assert!(
        !kinds.contains(&TokenKind::Indent),
        "newlines inside `(`…`)` are implicit continuations: \
         no INDENT/DEDENT/Newline tokens should be emitted. got {kinds:?}"
    );
}

#[test]
fn newline_inside_brackets_does_not_emit_dedent() {
    let kinds = token_kinds("f(\n    1,\n    2,\n)");

    assert!(
        !kinds.contains(&TokenKind::Dedent),
        "newlines inside `(`…`)` are implicit continuations: \
         no INDENT/DEDENT/Newline tokens should be emitted. got {kinds:?}"
    );
}

#[test]
fn list_literal_spans_lines_without_indent_tokens() {
    let kinds = token_kinds("x = [\n    1,\n    2,\n]");

    assert!(!kinds.contains(&TokenKind::Indent));
    assert!(!kinds.contains(&TokenKind::Dedent));
}

// -----------------------------------------------------------------------------
// `calculate_indentation` helper
// -----------------------------------------------------------------------------

#[test]
fn calculate_indentation_counts_spaces() {
    let (level, mixed) = calculate_indentation("    foo");

    assert_eq!(level, 4);
    assert!(!mixed);
}

#[test]
fn calculate_indentation_counts_tab_as_eight_spaces() {
    let (level, mixed) = calculate_indentation("\tfoo");

    assert_eq!(level, 8);
    assert!(mixed, "a leading tab should set the mixed-indentation flag");
}

#[test]
fn calculate_indentation_handles_empty_line() {
    let (level, mixed) = calculate_indentation("");

    assert_eq!(level, 0);
    assert!(!mixed);
}

// -----------------------------------------------------------------------------
// Spans on synthetic tokens
// -----------------------------------------------------------------------------

#[test]
fn indent_token_has_zero_length_span() {
    let toks = tokens("if True:\n    pass");
    let indent = toks
        .iter()
        .find(|t| t.kind == TokenKind::Indent)
        .expect("expected at least one INDENT token");

    assert_eq!(
        indent.span.start, indent.span.end,
        "INDENT is a synthetic token and should have a zero-length span"
    );
    assert_eq!(indent.lexeme, "");
}
