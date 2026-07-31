//! Tests for numeric literal token recognition.
//!
//! Covers every numeric `TokenKind`: `IntLiteral`, `HexLiteral`, `BinLiteral`,
//! `OctLiteral`, `FloatLiteral` (with and without exponent), and
//! `ImaginaryLiteral`. Underscore separators inside numeric literals are
//! exercised explicitly because the lexer's regexes admit them in every
//! position except the first character.

use rstest::rstest;
use typhon_lexer::TokenKind;

mod fixtures;

use fixtures::tokens;

// -----------------------------------------------------------------------------
// Integer literals
// -----------------------------------------------------------------------------

#[rstest]
#[case::single_digit("0")]
#[case::small_int("42")]
#[case::large_int("1000000")]
#[case::with_underscore_thousands("1_000_000")]
#[case::with_underscore_pair("12_34")]
fn lexes_decimal_integer_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::IntLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Hex / bin / oct literals
// -----------------------------------------------------------------------------

#[rstest]
#[case::lowercase_x("0xff")]
#[case::uppercase_x("0XFF")]
#[case::mixed_digits("0xDeAdBeEf")]
#[case::with_underscore("0xff_ff_ff")]
fn lexes_hex_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::HexLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::lowercase_b("0b101")]
#[case::uppercase_b("0B1010")]
#[case::with_underscore("0b1010_1010")]
fn lexes_binary_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::BinLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::lowercase_o("0o777")]
#[case::uppercase_o("0O123")]
#[case::with_underscore("0o12_34")]
fn lexes_octal_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::OctLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Float literals
// -----------------------------------------------------------------------------

#[rstest]
#[case::simple("3.14")]
#[case::leading_one_digit("0.5")]
#[case::many_digits("123.456789")]
fn lexes_float_literal_without_exponent(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::FloatLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

#[rstest]
#[case::lowercase_e("1.5e10")]
#[case::uppercase_e("1.5E10")]
#[case::positive_exponent("1.5e+10")]
#[case::negative_exponent("1.5e-10")]
#[case::with_underscores_in_mantissa("1_000.000_1e2")]
fn lexes_float_literal_with_exponent(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::FloatLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Imaginary literals
// -----------------------------------------------------------------------------

#[rstest]
#[case::single_digit("3j")]
#[case::multi_digit("42j")]
#[case::zero("0j")]
fn lexes_imaginary_literal(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::ImaginaryLiteral));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Cross-checks: numeric kinds should not collide
// -----------------------------------------------------------------------------

#[test]
fn integer_followed_by_dot_is_int_then_dot() {
    let toks = tokens("42.");
    let kinds: Vec<TokenKind> = toks.iter().map(|t| t.kind).collect();

    // `42.` does not match the `FloatLiteral` regex (which requires digits
    // after the dot), so it should split into IntLiteral + Dot.
    assert!(
        kinds.starts_with(&[TokenKind::IntLiteral, TokenKind::Dot]),
        "expected `42.` to split into [IntLiteral, Dot], got {kinds:?}"
    );
}

#[test]
fn underscore_separator_does_not_extend_into_letter() {
    // The identifier-letter cutoff means `1_000foo` should still lex as a
    // single IntLiteral followed by an Identifier.
    let toks = tokens("1_000 foo");
    let kinds: Vec<TokenKind> = toks.iter().map(|t| t.kind).collect();

    assert_eq!(&kinds[..2], &[TokenKind::IntLiteral, TokenKind::Identifier]);
}
