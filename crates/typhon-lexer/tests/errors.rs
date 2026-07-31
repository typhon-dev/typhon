//! Tests for [`LexError`] variants produced by the lexer.
//!
//! [`LexError`] currently has seven variants:
//!
//! | Variant              | Source / trigger                                                           |
//! |----------------------|----------------------------------------------------------------------------|
//! | `InvalidToken`       | A character outside the Typhon lexicon (e.g. `$`).                         |
//! | `IndentationError`   | An indent that does not match any level on the stack (mid-block dedent).   |
//! | `Other`              | Constructed by [`LexError::other`] (free-form messages).                   |
//! | `UnexpectedEof`      | Constructed by [`LexError::unexpected_eof`].                               |
//! | `InvalidCharacter`   | Constructed by [`LexError::invalid_character`] / `LexErrorBuilder`.        |
//! | `InvalidIndentation` | Constructed by [`LexError::invalid_indentation`] / `LexErrorBuilder`.      |
//! | `InvalidSyntax`      | Constructed by [`LexError::invalid_syntax`].                               |
//!
//! The lexer's *runtime* produces only the first two variants directly; the
//! remaining variants are part of the public error API used by the
//! `LexErrorBuilder` and by parsers/analysers that wrap lexer output. Every
//! variant is exercised here either via the lexer or via the constructor
//! surface.

use rstest::rstest;
use typhon_lexer::{LexError, LexErrorBuilder, LexErrorKind};
use typhon_source::types::{FileID, Position, SourceSpan};

mod fixtures;

use fixtures::lex_with_diagnostics;

// -----------------------------------------------------------------------------
// `InvalidToken` — produced by the lexer when logos rejects a byte
// -----------------------------------------------------------------------------

#[rstest]
#[case::dollar('$')]
#[case::backtick('`')]
#[case::question_mark('?')]
fn lexing_unknown_punctuation_records_invalid_token_error(#[case] bad: char) {
    let source = bad.to_string();
    let (_kinds, errors, _warnings) = lex_with_diagnostics(&source);

    assert!(
        errors.iter().any(
            |err| matches!(err, LexError::InvalidToken { character, .. } if *character == bad)
        ),
        "expected at least one InvalidToken({bad:?}) error, got {errors:?}"
    );
}

#[test]
fn invalid_token_error_carries_one_based_line_and_column() {
    let (_kinds, errors, _warnings) = lex_with_diagnostics("$");
    let err = errors.iter().find_map(|e| match e {
        LexError::InvalidToken { line, column, character } => Some((*line, *column, *character)),
        _ => None,
    });

    assert_eq!(err, Some((1, 1, '$')), "InvalidToken should carry 1-based source location");
}

// -----------------------------------------------------------------------------
// `IndentationError` — produced by the lexer on inconsistent dedents
// -----------------------------------------------------------------------------

#[test]
fn dedent_to_unknown_level_records_indentation_error() {
    // Outer level 0, inner level 4, attempt to dedent to level 2 (which is
    // not on the stack).
    let source = "if True:\n    if True:\n        pass\n  x = 1";
    let (_kinds, errors, _warnings) = lex_with_diagnostics(source);

    assert!(
        errors.iter().any(|err| matches!(err, LexError::IndentationError { .. })),
        "dedent to a level not on the indent stack should record an \
         IndentationError, got {errors:?}"
    );
}

// -----------------------------------------------------------------------------
// `LexError` constructor surface
// -----------------------------------------------------------------------------

#[test]
fn invalid_character_constructor_populates_fields() {
    let err = LexError::invalid_character(7, 3, '€');

    assert!(matches!(err, LexError::InvalidCharacter { line: 7, column: 3, character: '€' }));
}

#[test]
fn invalid_indentation_constructor_populates_fields() {
    let err = LexError::invalid_indentation(2, 1, 4, 2);

    assert!(matches!(
        err,
        LexError::InvalidIndentation { line: 2, column: 1, expected: 4, found: 2 }
    ));
}

