//! Tests for string-literal token recognition.
//!
//! Covers every string-shaped `TokenKind`:
//!
//! - [`StringLiteral`](typhon_lexer::TokenKind::StringLiteral) and
//!   [`MultilineStringLiteral`](typhon_lexer::TokenKind::MultilineStringLiteral)
//! - [`FmtStringLiteral`](typhon_lexer::TokenKind::FmtStringLiteral) and
//!   [`MultilineFmtStringLiteral`](typhon_lexer::TokenKind::MultilineFmtStringLiteral)
//!   (including `rf` / `fr` raw-format prefixes)
//! - [`TmplStringLiteral`](typhon_lexer::TokenKind::TmplStringLiteral) and
//!   [`MultilineTmplStringLiteral`](typhon_lexer::TokenKind::MultilineTmplStringLiteral)
//!   (including `rt` / `tr` raw-template prefixes)
//! - [`RawStringLiteral`](typhon_lexer::TokenKind::RawStringLiteral)
//! - [`BytesLiteral`](typhon_lexer::TokenKind::BytesLiteral) and
//!   [`MultilineBytesLiteral`](typhon_lexer::TokenKind::MultilineBytesLiteral)
//!   (including `rb` / `br` raw-bytes prefixes)
//!
//! Implicit string concatenation (the
//! [`is_string_literal`](typhon_lexer::is_string_literal) /
//! [`join_string_literals`](typhon_lexer::join_string_literals) flow) is also
//! exercised.

use rstest::rstest;
use typhon_lexer::{TokenKind, is_string_literal};

mod fixtures;

use fixtures::{token_kinds, tokens};

// -----------------------------------------------------------------------------
// Plain string literals
// -----------------------------------------------------------------------------

#[rstest]
#[case::double_quoted("\"hello\"")]
#[case::single_quoted("'hello'")]
#[case::empty_double("\"\"")]
#[case::empty_single("''")]
#[case::with_escape("\"hi\\nthere\"")]
fn lexes_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::StringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::triple_double("\"\"\"hello\"\"\"")]
#[case::triple_single("'''hello'''")]
#[case::with_newlines("\"\"\"line1\nline2\"\"\"")]
fn lexes_multiline_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::MultilineStringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Format strings (incl. raw-format prefixes)
// -----------------------------------------------------------------------------

#[rstest]
#[case::lowercase_f_double("f\"hello {name}\"")]
#[case::lowercase_f_single("f'hello {name}'")]
#[case::raw_format_rf("rf\"raw fmt {x}\"")]
#[case::raw_format_fr("fr\"raw fmt {x}\"")]
fn lexes_format_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::FmtStringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::triple_double("f\"\"\"hi {x}\"\"\"")]
#[case::triple_single("f'''hi {x}'''")]
#[case::raw_format_rf_triple("rf\"\"\"raw multi {x}\"\"\"")]
#[case::raw_format_fr_triple("fr\"\"\"raw multi {x}\"\"\"")]
fn lexes_multiline_format_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::MultilineFmtStringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Template strings (incl. raw-template prefixes)
// -----------------------------------------------------------------------------

#[rstest]
#[case::lowercase_t_double("t\"hello {name}\"")]
#[case::lowercase_t_single("t'hello {name}'")]
#[case::raw_template_rt("rt\"raw tmpl {x}\"")]
#[case::raw_template_tr("tr\"raw tmpl {x}\"")]
fn lexes_template_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::TmplStringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::triple_double("t\"\"\"hi {x}\"\"\"")]
#[case::triple_single("t'''hi {x}'''")]
#[case::raw_template_rt_triple("rt\"\"\"raw multi {x}\"\"\"")]
#[case::raw_template_tr_triple("tr\"\"\"raw multi {x}\"\"\"")]
fn lexes_multiline_template_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::MultilineTmplStringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Raw strings
// -----------------------------------------------------------------------------

#[rstest]
#[case::lowercase_r_double("r\"raw\\nstring\"")]
#[case::lowercase_r_single("r'raw\\nstring'")]
#[case::uppercase_r("R\"raw\"")]
fn lexes_raw_string_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::RawStringLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Bytes literals (incl. raw-bytes prefixes)
// -----------------------------------------------------------------------------

#[rstest]
#[case::lowercase_b_double("b\"bytes\"")]
#[case::lowercase_b_single("b'bytes'")]
#[case::raw_bytes_rb("rb\"raw bytes\"")]
#[case::raw_bytes_br("br\"raw bytes\"")]
fn lexes_bytes_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::BytesLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::triple_double("b\"\"\"hi\"\"\"")]
#[case::triple_single("b'''hi'''")]
#[case::raw_bytes_rb_triple("rb\"\"\"raw multi\"\"\"")]
#[case::raw_bytes_br_triple("br\"\"\"raw multi\"\"\"")]
fn lexes_multiline_bytes_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::MultilineBytesLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Implicit string concatenation
// -----------------------------------------------------------------------------
//
// The lexer flags adjacent string-shaped tokens and emits a *combined* token
// (kind taken from the first string, span widened to cover both lexemes) on
// the second `next()` call. The first string token is still surfaced
// individually, so the resulting stream is `[s1, joined(s1+s2)]` rather than a
// single token. Tests below assert this exact shape so future refactors
// cannot silently drop or duplicate the join.

#[test]
fn adjacent_string_literals_emit_first_then_combined() {
    let kinds = token_kinds("\"hello \" \"world\"");

    assert_eq!(
        kinds,
        vec![TokenKind::StringLiteral, TokenKind::StringLiteral],
        "expected first StringLiteral followed by a combined StringLiteral"
    );
}

#[test]
fn concatenated_second_token_span_covers_both_originals() {
    let toks = tokens("\"a\" \"b\"");

    assert_eq!(toks.len(), 2, "expected two tokens, got {toks:?}");

    // Second token is the join: span covers from start of `"a"` (offset 0) to
    // end of `"b"` (offset 7).
    let joined = &toks[1];

    assert_eq!(joined.kind, TokenKind::StringLiteral);
    assert_eq!(joined.span.start, 0);
    assert_eq!(joined.span.end, 7);
}

#[test]
fn concatenated_first_token_keeps_its_original_span() {
    let toks = tokens("\"a\" \"b\"");

    assert_eq!(toks.len(), 2);

    // First token is the unmodified original `"a"`: span covers offsets 0..3.
    let first = &toks[0];

    assert_eq!(first.kind, TokenKind::StringLiteral);
    assert_eq!(first.span.start, 0);
    assert_eq!(first.span.end, 3);
}

#[test]
fn is_string_literal_recognises_every_string_kind() {
    // Sanity-check the helper used by the implicit-concatenation pathway.
    for kind in [
        TokenKind::BytesLiteral,
        TokenKind::FmtStringLiteral,
        TokenKind::MultilineBytesLiteral,
        TokenKind::MultilineFmtStringLiteral,
        TokenKind::MultilineStringLiteral,
        TokenKind::MultilineTmplStringLiteral,
        TokenKind::RawStringLiteral,
        TokenKind::StringLiteral,
        TokenKind::TmplStringLiteral,
    ] {
        assert!(is_string_literal(kind), "{kind:?} should be classified as a string literal");
    }
}

#[test]
fn is_string_literal_rejects_non_string_kinds() {
    for kind in [TokenKind::Identifier, TokenKind::IntLiteral, TokenKind::Plus, TokenKind::Newline]
    {
        assert!(!is_string_literal(kind), "{kind:?} should not be classified as a string literal");
    }
}
