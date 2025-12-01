//! Tests for the lexer module

use std::sync::Arc;

use typhon_parser::diagnostics::DiagnosticReporter;
use typhon_parser::lexer::{Lexer, TokenKind};
use typhon_source::types::SourceManager;

fn setup_lexer(source: &str) -> Lexer<'_> {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());
    let diagnostic_reporter = Arc::new(DiagnosticReporter::new(Arc::new(source_manager)));

    Lexer::new(source, file_id, diagnostic_reporter)
}

#[test]
fn test_simple_tokens() {
    let source = "x = 42";
    let mut lexer = setup_lexer(source);

    let token1 = lexer.next().unwrap();
    assert_eq!(token1.kind, TokenKind::Identifier);
    assert_eq!(token1.lexeme, "x");

    let token2 = lexer.next().unwrap();
    assert_eq!(token2.kind, TokenKind::Equal);
    assert_eq!(token2.lexeme, "=");

    let token3 = lexer.next().unwrap();
    assert_eq!(token3.kind, TokenKind::IntLiteral);
    assert_eq!(token3.lexeme, "42");

    let token4 = lexer.next().unwrap();
    assert_eq!(token4.kind, TokenKind::EndOfFile);
}

#[test]
fn test_indentation() {
    let source = "def test():\n    x = 1\n    if True:\n        y = 2\n    z = 3\n";
    let lexer = setup_lexer(source);
    let mut tokens = Vec::new();

    for token in lexer {
        if token.kind == TokenKind::EndOfFile {
            break;
        }

        tokens.push(token);
    }

    // Check that we have the right number of INDENT/DEDENT tokens
    let indent_count = tokens.iter().filter(|t| t.kind == TokenKind::Indent).count();
    let dedent_count = tokens.iter().filter(|t| t.kind == TokenKind::Dedent).count();

    assert_eq!(indent_count, 2); // One for the function body, one for the if block
    assert_eq!(dedent_count, 2); // Matching DEDENT tokens
}

#[test]
fn test_string_concatenation() {
    let source = "x = 'hello' 'world'";
    let mut lexer = setup_lexer(source);

    // Skip 'x' and '='
    let _ = lexer.next();
    let _ = lexer.next();

    let token = lexer.next().unwrap();
    assert_eq!(token.kind, TokenKind::StringLiteral);
    assert_eq!(token.lexeme, "'hello' 'world'");
}

#[test]
fn test_line_continuation() {
    let source = "x = (\n    1 + \n    2\n)";
    let lexer = setup_lexer(source);
    let mut tokens = Vec::new();

    for token in lexer {
        if token.kind == TokenKind::EndOfFile {
            break;
        }

        tokens.push(token.kind);
    }

    // Check that we don't have any NEWLINE tokens inside the parentheses
    let newlines = tokens.iter().filter(|&&k| k == TokenKind::Newline).count();
    assert_eq!(newlines, 0);
}

#[test]
fn test_soft_keywords() {
    // Test that 'match', 'case', 'type', and '_' are recognized as soft keywords
    let source = "match case type _";
    let mut lexer = setup_lexer(source);

    // Check 'match' token
    let token1 = lexer.next().unwrap();
    assert_eq!(token1.kind, TokenKind::Match);
    assert_eq!(token1.lexeme, "match");

    // Check 'case' token
    let token2 = lexer.next().unwrap();
    assert_eq!(token2.kind, TokenKind::Case);
    assert_eq!(token2.lexeme, "case");

    // Check 'type' token
    let token3 = lexer.next().unwrap();
    assert_eq!(token3.kind, TokenKind::Type);
    assert_eq!(token3.lexeme, "type");

    // Check '_' token
    let token4 = lexer.next().unwrap();
    assert_eq!(token4.kind, TokenKind::Underscore);
    assert_eq!(token4.lexeme, "_");
}

#[test]
fn test_soft_keywords_as_identifiers() {
    // Test that soft keywords can be used as identifiers in valid contexts
    let source = "x = match; y = case; z = type; w = _";
    let mut lexer = setup_lexer(source);

    // Skip 'x' and '='
    let _ = lexer.next();
    let _ = lexer.next();

    // Check 'match' token - should be a Match token
    let token1 = lexer.next().unwrap();
    assert_eq!(token1.kind, TokenKind::Match);

    // Skip ';' and 'y' and '='
    let _ = lexer.next();
    let _ = lexer.next();
    let _ = lexer.next();

    // Check 'case' token - should be a Case token
    let token2 = lexer.next().unwrap();
    assert_eq!(token2.kind, TokenKind::Case);

    // Skip ';' and 'z' and '='
    let _ = lexer.next();
    let _ = lexer.next();
    let _ = lexer.next();

    // Check 'type' token - should be a Type token
    let token3 = lexer.next().unwrap();
    assert_eq!(token3.kind, TokenKind::Type);

    // Skip ';' and 'w' and '='
    let _ = lexer.next();
    let _ = lexer.next();
    let _ = lexer.next();

    // Check '_' token - should be an Underscore token
    let token4 = lexer.next().unwrap();
    assert_eq!(token4.kind, TokenKind::Underscore);
}

#[test]
fn test_template_string_interpolation() {
    // Test template string with interpolation
    let source = r#"t"Hello {name}!""#;
    let mut lexer = setup_lexer(source);

    // Check template string token
    let token1 = lexer.next().unwrap();
    assert_eq!(token1.kind, TokenKind::TmplStringLiteral);

    // The lexer itself doesn't parse the interpolation - that's handled by the parser
    // But we can check that it correctly identifies the template string token
    assert_eq!(token1.lexeme, r#"t"Hello {name}!""#);
}

#[test]
fn test_union_type_operator() {
    // Test union type operator in type context
    let source = "int | str | None";
    let mut lexer = setup_lexer(source);

    // Check 'int' token
    let token1 = lexer.next().unwrap();
    assert_eq!(token1.kind, TokenKind::Identifier);
    assert_eq!(token1.lexeme, "int");

    // Check '|' token
    let token2 = lexer.next().unwrap();
    assert_eq!(token2.kind, TokenKind::Pipe);
    assert_eq!(token2.lexeme, "|");

    // Check 'str' token
    let token3 = lexer.next().unwrap();
    assert_eq!(token3.kind, TokenKind::Identifier);
    assert_eq!(token3.lexeme, "str");

    // Check '|' token
    let token4 = lexer.next().unwrap();
    assert_eq!(token4.kind, TokenKind::Pipe);
    assert_eq!(token4.lexeme, "|");

    // Check 'None' token
    let token5 = lexer.next().unwrap();
    assert_eq!(token5.kind, TokenKind::None);
    assert_eq!(token5.lexeme, "None");
}
