//! Token definitions for the Typhon programming language.
//!
//! This module defines the token types and structures used by the lexer.

use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use logos::Logos;

/// Represents a bracket type for tracking open/close brackets
/// Used for implicit line continuation
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BracketType {
    /// Opening bracket: `(`, `[`, or `{`
    Open,
    /// Closing bracket: `)`, `]`, or `}`
    Close,
}

/// Represents the type of token in the Typhon language.
///
/// This enum contains all token types recognized by the lexer, including:
///
/// - Keywords like `if`, `else`, `def`
/// - Literals like strings, numbers
/// - Operators and delimiters
/// - Special tokens for Python-style indentation
#[derive(Logos, Debug, Eq, PartialEq, Clone, Copy, Hash)]
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

    // Keyword literals
    #[token("True")]
    True,
    #[token("False")]
    False,
    #[token("None")]
    None,

    // Type system tokens
    #[token("->")]
    Arrow,
    #[token("...")]
    Ellipsis,

    // Walrus operator (assignment expression)
    #[token(":=")]
    ColonEqual,

    // Pattern matching
    #[token("!")]
    Exclamation,

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
    #[regex(r"[0-9]+j")]
    ImaginaryLiteral,

    #[regex(r#""([^\\"]|\\.)*""#)]
    #[regex(r#"'([^\\']|\\.)*'"#)]
    StringLiteral,

    #[regex(r#""{3}[^"]*"{3}"#)]
    #[regex(r#"'{3}[^']*'{3}"#)]
    MultilineStringLiteral,

    // Properly handle format strings, allowing for raw format strings
    #[regex(r#"f"([^\\"]|\\.)*""#, ignore(case))]
    #[regex(r#"f'([^\\']|\\.)*'"#, ignore(case))]
    #[regex(r#"(rf|fr)"([^"])*""#, ignore(case))]
    #[regex(r#"(rf|fr)'([^'])*'"#, ignore(case))]
    FmtStringLiteral,

    // Properly handle template strings, allowing for raw template strings
    #[regex(r#"t"([^\\"]|\\.)*""#, ignore(case))]
    #[regex(r#"t'([^\\']|\\.)*'"#, ignore(case))]
    #[regex(r#"(rt|tr)"([^"])*""#, ignore(case))]
    #[regex(r#"(rt|tr)'([^'])*'"#, ignore(case))]
    TmplStringLiteral,

    // Multiline format strings - allow quotes as long as not 3 in a row
    #[regex(r#"f"{3}(([^"]|"[^"]|""[^"])*)"{3}"#, ignore(case))]
    #[regex(r#"f'{3}(([^']|'[^']|''[^'])*)'{3}"#, ignore(case))]
    #[regex(r#"(rf|fr)"{3}(([^"]|"[^"]|""[^"])*)"{3}"#, ignore(case))]
    #[regex(r#"(rf|fr)'{3}(([^']|'[^']|''[^'])*)'{3}"#, ignore(case))]
    MultilineFmtStringLiteral,

    // Multiline template strings - allow quotes as long as not 3 in a row
    #[regex(r#"t"{3}(([^"]|"[^"]|""[^"])*)"{3}"#, ignore(case))]
    #[regex(r#"t'{3}(([^']|'[^']|''[^'])*)'{3}"#, ignore(case))]
    #[regex(r#"(rt|tr)"{3}(([^"]|"[^"]|""[^"])*)"{3}"#, ignore(case))]
    #[regex(r#"(rt|tr)'{3}(([^']|'[^']|''[^'])*)'{3}"#, ignore(case))]
    MultilineTmplStringLiteral,

    // Raw string literals
    #[regex(r#"r"([^"])*""#, ignore(case))]
    #[regex(r#"r'([^'])*'"#, ignore(case))]
    RawStringLiteral,

    // Bytes literals, with and without raw prefix
    #[regex(r#"b"([^\\"]|\\.)*""#, ignore(case))]
    #[regex(r#"b'([^\\']|\\.)*'"#, ignore(case))]
    #[regex(r#"(rb|br)"([^"])*""#, ignore(case))]
    #[regex(r#"(rb|br)'([^'])*'"#, ignore(case))]
    BytesLiteral,

    // Multiline bytes literals - allow quotes as long as not 3 in a row
    #[regex(r#"b"{3}(([^"]|"[^"]|""[^"])*)"{3}"#, ignore(case))]
    #[regex(r#"b'{3}(([^']|'[^']|''[^'])*)'{3}"#, ignore(case))]
    #[regex(r#"(rb|br)"{3}(([^"]|"[^"]|""[^"])*)"{3}"#, ignore(case))]
    #[regex(r#"(rb|br)'{3}(([^']|'[^']|''[^'])*)'{3}"#, ignore(case))]
    MultilineBytesLiteral,

    // Identifiers - single unified token type
    // Naming convention semantics (private, const, mangled, dunder)
    // are determined by the AST layer
    #[regex(r"_*[a-zA-Z][a-zA-Z0-9_]*")]
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
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("//=")]
    DoubleSlashEqual,
    #[token("%=")]
    PercentEqual,
    #[token("**=")]
    DoubleStarEqual,
    #[token("<<=")]
    LeftShiftEqual,
    #[token(">>=")]
    RightShiftEqual,
    #[token("&=")]
    AmpersandEqual,
    #[token("|=")]
    PipeEqual,
    #[token("^=")]
    CaretEqual,
    #[token("@=")]
    AtEqual,

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
    #[regex(r"#[^\n]*", logos::skip)]
    Comment,

    // Synthetic tokens
    Indent,
    Dedent,
    EndOfFile,

    // Pattern matching and soft keywords (only act as keywords in specific contexts)
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("type")]
    Type,
    #[token("_")]
    Underscore,

    SoftKeyword,

    // Error
    Error,
}

