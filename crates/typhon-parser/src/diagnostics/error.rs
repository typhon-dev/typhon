//! Error types for the Typhon parser.
//!
//! This module defines the error types used by the Typhon parser, including:
//! - `DiagnosticLevel`: Severity level of diagnostic messages
//! - `LexError`: Errors that can occur during lexical analysis
//! - `ParserError`: Errors that can occur during parsing
//! - `Diagnostic`: A diagnostic message with source location

use std::{fmt, io};

use thiserror::Error;
use typhon_source::types::SourceSpan;

use crate::lexer::{Token, TokenKind};

/// Represents the severity level of a diagnostic message.
///
/// Used to categorize diagnostic messages by their severity, allowing
/// the compiler to present them appropriately to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    /// An error that prevents successful compilation
    Error,
    /// A warning about potential issues
    Warning,
    /// Informational message
    Info,
    /// Additional notes about other diagnostics
    Note,
}

impl DiagnosticLevel {
    /// Returns a string representation of the diagnostic level
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Note => "note",
        }
    }

    /// Returns the ANSI color code for the level
    #[must_use]
    pub const fn color_code(&self) -> &'static str {
        match self {
            Self::Error => "\x1b[31m",   // Red
            Self::Warning => "\x1b[33m", // Yellow
            Self::Info => "\x1b[36m",    // Cyan
            Self::Note => "\x1b[34m",    // Blue
        }
    }

    /// Returns the ANSI reset code
    #[must_use]
    pub const fn reset_code() -> &'static str { "\x1b[0m" }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.as_str()) }
}

/// Lexer error kind
#[derive(Clone, Copy, Debug)]
pub enum LexErrorKind {
    /// Expected indentation but found something else
    ExpectedIndentation,
    /// Indentation is inconsistent
    InconsistentIndentation,
    /// Invalid character
    InvalidCharacter(char),
    /// Docstring does not have a proper ending
    InvalidDocStringEnding,
    /// Invalid escape character in a string literal
    InvalidEscapeChar(char),
    /// Hexadecimal number literal is malformed
    InvalidHexNumber,
    /// Invalid indentation
    InvalidIndentation { expected: usize, found: usize },
    /// Number literal is malformed
    InvalidNumber,
    /// String literal does not have a closing quote
    InvalidStringEnding,
    /// Invalid token found
    InvalidToken(char),
    /// Invalid Unicode escape sequence
    InvalidUnicodeEscape,
    /// Tab character found in indentation
    TabInIndentation,
    /// Unexpected end of file
    UnexpectedEOF,
}

/// Lexer error type
#[derive(Debug, Error, Clone)]
pub enum LexError {
    /// Invalid indentation
    #[error(
        "Invalid indentation at line {line}, column {column}: expected {expected}, found {found}"
    )]
    InvalidIndentation {
        /// Line number
        line: usize,
        /// Column number
        column: usize,
        /// Expected indentation
        expected: usize,
        /// Found indentation
        found: usize,
    },
    /// Invalid character
    #[error("Invalid character '{character}' at line {line}, column {column}")]
    InvalidCharacter {
        /// Invalid character
        character: char,
        /// Line number
        line: usize,
        /// Column number
        column: usize,
    },
    /// Invalid token
    #[error("Invalid token '{character}' at line {line}, column {column}")]
    InvalidToken {
        /// Invalid character
        character: char,
        /// Line number
        line: usize,
        /// Column number
        column: usize,
    },
    /// Indentation error
    #[error("Indentation error: {message}")]
    IndentationError {
        /// Error message
        message: String,
        /// Expected indentation level
        expected: usize,
        /// Found indentation level
        found: usize,
    },
    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,
    /// Invalid syntax
    #[error("Invalid syntax: {message}")]
    InvalidSyntax {
        /// Error message
        message: String,
        /// Span of the error
        span: SourceSpan,
    },
    /// Other error
    #[error("{0}")]
    Other(String),
}

impl LexError {
    /// Creates a new invalid indentation error.
    #[must_use]
    pub const fn invalid_indentation(
        line: usize,
        column: usize,
        expected: usize,
        found: usize,
    ) -> Self {
        Self::InvalidIndentation { line, column, expected, found }
    }

    /// Creates a new invalid character error.
    #[must_use]
    pub const fn invalid_character(line: usize, column: usize, character: char) -> Self {
        Self::InvalidCharacter { line, column, character }
    }

