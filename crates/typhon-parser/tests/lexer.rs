//! Tests for the lexer module.

use typhon_parser::lexer::{LexError, Lexer, TokenKind};
use typhon_source::types::FileID;

fn create_lexer(source: &'_ str) -> Lexer<'_> { Lexer::new(source, FileID::new(1)) }

/// Lex `source` to completion, returning the kinds of every emitted token and
/// every error the lexer accumulated.
///
/// This helper exercises the new internal-accumulator pattern: the lexer no
/// longer takes a `DiagnosticReporter`; instead it pushes errors into a
/// `Vec<LexError>` that consumers drain via [`Lexer::take_errors`]. Tests can
/// therefore assert directly on the lexer's diagnostic output without going
/// through the parser's `Diagnostic` type.
fn lex_with_errors(source: &str) -> (Vec<TokenKind>, Vec<LexError>) {
    let mut lexer = create_lexer(source);
    let kinds: Vec<TokenKind> = (&mut lexer).map(|t| t.kind).collect();
    let errors = lexer.take_errors();

    (kinds, errors)
}

#[test]
fn test_integer_literal() {
    let mut lexer = create_lexer("42");
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::IntLiteral);
    assert_eq!(token.lexeme(), "42");
}

#[test]
fn test_float_literal() {
    let mut lexer = create_lexer("3.14");
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::FloatLiteral);
    assert_eq!(token.lexeme(), "3.14");
}

#[test]
fn test_string_literal() {
    let mut lexer = create_lexer("\"hello\"");
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::StringLiteral);
}

#[test]
fn test_identifier() {
    let mut lexer = create_lexer("variable_name");
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::Identifier);
    assert_eq!(token.lexeme(), "variable_name");
}

#[test]
fn test_keywords() {
    let keywords = vec![
        ("def", TokenKind::Def),
        ("class", TokenKind::Class),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("elif", TokenKind::Elif),
        ("while", TokenKind::While),
        ("for", TokenKind::For),
        ("return", TokenKind::Return),
        ("True", TokenKind::True),
        ("False", TokenKind::False),
        ("None", TokenKind::None),
    ];

    for (source, expected_kind) in keywords {
        let mut lexer = create_lexer(source);
        let token = lexer.next().expect("Expected token");
        assert_eq!(
            token.kind, expected_kind,
            "Expected {:?} for '{}', got {:?}",
            expected_kind, source, token.kind
        );
    }
}

#[test]
fn test_operators() {
    let operators = vec![
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("//", TokenKind::DoubleSlash),
        ("==", TokenKind::Equal),
        ("!=", TokenKind::NotEqual),
        ("<", TokenKind::LessThan),
        (">", TokenKind::GreaterThan),
        ("<=", TokenKind::LessEqual),
        (">=", TokenKind::GreaterEqual),
    ];

    for (source, expected_kind) in operators {
        let mut lexer = create_lexer(source);
        let token = lexer.next().expect("Expected token");
        assert_eq!(
            token.kind, expected_kind,
            "Expected {:?} for '{}', got {:?}",
            expected_kind, source, token.kind
        );
    }
}

#[test]
fn test_delimiters() {
    let delimiters = vec![
        ("(", TokenKind::LeftParen),
        (")", TokenKind::RightParen),
        ("[", TokenKind::LeftBracket),
        ("]", TokenKind::RightBracket),
        ("{", TokenKind::LeftBrace),
        ("}", TokenKind::RightBrace),
        (":", TokenKind::Colon),
        (",", TokenKind::Comma),
        (".", TokenKind::Dot),
    ];

    for (source, expected_kind) in delimiters {
        let mut lexer = create_lexer(source);
        let token = lexer.next().expect("Expected token");
        assert_eq!(
            token.kind, expected_kind,
            "Expected {:?} for '{}', got {:?}",
            expected_kind, source, token.kind
        );
    }
}

#[test]
fn test_fstring_token() {
    let mut lexer = create_lexer("f\"hello {name}\"");
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::FmtStringLiteral);
}

#[test]
fn test_lex_with_errors_invalid_token() {
    // The `$` character is not part of the Typhon lexicon; the lexer should
    // record an `InvalidToken` error and the helper should surface it.
    let (_kinds, errors) = lex_with_errors("$");

    assert!(
        errors.iter().any(|err| matches!(err, LexError::InvalidToken { character: '$', .. })),
        "expected at least one InvalidToken('$') error, got {errors:?}"
    );
}

#[test]
fn test_lex_with_errors_no_errors() {
    let (kinds, errors) = lex_with_errors("x = 42");

    assert!(!kinds.is_empty(), "expected at least one token");
    assert!(errors.is_empty(), "expected no lexer errors, got {errors:?}");
}

#[test]
fn test_multiline_string() {
    let source = "\"\"\"
    multiline
    string
    \"\"\"";
    let mut lexer = create_lexer(source);
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::MultilineStringLiteral);
}

#[test]
fn test_comment_ignored() {
    let mut lexer = create_lexer("42 # this is a comment");
    let token = lexer.next().expect("Expected token");

    assert_eq!(token.kind, TokenKind::IntLiteral);

    // Next token should be EOF, not the comment
    let token2 = lexer.next();

    assert!(token2.is_none() || token2.unwrap().kind == TokenKind::EndOfFile);
}

#[test]
fn test_indentation() {
    let source = "if True:\n    pass";
    let mut lexer = create_lexer(source);

    assert_eq!(lexer.next().unwrap().kind, TokenKind::If);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::True);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Colon);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Newline);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Indent);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Pass);
}

#[test]
fn test_multiple_tokens() {
    let mut lexer = create_lexer("x = 42 + y");

    assert_eq!(lexer.next().unwrap().kind, TokenKind::Identifier);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Assign);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Plus);
    assert_eq!(lexer.next().unwrap().kind, TokenKind::Identifier);
}
