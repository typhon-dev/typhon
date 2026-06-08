//! Lexer error and warning types.
//!
//! This module defines the diagnostic types produced during lexical analysis:
//! - [`LexError`]: Errors that prevent successful tokenization.
//! - [`LexErrorKind`]: A coarse-grained classification of lexer errors used by [`LexErrorBuilder`].
//! - [`LexErrorBuilder`]: Fluent builder for constructing [`LexError`] values.
//! - [`LexWarning`]: Non-fatal lexer diagnostics (e.g. mixed tabs and spaces).
//!
//! These types are intentionally free of any dependency on the
//! [`crate::diagnostics`] module so the lexer can be extracted into its own
//! crate later without inducing a dependency cycle. Conversion into the
//! consumer-side [`Diagnostic`](crate::diagnostics::Diagnostic) type is
//! provided via `impl From<LexError> for Diagnostic` (and similarly for
//! [`LexWarning`]) in the diagnostics module.

use thiserror::Error;
use typhon_source::types::SourceSpan;

/// Lexer error kind.
///
/// Coarse-grained classification used by [`LexErrorBuilder`] to construct a
/// [`LexError`] without specifying line/column up-front.
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

/// Lexer error type.
#[derive(Clone, Debug, Error)]
pub enum LexError {
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
    /// Invalid syntax
    #[error("Invalid syntax: {message}")]
    InvalidSyntax {
        /// Error message
        message: String,
        /// Span of the error
        span: SourceSpan,
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
    /// Other error
    #[error("{0}")]
    Other(String),
    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,
}

impl LexError {
    /// Creates a new invalid character error.
    #[must_use]
    pub const fn invalid_character(line: usize, column: usize, character: char) -> Self {
        Self::InvalidCharacter { line, column, character }
    }

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

    /// Creates a new invalid syntax error.
    pub fn invalid_syntax(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::InvalidSyntax { message: message.into(), span }
    }

    /// Creates a new other error.
    pub fn other(message: impl Into<String>) -> Self { Self::Other(message.into()) }

    /// Creates a new unexpected EOF error.
    #[must_use]
    pub const fn unexpected_eof() -> Self { Self::UnexpectedEof }
}

/// Builder for [`LexError`] values.
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

    /// Sets the column number.
    #[must_use]
    pub const fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Sets the error kind.
    #[must_use]
    pub const fn kind(mut self, kind: LexErrorKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the line number.
    #[must_use]
    pub const fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

/// Non-fatal lexer diagnostic (warning level).
///
/// Lexer warnings are accumulated separately from errors so that consumers can
/// distinguish them when forwarding to a diagnostic reporter. Today the lexer
/// only produces a single warning kind (mixed tabs and spaces in indentation),
/// but the type is structured as an enum so additional warnings can be added
/// without changing the public API.
#[derive(Clone, Debug)]
pub enum LexWarning {
    /// A free-form warning message attached to a source span.
    Message {
        /// Human-readable warning message.
        message: String,
        /// Source span the warning refers to.
        span: SourceSpan,
    },
}

impl LexWarning {
    /// Creates a new free-form warning message at the given span.
    #[must_use]
    pub fn message(message: impl Into<String>, span: SourceSpan) -> Self {
        Self::Message { message: message.into(), span }
    }
}
