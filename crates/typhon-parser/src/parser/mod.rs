//! Parser module for the Typhon programming language.
//!
//! This module provides the core parsing functionality for converting token streams
//! into an Abstract Syntax Tree (AST). It uses an arena-based memory allocation strategy
//! for efficient node storage and traversal.
//!
//! The parser is responsible for:
//!
//! - Converting tokens from the lexer into an AST
//! - Reporting syntax errors with helpful diagnostics
//! - Managing memory efficiently using an arena allocator
//! - Providing a clean API for the compiler pipeline

mod context;
mod declaration;
mod expressions;
mod identifier;
mod module;
mod pattern;
mod statement;
mod types;

use std::sync::Arc;

pub use context::{
    Context,
    ContextFlags,
    ContextStack,
    ContextType,
    FunctionModifiers,
    IdentifierType,
    StringType,
};
use typhon_ast::ast::AST;
use typhon_ast::nodes::{AnyNode, NodeID, NodeKind};
use typhon_source::types::{FileID, Position, SourceManager, SourceSpan, Span};

use crate::diagnostics::{DiagnosticReporter, ParseError, ParseErrorBuilder, ParseResult};
use crate::lexer::{Lexer, Token, TokenKind};

/// The Parser struct is responsible for converting a stream of tokens
/// into an Abstract Syntax Tree (AST).
///
/// It uses an arena-based memory allocation strategy for efficient storage
/// and traversal of the AST nodes.
#[derive(Debug)]
pub struct Parser<'src> {
    /// Source code being parsed
    source: &'src str,
    /// File identifier
    file_id: FileID,
    /// Source manager for file information
    source_manager: Arc<SourceManager>,
    /// AST instance for node allocation
    ast: AST,
    /// Lexer providing tokens
    lexer: Lexer<'src>,
    /// Current token
    current: Token<'src>,
    /// Lookahead token
    peek: Token<'src>,
    /// Diagnostic reporter for error messages
    diagnostics: DiagnosticReporter,
    /// Track indentation for Python-style blocks
    indent_stack: Vec<usize>,
    /// Context stack for tracking parsing context
    pub context_stack: ContextStack,
}

impl<'src> Parser<'src> {
    /// Create a new parser for the given source code
    #[must_use]
    pub fn new(source: &'src str, file_id: FileID, source_manager: Arc<SourceManager>) -> Self {
        // Create diagnostic reporter
        let diagnostics = DiagnosticReporter::new(source_manager.clone());

        // Create lexer
        let lexer = Lexer::new(source, file_id, Arc::new(diagnostics.clone()));

        // Create default tokens to initialize current and peek
        let default_token = Token::with_empty_lexeme(TokenKind::Error, 0..0);

        // Create the parser
        let mut parser = Self {
            source,
            file_id,
            source_manager,
            ast: AST::new(),
            lexer,
            current: default_token.clone(),
            peek: default_token,
            diagnostics,
            indent_stack: vec![0],
            context_stack: ContextStack::new(),
        };

        // Initialize current and peek tokens
        let _ = parser.advance();
        let _ = parser.advance();

        parser
    }

    /// Advance to the next token and return the current token
    fn advance(&mut self) -> Token<'src> {
        // Save the current token
        let previous = std::mem::replace(&mut self.current, self.peek.clone());

        // Get the next token from the lexer
        self.peek = match self.lexer.next() {
            Some(token) => token,
            None => {
                Token::with_empty_lexeme(TokenKind::EndOfFile, self.source.len()..self.source.len())
            }
        };

