#[cfg(test)]
mod tests {
    use super::super::lexer::Lexer;
    use super::super::token::TokenKind;

    fn assert_tokens(source: &str, expected_tokens: Vec<TokenKind>) {
        let mut lexer = Lexer::new(source);
        let mut actual_tokens = Vec::new();

        while let Some(token) = lexer.next() {
            actual_tokens.push(token.kind);
        }

        assert_eq!(actual_tokens, expected_tokens);
    }

    #[test]
    fn test_keywords() {
        assert_tokens(
            "if else elif while for def class return pass break continue",
            vec![
                TokenKind::If,
                TokenKind::Else,
                TokenKind::Elif,
                TokenKind::While,
                TokenKind::For,
                TokenKind::Def,
                TokenKind::Class,
                TokenKind::Return,
                TokenKind::Pass,
                TokenKind::Break,
                TokenKind::Continue,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_operators() {
        assert_tokens(
            "+ - * / // % ** << >> & | ^ ~ @",
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::DoubleSlash,
                TokenKind::Percent,
                TokenKind::DoubleStar,
                TokenKind::LeftShift,
                TokenKind::RightShift,
                TokenKind::Ampersand,
                TokenKind::Pipe,
                TokenKind::Caret,
                TokenKind::Tilde,
                TokenKind::At,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_comparisons() {
        assert_tokens(
            "< > <= >= == !=",
            vec![
                TokenKind::LessThan,
                TokenKind::GreaterThan,
                TokenKind::LessEqual,
                TokenKind::GreaterEqual,
                TokenKind::Equal,
                TokenKind::NotEqual,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_assignments() {
        assert_tokens(
            "= += -= *= /= //= %= **= <<= >>= &= |= ^= @=",
            vec![
                TokenKind::Assign,
                TokenKind::PlusAssign,
                TokenKind::MinusAssign,
                TokenKind::StarAssign,
                TokenKind::SlashAssign,
                TokenKind::DoubleSlashAssign,
                TokenKind::PercentAssign,
                TokenKind::DoubleStarAssign,
                TokenKind::LeftShiftAssign,
                TokenKind::RightShiftAssign,
                TokenKind::AmpersandAssign,
                TokenKind::PipeAssign,
                TokenKind::CaretAssign,
                TokenKind::AtAssign,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_delimiters() {
        assert_tokens(
            "( ) [ ] { } , : . ;",
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBracket,
                TokenKind::RightBracket,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Comma,
                TokenKind::Colon,
                TokenKind::Dot,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_literals() {
        assert_tokens(
            "123 0x1A 0b101 3.14 \"hello\" 'world'",
            vec![
                TokenKind::IntLiteral,
                TokenKind::HexLiteral,
                TokenKind::BinLiteral,
                TokenKind::FloatLiteral,
                TokenKind::StringLiteral,
                TokenKind::StringLiteral2,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_identifiers() {
        assert_tokens(
            "foo bar_123 _private",
            vec![
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_comments() {
        assert_tokens(
            "# This is a comment\nfoo",
            vec![
                TokenKind::Comment,
                TokenKind::Newline,
                TokenKind::Identifier,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_indentation() {
        assert_tokens(
            "if True:\n    print(\"Hello\")\n",
            vec![
                TokenKind::If,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::StringLiteral,
                TokenKind::RightParen,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_nested_indentation() {
        assert_tokens(
            "if True:\n    if True:\n        print(\"Hello\")\n    print(\"World\")\n",
            vec![
                TokenKind::If,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::If,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::StringLiteral,
                TokenKind::RightParen,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::StringLiteral,
                TokenKind::RightParen,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_multiple_dedents() {
        assert_tokens(
            "if True:\n    if True:\n        print(\"Hello\")\nprint(\"World\")\n",
            vec![
                TokenKind::If,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::If,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::StringLiteral,
                TokenKind::RightParen,
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Dedent,
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::StringLiteral,
                TokenKind::RightParen,
                TokenKind::Newline,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn test_complex_source() {
        let source = r#"
def factorial(n):
    # Recursive factorial function
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

result = factorial(5)  # Should be 120
"#;

        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        while let Some(token) = lexer.next() {
            if token.kind != TokenKind::Eof {
                tokens.push(token.kind);
            }
        }

        assert!(tokens.len() > 0);
        assert!(tokens.contains(&TokenKind::Def));
        assert!(tokens.contains(&TokenKind::If));
        assert!(tokens.contains(&TokenKind::Else));
        assert!(tokens.contains(&TokenKind::Return));
    }
}
