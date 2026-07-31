//! Parametrised tests for every non-literal [`TokenKind`].
//!
//! Numeric and string literals have their own dedicated files
//! ([`numbers`](super::numbers), [`strings`](super::strings)). This file
//! covers keywords, keyword literals, type-system tokens, operators,
//! comparisons, assignments, delimiters, identifiers, and soft keywords.

use rstest::rstest;
use typhon_lexer::TokenKind;

mod fixtures;

use fixtures::{token_kinds, tokens};

// -----------------------------------------------------------------------------
// Hard keywords
// -----------------------------------------------------------------------------

#[rstest]
#[case::keyword_and("and", TokenKind::And)]
#[case::keyword_as("as", TokenKind::As)]
#[case::keyword_assert("assert", TokenKind::Assert)]
#[case::keyword_async("async", TokenKind::Async)]
#[case::keyword_await("await", TokenKind::Await)]
#[case::keyword_break("break", TokenKind::Break)]
#[case::keyword_class("class", TokenKind::Class)]
#[case::keyword_continue("continue", TokenKind::Continue)]
#[case::keyword_def("def", TokenKind::Def)]
#[case::keyword_del("del", TokenKind::Del)]
#[case::keyword_elif("elif", TokenKind::Elif)]
#[case::keyword_else("else", TokenKind::Else)]
#[case::keyword_except("except", TokenKind::Except)]
#[case::keyword_finally("finally", TokenKind::Finally)]
#[case::keyword_for("for", TokenKind::For)]
#[case::keyword_from("from", TokenKind::From)]
#[case::keyword_global("global", TokenKind::Global)]
#[case::keyword_if("if", TokenKind::If)]
#[case::keyword_import("import", TokenKind::Import)]
#[case::keyword_in("in", TokenKind::In)]
#[case::keyword_is("is", TokenKind::Is)]
#[case::keyword_lambda("lambda", TokenKind::Lambda)]
#[case::keyword_nonlocal("nonlocal", TokenKind::Nonlocal)]
#[case::keyword_not("not", TokenKind::Not)]
#[case::keyword_or("or", TokenKind::Or)]
#[case::keyword_pass("pass", TokenKind::Pass)]
#[case::keyword_raise("raise", TokenKind::Raise)]
#[case::keyword_return("return", TokenKind::Return)]
#[case::keyword_try("try", TokenKind::Try)]
#[case::keyword_while("while", TokenKind::While)]
#[case::keyword_with("with", TokenKind::With)]
#[case::keyword_yield("yield", TokenKind::Yield)]
fn lexes_hard_keyword_as_its_own_kind(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(
        kinds.first().copied(),
        Some(expected),
        "lexing {source:?} should produce {expected:?} as the first token"
    );
}

// -----------------------------------------------------------------------------
// Keyword literals
// -----------------------------------------------------------------------------

