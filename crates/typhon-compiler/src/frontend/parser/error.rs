use thiserror::Error;

use crate::frontend::lexer::token::{
    Token,
    TokenKind,
    TokenSpan,
};

/// Parser error type
#[derive(Debug, Error)]
pub enum ParseError {
    /// Unexpected token encountered
    #[error("Unexpected token '{token_kind}' at line {line}, column {column}")]
    UnexpectedToken {
        /// Token that was encountered
        token: Token,
        /// Expected token kinds
        expected: Vec<TokenKind>,
        /// Line number
        line: usize,
        /// Column number
        column: usize,
        /// Token kind for display purposes
        token_kind: String,
    },

    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof {
        /// Expected token kinds
        expected: Vec<TokenKind>,
    },

    /// Invalid syntax
    #[error("Invalid syntax: {message}")]
    InvalidSyntax {
        /// Error message
        message: String,
        /// Span of the error
        span: TokenSpan,
    },

    /// Indentation error
    #[error("Indentation error: {message}")]
    IndentationError {
        /// Error message
        message: String,
        /// Span of the error
        span: TokenSpan,
    },

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Builder for parser errors
pub struct ParseErrorBuilder {
    /// Token that was encountered
    token: Option<Token>,
    /// Expected token kinds
    expected: Vec<TokenKind>,
    /// Error message
    message: Option<String>,
    /// Span of the error
    span: Option<TokenSpan>,
    /// Line number
    line: Option<usize>,
    /// Column number
    column: Option<usize>,
}

impl ParseErrorBuilder {
    /// Creates a new parse error builder.
    pub fn new() -> Self {
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
    pub fn token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    /// Adds an expected token kind.
    pub fn expected(mut self, kind: TokenKind) -> Self {
        self.expected.push(kind);
        self
    }

    /// Adds multiple expected token kinds.
    pub fn expected_kinds(mut self, kinds: Vec<TokenKind>) -> Self {
        self.expected.extend(kinds);
        self
    }

    /// Sets the error message.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Sets the span of the error.
    pub fn span(mut self, span: TokenSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Sets the line number.
    pub fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Sets the column number.
    pub fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Builds the parse error.
    pub fn build(self) -> ParseError {
        if let Some(token) = self.token {
            let token_kind = format!("{}", token.kind);
            ParseError::UnexpectedToken {
                token,
                expected: self.expected,
                line: self.line.unwrap_or(0),
                column: self.column.unwrap_or(0),
                token_kind,
            }
        } else if let Some(message) = self.message {
            if let Some(span) = self.span {
                ParseError::InvalidSyntax { message, span }
            } else {
                ParseError::Other(message)
            }
        } else {
            ParseError::UnexpectedEof {
                expected: self.expected,
            }
        }
    }
}

/// Helper functions for creating parse errors
impl ParseError {
    /// Creates a new unexpected token error.
    pub fn unexpected_token(
        token: Token,
        expected: Vec<TokenKind>,
        line: usize,
        column: usize,
    ) -> Self {
        let token_kind = format!("{}", token.kind);
        ParseError::UnexpectedToken {
            token,
            expected,
            line,
            column,
            token_kind,
        }
    }

    /// Creates a new unexpected EOF error.
    pub fn unexpected_eof(expected: Vec<TokenKind>) -> Self {
        ParseError::UnexpectedEof { expected }
    }

    /// Creates a new invalid syntax error.
    pub fn invalid_syntax(message: impl Into<String>, span: TokenSpan) -> Self {
        ParseError::InvalidSyntax {
            message: message.into(),
            span,
        }
    }

    /// Creates a new indentation error.
    pub fn indentation_error(message: impl Into<String>, span: TokenSpan) -> Self {
        ParseError::IndentationError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new other error.
    pub fn other(message: impl Into<String>) -> Self {
        ParseError::Other(message.into())
    }
}
