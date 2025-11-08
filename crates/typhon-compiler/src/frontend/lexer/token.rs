use std::fmt;

use logos::{
    Logos,
    Span,
};

/// A token type with source location information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The span of the token in the source code
    pub span: TokenSpan,
}

/// A span in the source code
pub type TokenSpan = Span;

/// Represents a token in the Typhon programming language
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(skip r"[ \t\f]+")] // Skip regular whitespace but not newlines
pub enum TokenKind {
    // Keywords
    #[token("and")]
    And,
    #[token("as")]
    As,
    #[token("assert")]
    Assert,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("break")]
    Break,
    #[token("class")]
    Class,
    #[token("continue")]
    Continue,
    #[token("def")]
    Def,
    #[token("del")]
    Del,
    #[token("elif")]
    Elif,
    #[token("else")]
    Else,
    #[token("except")]
    Except,
    #[token("finally")]
    Finally,
    #[token("for")]
    For,
    #[token("from")]
    From,
    #[token("global")]
    Global,
    #[token("if")]
    If,
    #[token("import")]
    Import,
    #[token("in")]
    In,
    #[token("is")]
    Is,
    #[token("lambda")]
    Lambda,
    #[token("nonlocal")]
    Nonlocal,
    #[token("not")]
    Not,
    #[token("or")]
    Or,
    #[token("pass")]
    Pass,
    #[token("raise")]
    Raise,
    #[token("return")]
    Return,
    #[token("try")]
    Try,
    #[token("while")]
    While,
    #[token("with")]
    With,
    #[token("yield")]
    Yield,

    // Typhon-specific keywords
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,

    // Type system keywords
    #[token("->")]
    Arrow,
    #[token("...")]
    Ellipsis,

    // Literals
    #[regex(r"[0-9][0-9_]*")]
    IntLiteral,
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*")]
    HexLiteral,
    #[regex(r"0[bB][01][01_]*")]
    BinLiteral,
    #[regex(r"0[oO][0-7][0-7_]*")]
    OctLiteral,
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?")]
    FloatLiteral,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    #[regex(r#"'([^'\\]|\\.)*'"#)]
    StringLiteral2,
    #[regex(r#"r"([^"\\]|\\.)*("|$)"#)]
    RawStringLiteral,
    #[regex(r#"r'([^'\\]|\\.)*('|$)"#)]
    RawStringLiteral2,
    #[regex(r#"b"([^"\\]|\\.)*""#)]
    BytesLiteral,
    #[regex(r#"b'([^'\\]|\\.)*'"#)]
    BytesLiteral2,

    // Triple-quoted string literals
    #[regex(r#""{3}([\s\S]*?)"{3}"#)]
    MultilineStringLiteral,
    #[regex(r#"'{3}([\s\S]*?)'{3}"#)]
    MultilineStringLiteral2,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("//")]
    DoubleSlash,
    #[token("%")]
    Percent,
    #[token("**")]
    DoubleStar,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("@")]
    At,

    // Comparisons
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,

    // Assignments
    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("//=")]
    DoubleSlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("**=")]
    DoubleStarAssign,
    #[token("<<=")]
    LeftShiftAssign,
    #[token(">>=")]
    RightShiftAssign,
    #[token("&=")]
    AmpersandAssign,
    #[token("|=")]
    PipeAssign,
    #[token("^=")]
    CaretAssign,
    #[token("@=")]
    AtAssign,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token(";")]
    Semicolon,

    // Whitespace and comments
    #[regex(r"\n")]
    Newline,
    #[regex(r"#[^\n]*")]
    Comment,

    // Indent/Dedent tokens (these are synthesized by the lexer)
    Indent,
    Dedent,

    // End of file
    Eof,

    // Error
    #[error]
    Error,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::And => write!(f, "and"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Assert => write!(f, "assert"),
            TokenKind::Async => write!(f, "async"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Def => write!(f, "def"),
            TokenKind::Del => write!(f, "del"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Except => write!(f, "except"),
            TokenKind::Finally => write!(f, "finally"),
            TokenKind::For => write!(f, "for"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Global => write!(f, "global"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Lambda => write!(f, "lambda"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Nonlocal => write!(f, "nonlocal"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Pass => write!(f, "pass"),
            TokenKind::Raise => write!(f, "raise"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::While => write!(f, "while"),
            TokenKind::With => write!(f, "with"),
            TokenKind::Yield => write!(f, "yield"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Ellipsis => write!(f, "..."),
            TokenKind::IntLiteral => write!(f, "<int>"),
            TokenKind::HexLiteral => write!(f, "<hex>"),
            TokenKind::BinLiteral => write!(f, "<bin>"),
            TokenKind::OctLiteral => write!(f, "<oct>"),
            TokenKind::FloatLiteral => write!(f, "<float>"),
            TokenKind::StringLiteral | TokenKind::StringLiteral2 => write!(f, "<string>"),
            TokenKind::RawStringLiteral | TokenKind::RawStringLiteral2 => write!(f, "<raw string>"),
            TokenKind::BytesLiteral | TokenKind::BytesLiteral2 => write!(f, "<bytes>"),
            TokenKind::MultilineStringLiteral | TokenKind::MultilineStringLiteral2 => {
                write!(f, "<multiline string>")
            }
            TokenKind::Identifier => write!(f, "<identifier>"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::DoubleSlash => write!(f, "//"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::DoubleStar => write!(f, "**"),
            TokenKind::LeftShift => write!(f, "<<"),
            TokenKind::RightShift => write!(f, ">>"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::At => write!(f, "@"),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Assign => write!(f, "="),
            TokenKind::PlusAssign => write!(f, "+="),
            TokenKind::MinusAssign => write!(f, "-="),
            TokenKind::StarAssign => write!(f, "*="),
            TokenKind::SlashAssign => write!(f, "/="),
            TokenKind::DoubleSlashAssign => write!(f, "//="),
            TokenKind::PercentAssign => write!(f, "%="),
            TokenKind::DoubleStarAssign => write!(f, "**="),
            TokenKind::LeftShiftAssign => write!(f, "<<="),
            TokenKind::RightShiftAssign => write!(f, ">>="),
            TokenKind::AmpersandAssign => write!(f, "&="),
            TokenKind::PipeAssign => write!(f, "|="),
            TokenKind::CaretAssign => write!(f, "^="),
            TokenKind::AtAssign => write!(f, "@="),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Newline => write!(f, "<newline>"),
            TokenKind::Comment => write!(f, "<comment>"),
            TokenKind::Indent => write!(f, "<indent>"),
            TokenKind::Dedent => write!(f, "<dedent>"),
            TokenKind::Eof => write!(f, "<eof>"),
            TokenKind::Error => write!(f, "<error>"),
        }
    }
}
