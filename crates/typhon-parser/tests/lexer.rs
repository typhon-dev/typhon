//! Tests for the lexer module.

use std::sync::Arc;

use typhon_parser::diagnostics::DiagnosticReporter;
use typhon_parser::lexer::{Lexer, TokenKind};
use typhon_source::types::{FileID, SourceManager};

fn create_lexer(source: &'_ str) -> Lexer<'_> {
    let source_manager = Arc::new(SourceManager::new());
    let file_id = FileID::new(1);
    let diagnostics = Arc::new(DiagnosticReporter::new(source_manager));

    Lexer::new(source, file_id, diagnostics)
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