    /// Creates a new unexpected EOF error.
    #[must_use]
    pub const fn unexpected_eof() -> Self { Self::UnexpectedEof }

    /// Creates a new invalid syntax error.
    pub fn invalid_syntax(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::InvalidSyntax { message: message.into(), span }
    }

    /// Creates a new other error.
    pub fn other(message: impl Into<String>) -> Self { Self::Other(message.into()) }
}

/// Builder for lexer errors
#[derive(Clone, Copy, Debug)]
pub struct LexErrorBuilder {
    /// Line number
    line: Option<usize>,
    /// Column number
    column: Option<usize>,
    /// Error kind
    kind: Option<LexErrorKind>,
}

impl Default for LexErrorBuilder {
    fn default() -> Self { Self::new() }
}

impl LexErrorBuilder {
    /// Creates a new lexer error builder.
    #[must_use]
    pub const fn new() -> Self { Self { line: None, column: None, kind: None } }

    /// Sets the line number.
    #[must_use]
    pub const fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Sets the column number.
    #[must_use]
    pub const fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Sets the column number.
    #[must_use]
    pub const fn kind(mut self, kind: LexErrorKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Builds the lexer error.
    #[must_use]
    pub fn build(self) -> LexError {
        let line = self.line.unwrap_or(0);
        let column = self.column.unwrap_or(0);

        match self.kind {
            Some(LexErrorKind::InvalidIndentation { expected, found }) => {
                LexError::InvalidIndentation { line, column, expected, found }
            }
            Some(
                LexErrorKind::InvalidCharacter(character)
                | LexErrorKind::InvalidEscapeChar(character),
            ) => LexError::InvalidCharacter { line, column, character },
            Some(LexErrorKind::InvalidToken(character)) => {
                LexError::InvalidToken { line, column, character }
            }
            Some(LexErrorKind::UnexpectedEOF) => LexError::UnexpectedEof,
            Some(LexErrorKind::InvalidStringEnding) => {
                LexError::Other("Invalid string ending".to_string())
            }
            Some(LexErrorKind::InvalidNumber) => {
                LexError::Other("Invalid number literal".to_string())
            }
            Some(LexErrorKind::InvalidHexNumber) => {
                LexError::Other("Invalid hexadecimal literal".to_string())
            }
            Some(LexErrorKind::InconsistentIndentation) => LexError::IndentationError {
                message: "Inconsistent indentation".to_string(),
                expected: 0,
                found: 0,
            },
            Some(LexErrorKind::InvalidDocStringEnding) => {
                LexError::Other("Invalid docstring ending".to_string())
            }
            Some(LexErrorKind::ExpectedIndentation) => LexError::IndentationError {
                message: "Expected indentation".to_string(),
                expected: 0,
                found: 0,
            },
            Some(LexErrorKind::TabInIndentation) => {
                LexError::Other("Tab character in indentation".to_string())
            }
            Some(LexErrorKind::InvalidUnicodeEscape) => {
                LexError::Other("Invalid Unicode escape sequence".to_string())
            }
            None => LexError::Other("Unknown lexer error".to_string()),
        }
    }
}

/// Parser error type
#[derive(Clone, Debug, Error)]
pub enum ParseError {
    /// Unexpected token encountered
    #[error("Unexpected token '{token_kind}' at {}:{} (expected one of: {:?})", .span.start.line, .span.start.column, .expected)]
    UnexpectedToken {
        /// Expected token kinds
        expected: Vec<TokenKind>,
        /// Span of the error
        span: SourceSpan,
        /// Token kind for display purposes
        token_kind: String,
    },
    /// Unexpected end of file
    #[error("Unexpected end of file at {span}")]
    UnexpectedEof {
        /// Expected token kinds
        expected: Vec<TokenKind>,
        /// Span of the error
        span: SourceSpan,
    },
    /// Invalid syntax
    #[error("Invalid syntax: {message}")]
    InvalidSyntax {
        /// Error message
        message: String,
        /// Span of the error
        span: SourceSpan,
    },
    /// Indentation error
    #[error("Indentation error: {message}")]
    IndentationError {
        /// Error message
        message: String,
        /// Span of the error
        span: SourceSpan,
    },
    /// Error during lexical analysis
    #[error("Lexical error: {message}")]
    LexicalError {
        /// Description of the lexical error
        message: String,
        /// Location of the error
        span: SourceSpan,
    },
    /// Invalid literal value
    #[error("Invalid literal: {message}")]
    InvalidLiteral {
        /// Description of the literal error
        message: String,
        /// Location of the error
        span: SourceSpan,
    },
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Helper functions for creating parse errors
impl ParseError {
    /// Creates a new unexpected token error.
    #[must_use]
    pub fn unexpected_token(
        token_kind: TokenKind,
        expected: Vec<TokenKind>,
        span: SourceSpan,
    ) -> Self {
        let token_kind = format!("{token_kind}");

        Self::UnexpectedToken { expected, span, token_kind }
    }