#[test]
fn invalid_syntax_constructor_uses_provided_message() {
    let span = SourceSpan::new(Position::new(1, 1, 0), Position::new(1, 1, 0), FileID::new(1));
    let err = LexError::invalid_syntax("nope", span);

    assert!(matches!(err, LexError::InvalidSyntax { ref message, .. } if message == "nope"));
}

#[test]
fn unexpected_eof_constructor_returns_unit_variant() {
    let err = LexError::unexpected_eof();

    assert!(matches!(err, LexError::UnexpectedEof));
}

#[test]
fn other_constructor_wraps_message() {
    let err = LexError::other("free form");

    assert!(matches!(err, LexError::Other(ref msg) if msg == "free form"));
}

// -----------------------------------------------------------------------------
// `LexErrorBuilder` round-trips
// -----------------------------------------------------------------------------

#[rstest]
#[case::expected_indent(LexErrorKind::ExpectedIndentation)]
#[case::inconsistent_indent(LexErrorKind::InconsistentIndentation)]
#[case::invalid_doc_string_ending(LexErrorKind::InvalidDocStringEnding)]
#[case::invalid_hex_number(LexErrorKind::InvalidHexNumber)]
#[case::invalid_number(LexErrorKind::InvalidNumber)]
#[case::invalid_string_ending(LexErrorKind::InvalidStringEnding)]
#[case::invalid_unicode_escape(LexErrorKind::InvalidUnicodeEscape)]
#[case::tab_in_indentation(LexErrorKind::TabInIndentation)]
fn builder_kinds_collapse_to_other_or_indentation_variant(#[case] kind: LexErrorKind) {
    // These kinds map to either `LexError::Other` or
    // `LexError::IndentationError` depending on the kind. The exact mapping
    // is part of the public contract of `LexErrorBuilder::build`.
    let err = LexErrorBuilder::new().line(1).column(1).kind(kind).build();

    assert!(matches!(err, LexError::Other(_) | LexError::IndentationError { .. }));
}

#[test]
fn builder_kind_invalid_character_maps_to_invalid_character() {
    let err =
        LexErrorBuilder::new().line(2).column(3).kind(LexErrorKind::InvalidCharacter('!')).build();

    assert!(matches!(err, LexError::InvalidCharacter { line: 2, column: 3, character: '!' }));
}

#[test]
fn builder_kind_invalid_token_maps_to_invalid_token() {
    let err =
        LexErrorBuilder::new().line(4).column(5).kind(LexErrorKind::InvalidToken('?')).build();

    assert!(matches!(err, LexError::InvalidToken { line: 4, column: 5, character: '?' }));
}

#[test]
fn builder_kind_invalid_indentation_maps_to_invalid_indentation() {
    let err = LexErrorBuilder::new()
        .line(3)
        .column(0)
        .kind(LexErrorKind::InvalidIndentation { expected: 4, found: 2 })
        .build();

    assert!(matches!(
        err,
        LexError::InvalidIndentation { line: 3, column: 0, expected: 4, found: 2 }
    ));
}

#[test]
fn builder_kind_unexpected_eof_maps_to_unexpected_eof() {
    let err = LexErrorBuilder::new().line(1).column(1).kind(LexErrorKind::UnexpectedEOF).build();

    assert!(matches!(err, LexError::UnexpectedEof));
}

#[test]
fn builder_with_no_kind_falls_back_to_other() {
    let err = LexErrorBuilder::new().build();

    assert!(matches!(err, LexError::Other(_)));
}

// -----------------------------------------------------------------------------
// No false positives on valid input
// -----------------------------------------------------------------------------

#[test]
fn valid_program_produces_no_errors() {
    let (kinds, errors, warnings) = lex_with_diagnostics("x = 42");

    assert!(!kinds.is_empty(), "expected at least one token");
    assert!(errors.is_empty(), "expected no lexer errors, got {errors:?}");
    assert!(warnings.is_empty(), "expected no lexer warnings, got {warnings:?}");
}