#[rstest]
#[case::keyword_true("True", TokenKind::True)]
#[case::keyword_false("False", TokenKind::False)]
#[case::keyword_none("None", TokenKind::None)]
fn lexes_keyword_literal(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

// -----------------------------------------------------------------------------
// Type-system & misc punctuation
// -----------------------------------------------------------------------------

#[rstest]
#[case::arrow("->", TokenKind::Arrow)]
#[case::ellipsis("...", TokenKind::Ellipsis)]
#[case::colon_equal(":=", TokenKind::ColonEqual)]
#[case::exclamation("!", TokenKind::Exclamation)]
fn lexes_type_system_token(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

// -----------------------------------------------------------------------------
// Operators (arithmetic, bitwise, decorator)
// -----------------------------------------------------------------------------

#[rstest]
#[case::plus("+", TokenKind::Plus)]
#[case::minus("-", TokenKind::Minus)]
#[case::star("*", TokenKind::Star)]
#[case::slash("/", TokenKind::Slash)]
#[case::double_slash("//", TokenKind::DoubleSlash)]
#[case::percent("%", TokenKind::Percent)]
#[case::double_star("**", TokenKind::DoubleStar)]
#[case::left_shift("<<", TokenKind::LeftShift)]
#[case::right_shift(">>", TokenKind::RightShift)]
#[case::ampersand("&", TokenKind::Ampersand)]
#[case::pipe("|", TokenKind::Pipe)]
#[case::caret("^", TokenKind::Caret)]
#[case::tilde("~", TokenKind::Tilde)]
#[case::at("@", TokenKind::At)]
fn lexes_operator(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

// -----------------------------------------------------------------------------
// Comparison operators
// -----------------------------------------------------------------------------

#[rstest]
#[case::less_than("<", TokenKind::LessThan)]
#[case::greater_than(">", TokenKind::GreaterThan)]
#[case::less_equal("<=", TokenKind::LessEqual)]
#[case::greater_equal(">=", TokenKind::GreaterEqual)]
#[case::equal("==", TokenKind::Equal)]
#[case::not_equal("!=", TokenKind::NotEqual)]
fn lexes_comparison_operator(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

// -----------------------------------------------------------------------------
// Assignment operators
// -----------------------------------------------------------------------------

#[rstest]
#[case::assign("=", TokenKind::Assign)]
#[case::plus_equal("+=", TokenKind::PlusEqual)]
#[case::minus_equal("-=", TokenKind::MinusEqual)]
#[case::star_equal("*=", TokenKind::StarEqual)]
#[case::slash_equal("/=", TokenKind::SlashEqual)]
#[case::double_slash_equal("//=", TokenKind::DoubleSlashEqual)]
#[case::percent_equal("%=", TokenKind::PercentEqual)]
#[case::double_star_equal("**=", TokenKind::DoubleStarEqual)]
#[case::left_shift_equal("<<=", TokenKind::LeftShiftEqual)]
#[case::right_shift_equal(">>=", TokenKind::RightShiftEqual)]
#[case::ampersand_equal("&=", TokenKind::AmpersandEqual)]
#[case::pipe_equal("|=", TokenKind::PipeEqual)]
#[case::caret_equal("^=", TokenKind::CaretEqual)]
#[case::at_equal("@=", TokenKind::AtEqual)]
fn lexes_assignment_operator(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

// -----------------------------------------------------------------------------
// Delimiters
// -----------------------------------------------------------------------------

#[rstest]
#[case::left_paren("(", TokenKind::LeftParen)]
#[case::right_paren(")", TokenKind::RightParen)]
#[case::left_bracket("[", TokenKind::LeftBracket)]
#[case::right_bracket("]", TokenKind::RightBracket)]
#[case::left_brace("{", TokenKind::LeftBrace)]
#[case::right_brace("}", TokenKind::RightBrace)]
#[case::comma(",", TokenKind::Comma)]
#[case::colon(":", TokenKind::Colon)]
#[case::dot(".", TokenKind::Dot)]
#[case::semicolon(";", TokenKind::Semicolon)]
fn lexes_delimiter(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

// -----------------------------------------------------------------------------
// Identifiers
// -----------------------------------------------------------------------------

#[rstest]
#[case::lowercase("variable")]
#[case::camel_case("variableName")]
#[case::snake_case("variable_name")]
#[case::leading_underscore("_private")]
#[case::dunder("__init__")]
#[case::with_digits("foo123")]
#[case::single_letter("x")]
fn lexes_identifier(#[case] source: &str) {
    let toks = tokens(source);

    assert_eq!(toks.first().map(|t| t.kind), Some(TokenKind::Identifier));
    assert_eq!(toks.first().map(|t| t.lexeme), Some(source));
}

// -----------------------------------------------------------------------------
// Soft keywords
// -----------------------------------------------------------------------------

#[rstest]
#[case::soft_match("match", TokenKind::Match)]
#[case::soft_case("case", TokenKind::Case)]
#[case::soft_type("type", TokenKind::Type)]
#[case::soft_underscore("_", TokenKind::Underscore)]
fn lexes_soft_keyword(#[case] source: &str, #[case] expected: TokenKind) {
    let kinds = token_kinds(source);

    assert_eq!(kinds.first().copied(), Some(expected));
}

#[test]
fn lexes_simple_assignment_as_three_tokens() {
    let kinds = token_kinds("x = 42");

    assert_eq!(kinds[..3], [TokenKind::Identifier, TokenKind::Assign, TokenKind::IntLiteral]);
}

#[test]
fn comment_is_skipped_and_yields_no_token() {
    let kinds = token_kinds("42 # this is a comment");

    assert_eq!(kinds.first().copied(), Some(TokenKind::IntLiteral));
    assert!(
        !kinds.contains(&TokenKind::Comment),
        "comments are configured with logos::skip and should never appear in the stream"
    );
}