    /// Creates a new unexpected EOF error.
    #[must_use]
    pub const fn unexpected_eof(expected: Vec<TokenKind>, span: SourceSpan) -> Self {
        Self::UnexpectedEof { expected, span }
    }

    /// Creates a new invalid syntax error.
    pub fn invalid_syntax(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::InvalidSyntax { message: message.into(), span }
    }

    /// Creates a new indentation error.
    pub fn indentation_error(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::IndentationError { message: message.into(), span }
    }

    /// Creates a new lexical error.
    pub fn lexical_error(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::LexicalError { message: message.into(), span }
    }

    /// Creates a new invalid literal error.
    pub fn invalid_literal(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::InvalidLiteral { message: message.into(), span }
    }

    /// Creates a new other error.
    pub fn other(message: impl Into<String>) -> Self { Self::Other(message.into()) }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self { Self::Other(err.to_string()) }
}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Builder for parser errors
#[derive(Clone, Debug)]
pub struct ParseErrorBuilder {
    /// Token that was encountered
    token: Option<Token<'static>>,
    /// Expected token kinds
    expected: Vec<TokenKind>,
    /// Error message
    message: Option<String>,
    /// Span of the error
    span: Option<SourceSpan>,
    /// Line number
    line: Option<usize>,
    /// Column number
    column: Option<usize>,
}

impl Default for ParseErrorBuilder {
    fn default() -> Self { Self::new() }
}

impl ParseErrorBuilder {
    /// Creates a new parse error builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            token: None,
            expected: Vec::new(),
            message: None,
            span: None,
            line: None,
            column: None,
        }
    }

    /// Sets the token that was encountered.
    #[must_use]
    pub const fn token(mut self, token: Token<'static>) -> Self {
        // Create a new static token with owned string
        let token_static: Token<'static> = token;
        self.token = Some(token_static);
        self
    }

    /// Adds an expected token kind.
    #[must_use]
    pub fn expected(mut self, kind: TokenKind) -> Self {
        self.expected.push(kind);
        self
    }

    /// Adds multiple expected token kinds.
    #[must_use]
    pub fn expected_kinds(mut self, kinds: Vec<TokenKind>) -> Self {
        self.expected.extend(kinds);
        self
    }

    /// Sets the error message.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Sets the span of the error.
    #[must_use]
    pub const fn span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Sets the line number.
    #[must_use]
    pub const fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Sets the column number.
    #[must_use]
    pub const fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Builds the parse error.
    #[must_use]
    pub fn build(self) -> ParseError {
        if let Some(token) = self.token {
            if let Some(span) = self.span {
                let token_kind = format!("{}", token.kind);
                ParseError::UnexpectedToken { expected: self.expected, span, token_kind }
            } else {
                // Fall back to Other if we don't have a span
                ParseError::Other(format!("Unexpected token: {}", token.kind))
            }
        } else if let Some(message) = self.message {
            if let Some(span) = self.span {
                ParseError::InvalidSyntax { message, span }
            } else {
                ParseError::Other(message)
            }
        } else if let Some(span) = self.span {
            ParseError::UnexpectedEof { expected: self.expected, span }
        } else {
            ParseError::Other("Unknown parser error".to_string())
        }
    }
}

