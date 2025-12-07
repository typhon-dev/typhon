//! # Lexer module for the Typhon programming language.
//!
//! This module is responsible for converting source code into tokens.
//! It uses the `logos` crate for efficient tokenization and adds support for
//! Python-style indentation-based syntax.
//!
//! The lexer handles:
//! - Indentation tracking to generate INDENT/DEDENT tokens
//! - Line continuation handling with backslashes and inside brackets
//! - Implicit string concatenation
//! - Error reporting and recovery
//! - Tab vs. spaces warnings

mod rules;
mod token;

use std::collections::VecDeque;
use std::sync::Arc;

use logos::Lexer as LogosLexer;
pub use rules::*;
pub use token::*;
use typhon_source::types::{FileID, Position, SourceSpan, Span};

use crate::diagnostics::{DiagnosticReporter, LexError};

/// Custom lexer that handles Python's indentation rules
#[derive(Debug)]
pub struct Lexer<'src> {
    /// The inner logos lexer
    inner: LogosLexer<'src, TokenKind>,
    /// Source code
    source: &'src str,
    /// File identifier
    file_id: FileID,
    /// Diagnostic reporter for error reporting
    diagnostic_reporter: Arc<DiagnosticReporter>,
    /// Indentation management
    indent_stack: Vec<usize>,
    /// Queue of pending tokens to return
    pending_tokens: VecDeque<Token<'src>>,
    /// Whether currently at the beginning of the line
    at_line_start: bool,
    /// Whether currently in the context of a bracket expression
    in_brackets: usize,
    /// Current line
    line: usize,
    /// Current column
    column: usize,
    /// Current byte offset
    byte_offset: usize,
    /// Track the previous token for implicit string concatenation
    prev_token: Option<Token<'src>>,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given source
    #[must_use]
    pub fn new(
        source: &'src str,
        file_id: FileID,
        diagnostic_reporter: Arc<DiagnosticReporter>,
    ) -> Self {
        let inner = LogosLexer::new(source);

        Self {
            inner,
            source,
            file_id,
            diagnostic_reporter,
            indent_stack: vec![0], // Start with no indentation
            pending_tokens: VecDeque::new(),
            at_line_start: true,
            in_brackets: 0,
            line: 1,
            column: 1,
            byte_offset: 0,
            prev_token: None,
        }
    }

    /// Get the next token from the logos lexer
    fn next_logos_token(&mut self) -> Option<Token<'src>> {
        let result = self.inner.next();

        if let Some(kind) = result {
            let token_span = self.inner.span();
            let start_offset = token_span.start;
            let end_offset = token_span.end;

            // Update position info
            let start_line = self.line;
            let start_col = self.column;
            let lexeme = &self.source[start_offset..end_offset];

            // Update line/column information for the current token
            for c in lexeme.chars() {
                if c == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
            }

            // Create span for the token
            let start_pos = Position::new(start_line, start_col, start_offset);
            let end_pos = Position::new(self.line, self.column, end_offset);
            let source_span = SourceSpan::new(start_pos, end_pos, self.file_id);
            let span: Span = source_span.into();

            if let Ok(token_kind) = kind {
                // Check if this is a soft keyword
                let token_kind = if token_kind == TokenKind::Identifier {
                    // Check if this identifier is actually a soft keyword
                    check_soft_keyword(lexeme).unwrap_or(token_kind)
                } else {
                    token_kind
                };

                // Create token
                let token = Token::new(token_kind, lexeme, span.into());

                // Update byte offset
                self.byte_offset = end_offset;

                Some(token)
            } else {
                // Create a LexError for the invalid token
                let error = LexError::InvalidToken {
                    character: self.source[start_offset..end_offset].chars().next().unwrap_or('?'),
                    line: start_line,
                    column: start_col,
                };

                // Create a clone of the reporter for mutation
                let mut reporter_clone = (*self.diagnostic_reporter).clone();
                reporter_clone.add_diagnostic(error.into());
                self.diagnostic_reporter = Arc::new(reporter_clone);

                None
            }
        } else if self.indent_stack.len() > 1 {
            // End of file - generate necessary DEDENT tokens
            let _ = self.indent_stack.pop();
            let pos = Position::new(self.line, self.column, self.byte_offset);
            let source_span = SourceSpan::new(pos, pos, self.file_id);
            let span: Span = source_span.into();
            let token = Token::with_empty_lexeme(TokenKind::Dedent, span.into());

            Some(token)
        } else {
            None
        }
    }

    /// Returns the current source code being lexed
    #[must_use]
    pub const fn source(&self) -> &'src str { self.source }

    /// Returns the file ID
    #[must_use]
    pub const fn file_id(&self) -> FileID { self.file_id }

    /// Returns the diagnostic reporter
    #[must_use]
    pub const fn diagnostic_reporter(&self) -> &Arc<DiagnosticReporter> {
        &self.diagnostic_reporter
    }

    /// Returns the current line number
    #[must_use]
    pub const fn line(&self) -> usize { self.line }

    /// Returns the current column number
    #[must_use]
    pub const fn column(&self) -> usize { self.column }

    /// Returns the current byte offset
    #[must_use]
    pub const fn byte_offset(&self) -> usize { self.byte_offset }

    /// Returns whether the lexer is currently at the start of a line
    #[must_use]
    pub const fn is_at_line_start(&self) -> bool { self.at_line_start }

    /// Returns the current indentation stack
    #[must_use]
    pub fn indent_stack(&self) -> &[usize] { &self.indent_stack }

    /// Returns the number of open brackets
    #[must_use]
    pub const fn in_brackets(&self) -> usize { self.in_brackets }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return pending tokens first
        if let Some(token) = self.pending_tokens.pop_front() {
            self.prev_token = Some(token.clone());
            return Some(token);
        }

        // Handle indentation at line start
        if self.at_line_start && self.in_brackets == 0 {
            let mut space_count = 0;

            // Count spaces at the beginning of the line
            let mut chars = self.source[self.byte_offset..].chars();
            for c in chars.by_ref() {
                match c {
                    ' ' => {
                        space_count += 1;
                        self.byte_offset += 1;
                        self.column += 1;
                    }
                    '\t' => {
                        // Tab counts as 8 spaces in Python
                        space_count += 8;
                        self.byte_offset += 1;
                        self.column += 1;

                        // Report warning about mixing tabs and spaces
                        let pos = Position::new(self.line, self.column, self.byte_offset - 1);
                        let source_span = SourceSpan::new(pos, pos, self.file_id);

                        // Clone reporter, add diagnostic, and replace the original
                        let mut reporter_clone = (*self.diagnostic_reporter).clone();
                        let _ = reporter_clone.warning(
                            "Inconsistent indentation: mixing tabs and spaces".to_string(),
                            source_span,
                        );
                        self.diagnostic_reporter = Arc::new(reporter_clone);
                    }
                    _ => break,
                }
            }

            // If not just a blank line or comment, check indentation
            let is_blank_or_comment = match chars.next() {
                None | Some('\n' | '#') => true, // EOF, blank line, or comment
                _ => false,
            };

            if !is_blank_or_comment {
                // Compare with current indentation level
                let current_indent = *self.indent_stack.last().unwrap();

                // Compare current space_count with current_indent
                match space_count.cmp(&current_indent) {
                    // Indentation increased - push level and generate INDENT token
                    std::cmp::Ordering::Greater => {
                        self.indent_stack.push(space_count);

                        let pos = Position::new(self.line, 1, self.byte_offset - space_count);
                        let source_span = SourceSpan::new(pos, pos, self.file_id);
                        let span: Span = source_span.into();
                        let token = Token::with_empty_lexeme(TokenKind::Indent, span.into());
                        self.prev_token = Some(token.clone());

                        // Mark that we're no longer at line start
                        self.at_line_start = false;

                        return Some(token);
                    }
                    // Indentation decreased - pop levels and generate DEDENT tokens
                    std::cmp::Ordering::Less => {
                        let mut dedent_tokens = Vec::new();

                        while let Some(&level) = self.indent_stack.last() {
                            if space_count >= level {
                                break;
                            }

                            let _ = self.indent_stack.pop();

                            // Check if indentation matches any level in stack
                            if !self.indent_stack.is_empty()
                                && !self.indent_stack.contains(&space_count)
                            {
                                // Create error builder
                                let error = LexError::IndentationError {
                                    message: "Inconsistent indentation level".to_string(),
                                    expected: *self.indent_stack.last().unwrap(),
                                    found: space_count,
                                };

                                // Clone reporter, add diagnostic, and replace the original
                                let mut reporter_clone = (*self.diagnostic_reporter).clone();
                                reporter_clone.add_diagnostic(error.into());
                                self.diagnostic_reporter = Arc::new(reporter_clone);
                            }

                            let pos = Position::new(self.line, 1, self.byte_offset - space_count);
                            let source_span = SourceSpan::new(pos, pos, self.file_id);
                            let span: Span = source_span.into();
                            dedent_tokens
                                .push(Token::with_empty_lexeme(TokenKind::Dedent, span.into()));
                        }

                        // Queue the DEDENT tokens
                        if !dedent_tokens.is_empty() {
                            dedent_tokens.reverse();
                            self.pending_tokens.extend(dedent_tokens);
                            let token = self.pending_tokens.pop_front().unwrap();
                            self.prev_token = Some(token.clone());

                            // Mark that we're no longer at line start
                            self.at_line_start = false;

                            return Some(token);
                        }
                    }
                    // Same indentation level - no change
                    std::cmp::Ordering::Equal => {}
                }
            }

            self.at_line_start = false;
        }

        // Get the next token from the logos lexer
        let token_result = self.next_logos_token();

        if let Some(token) = &token_result {
            // Handle brackets (for tracking implicit line continuation)
            match token.kind {
                TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::LeftBrace => {
                    self.in_brackets += 1;
                }
                TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                    if self.in_brackets > 0 {
                        self.in_brackets -= 1;
                    }
                }
                TokenKind::Newline => {
                    // Skip newlines inside brackets (implicit line continuation)
                    if self.in_brackets > 0 {
                        return self.next();
                    }
                    self.at_line_start = true;
                }
                _ => {}
            }

            // Handle specialized contextual lexing for template strings
            if let Some(prev) = &self.prev_token {
                // For template string interpolation: {expr}
                // When we see a left brace inside a template string context,
                // we'll treat it as the start of an interpolation
                if is_in_template_string_context(prev.kind, self.in_brackets)
                    && token.kind == TokenKind::LeftBrace
                {
                    // We're in a template string interpolation context
                    // Continue processing normally, but with awareness of being in interpolation
                    // This will be used by the parser to know when to parse expressions in template strings
                }

                // For union type operator: Type1 | Type2
                // When we see a pipe in a type annotation context, it should be treated as a union operator
                // The type annotation context is detected by the parser, not here in the lexer
            }

            // Handle implicit string concatenation
            if let Some(prev) = &self.prev_token
                && is_string_literal(prev.kind)
                && is_string_literal(token.kind)
            {
                // Combine the two string literals and adjust span
                let combined_token = join_string_literals(prev, token, self.source);
                self.prev_token = Some(combined_token.clone());

                return Some(combined_token);
            }

            self.prev_token = token_result.clone();
        }

        token_result
    }
}