impl Display for TokenKind {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // Keywords
            Self::And => write!(f, "and"),
            Self::As => write!(f, "as"),
            Self::Assert => write!(f, "assert"),
            Self::Async => write!(f, "async"),
            Self::Await => write!(f, "await"),
            Self::Break => write!(f, "break"),
            Self::Class => write!(f, "class"),
            Self::Continue => write!(f, "continue"),
            Self::Def => write!(f, "def"),
            Self::Del => write!(f, "del"),
            Self::Elif => write!(f, "elif"),
            Self::Else => write!(f, "else"),
            Self::Except => write!(f, "except"),
            Self::Finally => write!(f, "finally"),
            Self::For => write!(f, "for"),
            Self::From => write!(f, "from"),
            Self::Global => write!(f, "global"),
            Self::If => write!(f, "if"),
            Self::Import => write!(f, "import"),
            Self::In => write!(f, "in"),
            Self::Is => write!(f, "is"),
            Self::Lambda => write!(f, "lambda"),
            Self::Nonlocal => write!(f, "nonlocal"),
            Self::Not => write!(f, "not"),
            Self::Or => write!(f, "or"),
            Self::Pass => write!(f, "pass"),
            Self::Raise => write!(f, "raise"),
            Self::Return => write!(f, "return"),
            Self::Try => write!(f, "try"),
            Self::While => write!(f, "while"),
            Self::With => write!(f, "with"),
            Self::Yield => write!(f, "yield"),

            // Keyword literals
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::None => write!(f, "None"),

            // Type system tokens
            Self::Arrow => write!(f, "->"),
            Self::Ellipsis => write!(f, "..."),
            Self::ColonEqual => write!(f, ":="),
            Self::Exclamation => write!(f, "!"),

            // Literals
            Self::IntLiteral => write!(f, "<int>"),
            Self::HexLiteral => write!(f, "<hex>"),
            Self::BinLiteral => write!(f, "<bin>"),
            Self::OctLiteral => write!(f, "<oct>"),
            Self::FloatLiteral => write!(f, "<float>"),
            Self::ImaginaryLiteral => write!(f, "<imaginary>"),
            Self::StringLiteral => write!(f, "<string>"),
            Self::FmtStringLiteral => write!(f, "<format string>"),
            Self::TmplStringLiteral => write!(f, "<template string>"),
            Self::RawStringLiteral => write!(f, "<raw string>"),
            Self::BytesLiteral => write!(f, "<bytes>"),
            Self::MultilineStringLiteral => write!(f, "<multiline string>"),
            Self::MultilineFmtStringLiteral => write!(f, "<multiline format string>"),
            Self::MultilineTmplStringLiteral => write!(f, "<multiline template string>"),
            Self::MultilineBytesLiteral => write!(f, "<multiline bytes>"),

            // Identifiers
            Self::Identifier => write!(f, "<identifier>"),

            // Operators
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::DoubleSlash => write!(f, "//"),
            Self::Percent => write!(f, "%"),
            Self::DoubleStar => write!(f, "**"),
            Self::LeftShift => write!(f, "<<"),
            Self::RightShift => write!(f, ">>"),
            Self::Ampersand => write!(f, "&"),
            Self::Pipe => write!(f, "|"),
            Self::Caret => write!(f, "^"),
            Self::Tilde => write!(f, "~"),
            Self::At => write!(f, "@"),