/// A diagnostic message with source location information.
///
/// Diagnostics represent issues found during parsing, type checking, or other
/// compiler stages. They include severity level, message, source location,
/// and optional notes and suggestions.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// The severity level of this diagnostic
    pub level: DiagnosticLevel,
    /// The message describing the issue
    pub message: String,
    /// Source location of the issue
    pub span: SourceSpan,
    /// Additional explanatory notes
    pub notes: Vec<String>,
    /// Suggested fixes or alternatives
    pub suggestions: Vec<String>,
    /// Optional error code (like E0001)
    pub code: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    #[must_use]
    pub const fn error(message: String, span: SourceSpan) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
            code: None,
        }
    }

    /// Create a new warning diagnostic
    #[must_use]
    pub const fn warning(message: String, span: SourceSpan) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
            code: None,
        }
    }

    /// Create a new info diagnostic
    #[must_use]
    pub const fn info(message: String, span: SourceSpan) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
            code: None,
        }
    }

    /// Create a new note diagnostic
    #[must_use]
    pub const fn note(message: String, span: SourceSpan) -> Self {
        Self {
            level: DiagnosticLevel::Note,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
            code: None,
        }
    }

    /// Add an explanatory note to this diagnostic
    #[must_use]
    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    /// Add multiple explanatory notes to this diagnostic
    #[must_use]
    pub fn with_notes(mut self, notes: Vec<String>) -> Self {
        self.notes.extend(notes);
        self
    }

    /// Add a suggested fix to this diagnostic
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add multiple suggested fixes to this diagnostic
    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Add an error code to this diagnostic
    #[must_use]
    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }
}

/// Convert `LexError` to Diagnostic
impl From<LexError> for Diagnostic {
    fn from(error: LexError) -> Self {
        match error {
            LexError::InvalidIndentation { expected, found, .. } => {
                let span = SourceSpan::default(); // Need to create an appropriate span from line, column
                Self::error(
                    format!("Invalid indentation: expected {expected}, found {found}"),
                    span,
                )
                .with_note("Python-style indentation is significant in Typhon".to_string())
            }
            LexError::InvalidCharacter { character, .. } => {
                let span = SourceSpan::default(); // Need to create an appropriate span from line, column
                Self::error(format!("Invalid character: '{character}'"), span)
            }
            LexError::InvalidToken { character, .. } => {
                let span = SourceSpan::default(); // Need to create an appropriate span from line, column
                Self::error(format!("Invalid token: '{character}'"), span)
            }
            LexError::IndentationError { message, .. } => {
                let span = SourceSpan::default();
                Self::error(message, span)
                    .with_note("Python-style indentation is significant in Typhon".to_string())
            }
            LexError::UnexpectedEof => {
                Self::error("Unexpected end of file".to_string(), SourceSpan::default())
            }
            LexError::InvalidSyntax { message, span } => {
                Self::error(format!("Invalid syntax: {message}"), span)
            }
            LexError::Other(message) => Self::error(message, SourceSpan::default()),
        }
    }
}

/// Convert `ParserError` to Diagnostic
impl From<ParseError> for Diagnostic {
    fn from(error: ParseError) -> Self {
        match error {
            ParseError::UnexpectedToken { expected, span, token_kind, .. } => {
                let expected_str = if expected.is_empty() {
                    "something else".to_string()
                } else {
                    expected.iter().map(|kind| format!("{kind}")).collect::<Vec<_>>().join(", ")
                };
                Self::error(format!("Unexpected token {token_kind}, expected {expected_str}"), span)
                    .with_suggestion(format!("Try using one of: {expected_str}"))
            }
            ParseError::UnexpectedEof { expected, span } => {
                let expected_str = if expected.is_empty() {
                    "more input".to_string()
                } else {
                    expected.iter().map(|kind| format!("{kind}")).collect::<Vec<_>>().join(", ")
                };
                Self::error(format!("Unexpected end of file, expected {expected_str}"), span)
            }
            ParseError::InvalidSyntax { message, span } => {
                Self::error(format!("Invalid syntax: {message}"), span)
            }
            ParseError::IndentationError { message, span } => {
                Self::error(format!("Indentation error: {message}"), span)
                    .with_note("Python-style indentation is significant in Typhon".to_string())
            }
            ParseError::LexicalError { message, span } => {
                Self::error(format!("Lexical error: {message}"), span)
            }
            ParseError::InvalidLiteral { message, span } => {
                Self::error(format!("Invalid literal: {message}"), span)
            }
            ParseError::Other(message) => Self::error(message, SourceSpan::default()),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let color = self.level.color_code();
        let reset = DiagnosticLevel::reset_code();

        write!(f, "{}{}{}: {}", color, self.level, reset, self.message)?;

        if let Some(code) = &self.code {
            write!(f, " [{code}]")?;
        }

        write!(f, " at {}", self.span)?;

        for note in &self.notes {
            write!(f, "\n  note: {note}")?;
        }

        for suggestion in &self.suggestions {
            write!(f, "\n  suggestion: {suggestion}")?;
        }

        Ok(())
    }
}
