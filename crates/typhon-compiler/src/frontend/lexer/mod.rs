// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/frontend/lexer/mod.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! Lexer implementation for the Typhon programming language.

use std::collections::VecDeque;
use std::ops::Range;

use logos::Logos;

use self::token::{
    Token,
    TokenKind,
};
use crate::common::Span;

pub mod token;

/// Lexer for the Typhon programming language
pub struct Lexer<'a> {
    /// The input source code
    source: &'a str,
    /// The logos lexer
    logos_lexer: logos::Lexer<'a, TokenKind>,
    /// Queue of tokens that have been produced but not yet consumed
    token_queue: VecDeque<Token>,
    /// Stack of indentation levels
    indent_stack: Vec<usize>,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed)
    column: usize,
    /// Flag indicating if we're at the beginning of a line
    at_line_start: bool,
    /// Flag indicating if we've reached the end of file
    reached_eof: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source code.
    pub fn new(source: &'a str) -> Self {
        let logos_lexer = TokenKind::lexer(source);
        let mut lexer = Self {
            source,
            logos_lexer,
            token_queue: VecDeque::new(),
            indent_stack: vec![0], // Start with 0 indentation
            line: 1,
            column: 1,
            at_line_start: true,
            reached_eof: false,
        };

        // Prime the lexer by reading the first tokens
        lexer.advance_tokens();

        lexer
    }

    /// Returns the next token without consuming it.
    pub fn peek(&mut self) -> Option<Token> {
        // Make sure we have at least one token in the queue
        if self.token_queue.is_empty() {
            self.advance_tokens();
        }

        self.token_queue.front().copied()
    }

    /// Returns and consumes the next token.
    pub fn next(&mut self) -> Option<Token> {
        // Make sure we have at least one token in the queue
        if self.token_queue.is_empty() {
            self.advance_tokens();
        }

        self.token_queue.pop_front()
    }

    /// Advances the token queue by reading more tokens from the source.
    fn advance_tokens(&mut self) {
        if self.reached_eof {
            return;
        }

        // Get the next token from the logos lexer
        let logos_token = self.logos_lexer.next();

        match logos_token {
            Some(token_kind) => {
                let span = self.logos_lexer.span();

                match token_kind {
                    Ok(TokenKind::Newline) => {
                        // If we're at a new line, check for indentation on the next line
                        self.handle_newline();
                    }
                    Ok(TokenKind::Comment) => {
                        // Skip comments and continue
                        self.advance_tokens();
                    }
                    _ => {
                        // Handle indentation at the start of a line
                        if self.at_line_start && token_kind != Ok(TokenKind::Newline) {
                            self.handle_indentation(span.start);
                        }

                        // For all other tokens, just add them to the queue
                        if token_kind != Ok(TokenKind::Newline) {
                            self.at_line_start = false;
                            self.token_queue.push_back(Token {
                                kind: token_kind.expect("Invalid token"),
                                span: span.into(),
                            });
                        }
                    }
                }
            }
            None => {
                // Handle EOF
                if !self.reached_eof {
                    self.reached_eof = true;

                    // Handle any remaining dedents
                    self.handle_eof();
                }
            }
        }
    }

    /// Handles a newline token by updating line/column information and setting the at_line_start flag.
    fn handle_newline(&mut self) {
        self.line += 1;
        self.column = 1;
        self.at_line_start = true;

        // Add a newline token to the queue
        let span = self.logos_lexer.span();
        self.token_queue.push_back(Token {
            kind: TokenKind::Newline,
            span: span.into(),
        });
    }

    /// Handles indentation at the start of a line.
    fn handle_indentation(&mut self, pos: usize) {
        // Calculate the indentation level
        let indentation = self.calculate_indentation(pos);
        let current = *self.indent_stack.last().unwrap();

        if indentation > current {
            // Indentation increased, emit an INDENT token
            self.indent_stack.push(indentation);
            self.token_queue.push_back(Token {
                kind: TokenKind::Indent,
                span: Span::new(0, 0), // Zero-width span at the start of the line
            });
        } else if indentation < current {
            // Indentation decreased, emit DEDENT tokens
            while !self.indent_stack.is_empty() && *self.indent_stack.last().unwrap() > indentation
            {
                self.indent_stack.pop();
                self.token_queue.push_back(Token {
                    kind: TokenKind::Dedent,
                    span: Span::new(0, 0), // Zero-width span at the start of the line
                });
            }

            // Make sure we have a matching indentation level
            if self.indent_stack.is_empty() || *self.indent_stack.last().unwrap() != indentation {
                // This is an indentation error, but we'll just emit an error token
                self.token_queue.push_back(Token {
                    kind: TokenKind::Error,
                    span: Span::new(0, 0), // Zero-width span at the start of the line
                });
                // Reset indentation to avoid further errors
                if self.indent_stack.is_empty() {
                    self.indent_stack.push(0);
                }
            }
        }
        // If indentation == current, no tokens need to be emitted
    }

    /// Calculates the indentation level at the given position.
    fn calculate_indentation(&self, pos: usize) -> usize {
        // Count the number of spaces at the current position
        let mut count = 0;
        let mut i = pos - self.column + 1; // Start at the beginning of the line

        while i < self.source.len() {
            match self.source.as_bytes()[i] {
                b' ' => count += 1,
                b'\t' => count += 8, // Assuming tabs are 8 spaces
                _ => break,
            }
            i += 1;
        }

        count
    }

    /// Handles the end of file by emitting any remaining dedent tokens.
    fn handle_eof(&mut self) {
        // Emit DEDENT tokens until we're back to the base level
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.token_queue.push_back(Token {
                kind: TokenKind::Dedent,
                span: self.logos_lexer.span().into(), // Use the last span
            });
        }

        // Emit an EOF token
        self.token_queue.push_back(Token {
            kind: TokenKind::Eof,
            span: self.logos_lexer.span().into(), // Use the last span
        });
    }

    /// Returns the current line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the current column number.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns a slice of the source code corresponding to the given span.
    pub fn slice(&self, span: Span) -> &'a str {
        &self.source[<Span as Into<Range<usize>>>::into(span)]
    }
}
