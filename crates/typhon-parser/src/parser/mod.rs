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
mod statements;
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
    /// Additional lookahead buffer for `peek_nth()`
    lookahead_buffer: Vec<Token<'src>>,
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
            lookahead_buffer: Vec::new(),
            diagnostics,
            indent_stack: vec![0],
            context_stack: ContextStack::new(),
        };

        // Initialize current and peek tokens
        parser.skip();
        parser.skip();

        parser
    }

    /// Advance to the next token and return the current token
    fn advance(&mut self) -> Token<'src> {
        // Save the current token
        let previous = std::mem::replace(&mut self.current, self.peek.clone());

        // Get the next token - either from buffer or lexer
        self.peek = if self.lookahead_buffer.is_empty() {
            match self.lexer.next() {
                Some(token) => token,
                None => Token::with_empty_lexeme(
                    TokenKind::EndOfFile,
                    self.source.len()..self.source.len(),
                ),
            }
        } else {
            self.lookahead_buffer.remove(0)
        };

        previous
    }

    /// Allocate an AST node
    pub fn alloc_node(&mut self, kind: NodeKind, data: AnyNode, span: Span) -> NodeID {
        self.ast.alloc_node(kind, data, span)
    }

    /// Get access to the AST arena
    #[inline]
    pub const fn ast(&self) -> &AST { &self.ast }

    /// Get mutable access to the AST arena
    #[inline]
    pub const fn ast_mut(&mut self) -> &mut AST { &mut self.ast }

    /// Check if the current token is of the specified kind
    #[inline]
    pub fn check(&self, kind: TokenKind) -> bool { self.current_token().kind() == &kind }

    /// Returns the lexer's current column number.
    pub const fn column(&self) -> usize { self.lexer.column() }

    /// Consume the current token if it matches the expected kind
    ///
    /// This is similar to [`expect()`](Self::expect) but returns the consumed token instead of `()`.
    /// Use this when you need access to the token's lexeme or span after consuming it.
    ///
    /// ## Errors
    ///
    /// Returns an error if the current token doesn't match the expected kind.
    pub fn consume(&mut self, kind: TokenKind) -> ParseResult<Token<'src>> {
        if self.check(kind) { Ok(self.advance()) } else { Err(self.unexpected_token(kind)) }
    }

    /// Look at the current token without consuming it
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

    #[inline]
    pub const fn current_token(&self) -> &Token<'src> { &self.current }

    /// Get the diagnostics reporter
    #[inline]
    pub const fn diagnostics(&self) -> &DiagnosticReporter { &self.diagnostics }

    /// Report an error with the given message
    pub fn error(&self, message: &str) -> ParseError {
        let span = self.create_source_span(self.current.span.start, self.current.span.end);

        ParseError::invalid_syntax(message, span)
    }

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

    /// Consume the current token if it matches the expected kind
    ///
    /// This is similar to [`consume()`](Self::consume) but doesn't return the token.
    /// Use this when you need to verify and skip a token without using its contents.
    ///
    /// ## Errors
    ///
    /// Returns an error if the current token doesn't match the expected kind.
    pub fn expect(&mut self, kind: TokenKind) -> ParseResult<()> {
        if self.check(kind) {
            self.skip();

            Ok(())
        } else {
            Err(self.unexpected_token(kind))
        }
    }

    /// Expect a statement end (newline or semicolon)
    ///
    /// ## Errors
    ///
    /// Returns an error if the current token is not a valid statement terminator.
    /// Valid terminators include: newline, semicolon, EOF, dedent, or the start of a new statement.
    pub fn expect_statement_end(&mut self) -> ParseResult<()> {
        // Check for explicit statement terminators that should be consumed
        if self.matches(&[TokenKind::Newline, TokenKind::Semicolon]) {
            self.skip();

            return Ok(());
        }

        // Check for implicit terminators that should NOT be consumed
        // These indicate the start of a new statement or end of a block
        if self.matches(&[
            TokenKind::Dedent,
            TokenKind::EndOfFile,
            TokenKind::Identifier, // Start of next statement (e.g., next variable declaration)
            TokenKind::Def,        // Start of method definition
            TokenKind::Class,      // Start of class definition
            TokenKind::Async,      // Start of async statement
            TokenKind::At,         // Start of decorated statement
        ]) {
            // Don't consume - let the next parse_statement() handle it
            return Ok(());
        }

        Err(self.error("Expected newline or semicolon after statement"))
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
                    self.skip();
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
        self.skip();
        None
    }

    /// Safely get the span of a node by its ID
    ///
    /// This is a helper method to replace `.get_node().unwrap()` patterns throughout the parser.
    /// It returns a `ParseError` if the node ID is invalid, which should never happen in correct code.
    ///
    /// ## Errors
    ///
    /// Returns a `ParseError` with an internal error message if the node ID is invalid.
    pub fn get_node_span(&self, node_id: NodeID) -> ParseResult<Span> {
        self.ast
            .get_node(node_id)
            .ok_or_else(|| {
                self.error(&format!("Internal error: invalid node ID {}", node_id.index()))
            })
            .map(|node| node.span)
    }

    /// Returns the lexer's current line number.
    pub const fn line(&self) -> usize { self.lexer.line() }

    /// Match the current token against a set of kinds
    #[inline]
    pub fn matches(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(self.current_token().kind())
    }

    /// Look ahead n tokens without consuming them
    ///
    /// Returns a reference to the token at position n ahead of the current token:
    /// - n=0 returns the current token
    /// - n=1 returns the peek token
    /// - n=2+ fetches additional tokens from the lexer and caches them
    ///
    /// The lookahead buffer is automatically maintained and cleared as tokens are consumed.
    pub fn peek_nth(&mut self, n: usize) -> Option<&Token<'src>> {
        match n {
            0 => Some(&self.current),
            1 => Some(&self.peek),
            _ => {
                // Calculate how many additional tokens we need
                let buffer_index = n - 2;

                // Fill the buffer up to the requested position
                while self.lookahead_buffer.len() <= buffer_index {
                    let token = match self.lexer.next() {
                        Some(t) => t,
                        None => Token::with_empty_lexeme(
                            TokenKind::EndOfFile,
                            self.source.len()..self.source.len(),
                        ),
                    };

                    self.lookahead_buffer.push(token);
                }

                self.lookahead_buffer.get(buffer_index)
            }
        }
    }

    /// Look at the next token without consuming it
    #[inline]
    pub const fn peek_token(&self) -> &Token<'src> { &self.peek }

    /// Report an error to the diagnostic reporter
    pub fn report_error(&mut self, error: ParseError) {
        // Add the error to the diagnostic reporter
        self.diagnostics.add_diagnostic(error.into());
    }

    /// Sets the parent of a node without returning the result
    ///
    /// This is a convenience wrapper around `ast.set_parent()` for cases
    /// where you don't need to check if the operation succeeded.
    #[inline]
    pub fn set_parent(&mut self, child: NodeID, parent: NodeID) {
        let _ = self.ast.set_parent(child, parent);
    }

    /// Skip the current token without returning it
    ///
    /// This is a convenience method for cases where you need to advance
    /// but don't care about the token being consumed.
    #[inline]
    fn skip(&mut self) { let _ = self.advance(); }

    /// Skip the current token without returning it
    /// if it matches one of the expected kinds
    ///
    /// This is a convenience method for cases where you need to conditionally
    /// advance and don't care about the token being consumed.
    #[inline]
    pub fn skip_if(&mut self, kinds: &[TokenKind]) {
        if self.matches(kinds) {
            self.skip();
        }
    }

    /// Skip over any consecutive newline tokens
    ///
    /// This is a utility method to consume blank lines in contexts where they
    /// should be ignored, such as:
    /// - Between the colon and Indent token when starting a block
    /// - Between statements at the same indentation level
    /// - After certain keywords that allow blank lines before their body
    ///
    /// This method does not affect the parser state beyond consuming Newline tokens.
    pub fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.skip();
        }
    }

    /// Skip over consecutive tokens that match any of the specified kinds
    ///
    /// This is a utility method to consume multiple consecutive tokens of specific types.
    /// It continues skipping as long as the current token matches any kind in the provided slice.
    ///
    /// Common use cases:
    /// - Skipping stray indent/dedent tokens during indentation transitions
    /// - Consuming multiple separator tokens
    /// - Cleaning up whitespace-related tokens
    ///
    /// This method does not affect the parser state beyond consuming matching tokens.
    pub fn skip_while(&mut self, kinds: &[TokenKind]) {
        while self.matches(kinds) {
            self.skip();
        }
    }

    /// Synchronizes the parser after encountering an error.
    ///
    /// This method skips tokens until it finds a token that could reasonably
    /// be the start of a new statement, like a newline, semicolon, or keyword.
    /// It's primarily used for coarse-grained error recovery at the statement level.
    pub fn synchronize(&mut self) {
        while !self.check(TokenKind::EndOfFile) {
            // Statements typically end with a newline or semicolon
            // Declarations and blocks can serve as synchronization points
            if self.matches(&[
                TokenKind::Class,
                TokenKind::Def,
                TokenKind::For,
                TokenKind::If,
                TokenKind::Newline,
                TokenKind::Return,
                TokenKind::Semicolon,
                TokenKind::While,
            ]) {
                self.skip();
            }
        }
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
}
