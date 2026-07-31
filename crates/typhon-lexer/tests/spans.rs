//! Tests for [`Token::span`](typhon_lexer::Token::span) start/end byte
//! offsets, parametrised over every token category.
//!
//! Span tests assert only on `start` and `end` to avoid coupling to the
//! lexer's internal arithmetic (line/column tracking). The byte offsets are
//! the same offsets [`logos::Lexer::span`] would report for the underlying
//! byte slice plus the lexer's own synthetic-token bookkeeping.

use rstest::rstest;
use typhon_lexer::TokenKind;

mod fixtures;

use fixtures::tokens;

// -----------------------------------------------------------------------------
// Single-token spans cover the entire source
// -----------------------------------------------------------------------------

#[rstest]
#[case::keyword("def", 0, 3)]
#[case::identifier("foo_bar", 0, 7)]
#[case::int_literal("42", 0, 2)]
#[case::float_literal("3.14", 0, 4)]
#[case::operator("+=", 0, 2)]
#[case::delimiter("(", 0, 1)]
#[case::ellipsis("...", 0, 3)]
#[case::arrow("->", 0, 2)]
#[case::string("\"hi\"", 0, 4)]
#[case::raw_string("r\"hi\"", 0, 5)]
#[case::format_string("f\"hi {x}\"", 0, 9)]
#[case::bytes("b\"hi\"", 0, 5)]
fn first_token_span_matches_source_bounds(
    #[case] source: &str,
    #[case] expected_start: usize,
    #[case] expected_end: usize,
) {
    let toks = tokens(source);
    let first = toks.first().expect("expected at least one token");

    assert_eq!(first.span.start, expected_start, "{source:?} -> {first:?}");
    assert_eq!(first.span.end, expected_end, "{source:?} -> {first:?}");
}

// -----------------------------------------------------------------------------
// Multi-line spans
// -----------------------------------------------------------------------------

#[test]
fn multiline_string_span_covers_all_lines() {
    let source = "\"\"\"line1\nline2\nline3\"\"\"";
    let toks = tokens(source);
    let first = toks.first().expect("expected at least one token");

    assert_eq!(first.kind, TokenKind::MultilineStringLiteral);
    assert_eq!(first.span.start, 0);
    assert_eq!(first.span.end, source.len());
}

#[test]
fn multiline_format_string_span_covers_all_lines() {
    let source = "f\"\"\"hi\n{x}\n\"\"\"";
    let toks = tokens(source);
    let first = toks.first().expect("expected at least one token");

    assert_eq!(first.kind, TokenKind::MultilineFmtStringLiteral);
    assert_eq!(first.span.start, 0);
    assert_eq!(first.span.end, source.len());
}

// -----------------------------------------------------------------------------
// Spans of consecutive tokens are non-overlapping and monotonically advance
// -----------------------------------------------------------------------------

#[test]
fn consecutive_token_spans_are_non_overlapping_and_monotonic() {
    let toks = tokens("x = 42 + y");
    let real: Vec<_> = toks
        .iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::Indent | TokenKind::Dedent | TokenKind::EndOfFile | TokenKind::Newline
            )
        })
        .collect();

    for window in real.windows(2) {
        let [a, b] = window else { unreachable!() };
        assert!(
            a.span.end <= b.span.start,
            "token {a:?} overlaps the next token {b:?}; spans must be non-overlapping"
        );
    }
}

// -----------------------------------------------------------------------------
// Spans of synthetic tokens (Indent/Dedent) are zero-length
// -----------------------------------------------------------------------------

#[test]
fn indent_span_is_zero_length() {
    let toks = tokens("if True:\n    pass");
    let indent = toks.iter().find(|t| t.kind == TokenKind::Indent).expect("expected INDENT");

    assert_eq!(indent.span.start, indent.span.end);
}

#[test]
fn dedent_span_is_zero_length() {
    let toks = tokens("if True:\n    pass");
    let dedent = toks.iter().find(|t| t.kind == TokenKind::Dedent).expect("expected DEDENT");

    assert_eq!(dedent.span.start, dedent.span.end);
}

// -----------------------------------------------------------------------------
// Token at end of source has span.end == source.len()
// -----------------------------------------------------------------------------

#[test]
fn last_real_token_span_end_equals_source_length() {
    let source = "x = 1";
    let toks = tokens(source);
    let last_real = toks
        .iter()
        .rev()
        .find(|t| {
            !matches!(
                t.kind,
                TokenKind::Indent | TokenKind::Dedent | TokenKind::EndOfFile | TokenKind::Newline
            )
        })
        .expect("expected at least one real token");

    assert_eq!(last_real.span.end, source.len());
}