        previous
    }

    /// Look at the current token without consuming it
    #[inline]
    pub const fn current_token(&self) -> &Token<'src> { &self.current }

    /// Look at the next token without consuming it
    #[inline]
    pub const fn peek_token(&self) -> &Token<'src> { &self.peek }

    /// Look ahead n tokens without consuming them
    pub const fn peek_n(&mut self, n: usize) -> Option<&Token<'src>> {
        if n == 0 {
            Some(&self.current)
        } else if n == 1 {
            Some(&self.peek)
        } else {
            // For deeper lookahead, we'd need a more complex implementation
            None
        }
    }

    /// Check if the current token is of the specified kind
    #[inline]
    pub fn check(&self, kind: TokenKind) -> bool { self.current_token().kind() == &kind }

    /// Match the current token against a set of kinds
    #[inline]
    pub fn matches(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(self.current_token().kind())
    }

    /// Consume the current token if it matches the expected kind
    ///
    /// ## Errors
    ///
    /// TODO: add context
    pub fn expect(&mut self, kind: TokenKind) -> ParseResult<()> {
        if self.check(kind) {
            let _ = self.advance();

            Ok(())
        } else {
            Err(self.unexpected_token(kind))
        }
    }

    /// Create a source span from start and end positions
    #[inline]
    pub fn create_source_span(&self, start: usize, end: usize) -> SourceSpan {
        // Use unwrap_or instead of unwrap_or_else to avoid closure allocation
        let start_pos = self
            .source_manager
            .position_from_offset(self.file_id, start)
            .unwrap_or_else(|| Position::new(1, 1, start));

        let end_pos = self
            .source_manager
            .position_from_offset(self.file_id, end)
            .unwrap_or_else(|| Position::new(1, 1, end));

        SourceSpan::new(start_pos, end_pos, self.file_id)
    }

    /// Convert a token's span to a source span
    #[inline]
    pub const fn token_to_span(&self, token: &Token<'src>) -> Span {
        Span::new(token.span.start, token.span.end)
    }

    /// Create an unexpected token error
    pub fn unexpected_token(&self, expected: TokenKind) -> ParseError {
        let span = self.create_source_span(self.current.span.start, self.current.span.end);

        // Create a token that doesn't have a lifetime tied to 'src
        let token_kind = self.current.kind;
        let token_span = self.current.span.clone();
        let token = Token::with_empty_lexeme(token_kind, token_span);

        // Create a builder and construct the error
        ParseErrorBuilder::new().token(token).expected(expected).span(span).build()
    }

    /// Report an error with the given message
    pub fn error(&self, message: &str) -> ParseError {
        let span = self.create_source_span(self.current.span.start, self.current.span.end);

        ParseError::invalid_syntax(message, span)
    }

    /// Report an error to the diagnostic reporter
    pub fn report_error(&mut self, error: ParseError) {
        // Add the error to the diagnostic reporter
        self.diagnostics.add_diagnostic(error.into());
    }

    /// Get access to the AST arena
    #[inline]
    pub const fn ast(&self) -> &AST { &self.ast }

    /// Get mutable access to the AST arena
    #[inline]
    pub const fn ast_mut(&mut self) -> &mut AST { &mut self.ast }

    /// Allocate an AST node
    pub fn alloc_node(&mut self, kind: NodeKind, data: AnyNode, span: Span) -> NodeID {
        self.ast.alloc_node(kind, data, span)
    }

    /// Get the diagnostics reporter
    #[inline]
    pub const fn diagnostics(&self) -> &DiagnosticReporter { &self.diagnostics }

    /// Create a diagnostic error with recovery information
    pub fn error_with_recovery(&self, message: &str, expected_tokens: &[TokenKind]) -> ParseError {
        // If there are expected tokens, include them in the error message
        if !expected_tokens.is_empty() {
            let expected_str =
                expected_tokens.iter().map(|t| format!("{t}")).collect::<Vec<_>>().join(", ");

            // Create enhanced error message including the expected tokens
            let enhanced_message = format!("{message} (expected one of: {expected_str})");
            return self.error(&enhanced_message);
        }

        // Otherwise, just return a regular error
        self.error(message)
    }

    /// Find the next valid expression token during error recovery
    ///
    /// This is a fine-grained error recovery mechanism specifically for expressions.
    /// It searches for any token in the provided list that could potentially continue
    /// parsing an expression. It has a limited lookahead to prevent excessive searching.
    ///
    /// Returns the token found (if any) to allow creating appropriate recovery nodes.
    pub fn find_next_valid_expression_token(
        &mut self,
        valid_tokens: &[TokenKind],
    ) -> Option<Token<'src>> {
        let mut lookahead = 0;
        let max_lookahead = 10; // Limit how far ahead we look to prevent infinite loops

        while lookahead < max_lookahead {
            if lookahead == 0 {
                if valid_tokens.contains(&self.current.kind) {
                    return Some(self.current.clone());
                }
            } else if lookahead == 1 {
                if valid_tokens.contains(&self.peek.kind) {
                    let _ = self.advance(); // Advance to the peek token
                    return Some(self.current.clone());
                }
            } else {
                // For deeper lookahead, we'd need to advance multiple times
                // which would disrupt the parsing state, so we just check current and peek
                break;
            }

            lookahead += 1;
        }

        // If we didn't find a sync token within the limit, just advance one token
        // to avoid getting stuck in an infinite loop
        let _ = self.advance();
        None
    }

    /// Skip until next statement boundary after encountering an error
    ///
    /// This method skips tokens until it finds a token that could reasonably
    /// be the start of a new statement, like a newline, semicolon, or keyword.
    /// It's primarily used for coarse-grained error recovery at the statement level.
    pub fn skip_to_statement_boundary(&mut self) {
        while self.current.kind != TokenKind::EndOfFile {
            // Statements typically end with a newline or semicolon
            if self.current.kind == TokenKind::Newline || self.current.kind == TokenKind::Semicolon
            {
                let _ = self.advance();

                return;
            }

            // Declarations and blocks can serve as synchronization points
            match self.current.kind {
                TokenKind::Class
                | TokenKind::Def
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return => return,
                _ => {
                    let _ = self.advance();
                }
            }
        }
    }

    /// Expect a statement end (newline or semicolon)
    ///
    /// ## Errors
    ///
    /// Returns an error if the current token is not a valid statement terminator.
    /// Valid terminators include: newline, semicolon, EOF, dedent, or the start of a new statement.
    pub fn expect_statement_end(&mut self) -> ParseResult<()> {
        // Check for explicit statement terminators
        if self.check(TokenKind::Newline) || self.check(TokenKind::Semicolon) {
            let _ = self.advance();

            return Ok(());
        }

        // Allow EOF, dedent, or the start of a new statement as implicit terminators
        // This handles cases where there's no explicit newline token available.
        // For example, after a statement ends at EOF, or when implicit line continuation
        // via brackets consumed the newlines.
        if self.matches(&[
            TokenKind::Async,
            TokenKind::Class,
            TokenKind::Dedent,
            TokenKind::Def,
            TokenKind::EndOfFile,
            TokenKind::From,
            TokenKind::Identifier,
            TokenKind::Import,
        ]) {
            return Ok(());
        }

        Err(self.error("Expected newline or semicolon after statement"))
    }
}