            // Comparisons
            Self::LessThan => write!(f, "<"),
            Self::GreaterThan => write!(f, ">"),
            Self::LessEqual => write!(f, "<="),
            Self::GreaterEqual => write!(f, ">="),
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),

            // Assignments
            Self::Assign => write!(f, "="),
            Self::PlusEqual => write!(f, "+="),
            Self::MinusEqual => write!(f, "-="),
            Self::StarEqual => write!(f, "*="),
            Self::SlashEqual => write!(f, "/="),
            Self::DoubleSlashEqual => write!(f, "//="),
            Self::PercentEqual => write!(f, "%="),
            Self::DoubleStarEqual => write!(f, "**="),
            Self::LeftShiftEqual => write!(f, "<<="),
            Self::RightShiftEqual => write!(f, ">>="),
            Self::AmpersandEqual => write!(f, "&="),
            Self::PipeEqual => write!(f, "|="),
            Self::CaretEqual => write!(f, "^="),
            Self::AtEqual => write!(f, "@="),

            // Delimiters
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::Dot => write!(f, "."),
            Self::Semicolon => write!(f, ";"),

            // Whitespace and comments
            Self::Newline => write!(f, "<newline>"),
            Self::Comment => write!(f, "<comment>"),

            // Synthetic tokens
            Self::Indent => write!(f, "<indent>"),
            Self::Dedent => write!(f, "<dedent>"),
            Self::EndOfFile => write!(f, "<end of file>"),
            Self::Match => write!(f, "match"),
            Self::Case => write!(f, "case"),
            Self::Type => write!(f, "type"),
            Self::Underscore => write!(f, "_"),
            Self::SoftKeyword => write!(f, "<soft keyword>"),

            // Error
            Self::Error => write!(f, "<error>"),
        }
    }
}
/// Represents a token in the Typhon language.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token<'src> {
    /// The kind of token.
    pub kind: TokenKind,
    /// The lexeme (the actual text of the token) from the source code.
    pub lexeme: &'src str,
    /// The span of the token in the source code.
    pub span: Range<usize>,
}

impl<'src> Token<'src> {
    /// Creates a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, lexeme: &'src str, span: Range<usize>) -> Self {
        Self { kind, lexeme, span }
    }

    /// Creates a token with an empty lexeme.
    #[must_use]
    pub const fn with_empty_lexeme(kind: TokenKind, span: Range<usize>) -> Self {
        Self { kind, span, lexeme: "" }
    }

    /// Returns the token kind.
    #[must_use]
    pub const fn kind(&self) -> &TokenKind { &self.kind }

    /// Returns the lexeme.
    #[must_use]
    pub const fn lexeme(&self) -> &'src str { self.lexeme }

    /// Returns the span.
    #[must_use]
    pub const fn span(&self) -> &Range<usize> { &self.span }

    /// Get the lexeme of the string literals token without surrounding quotes
    #[must_use]
    pub fn lexeme_unquote(&self) -> &str {
        let (mut start, end) = match self.kind {
            TokenKind::StringLiteral => (1, self.lexeme.len() - 1),
            TokenKind::MultilineStringLiteral => (3, self.lexeme.len() - 3),

            TokenKind::RawStringLiteral
            | TokenKind::BytesLiteral
            | TokenKind::FmtStringLiteral
            | TokenKind::TmplStringLiteral => (2, self.lexeme.len() - 1),

            TokenKind::MultilineFmtStringLiteral
            | TokenKind::MultilineBytesLiteral
            | TokenKind::MultilineTmplStringLiteral => (4, self.lexeme.len() - 3),

            _ => (0, self.lexeme.len()),
        };

        // Slice an extra character from lexeme start if contains multiple
        // string modifiers (e.g. `rf"raw format string: {value}"`)
        if [
            TokenKind::BytesLiteral,
            TokenKind::FmtStringLiteral,
            TokenKind::MultilineBytesLiteral,
            TokenKind::MultilineFmtStringLiteral,
            TokenKind::MultilineTmplStringLiteral,
            TokenKind::RawStringLiteral,
            TokenKind::TmplStringLiteral,
        ]
        .contains(&self.kind)
            && self.kind.to_string().get(..2).is_some_and(|prefix| {
                prefix.eq_ignore_ascii_case("rf")
                    || prefix.eq_ignore_ascii_case("fr")
                    || prefix.eq_ignore_ascii_case("rt")
                    || prefix.eq_ignore_ascii_case("tr")
            })
        {
            start += 1;
        }

        &self.lexeme[start..end]
    }

    /// Checks if the token is of the specified kind.
    #[must_use]
    pub fn is(&self, kind: TokenKind) -> bool { self.kind == kind }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}..{}", self.kind, self.span.start, self.span.end)
    }
}
