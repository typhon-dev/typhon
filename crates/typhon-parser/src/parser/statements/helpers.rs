//! Helper functions and utilities for statement parsing.

use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Handle indentation tokens (indent/dedent) to track Python's block structure
    pub fn handle_indentation(&mut self) -> ParseResult<()> {
        match self.current_token().kind {
            TokenKind::Indent => {
                // Process indent and update context
                self.skip();
                let indent_level = self.context_stack.current_indent_level() + 1;
                self.context_stack.current_mut().indent_level = indent_level;
                self.indent_stack.push(indent_level);
            }
            TokenKind::Dedent => {
                // Process dedent and update context
                self.skip();

                let _ = self.indent_stack.pop();
                let indent_level = match self.indent_stack.last() {
                    Some(&level) => level,
                    None => 0,
                };
                self.context_stack.current_mut().indent_level = indent_level;
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if the current token could be the start of an annotated assignment.
    ///
    /// An annotated assignment can have a complex target like `self.x: type = value`
    /// We check if we have an expression followed by `:`
    pub(super) fn is_annotated_assignment(&mut self) -> bool {
        // Look ahead to see if there's a colon after what looks like an expression
        // This is a bit tricky because we need to skip over potential attribute access
        // For now, check if current is identifier and we can find a colon within a few tokens
        if !self.check(TokenKind::Identifier) {
            return false;
        }

        // Look ahead for a pattern like: identifier (. identifier)* :
        let mut offset = 1;
        while let Some(token) = self.peek_nth(offset) {
            match token.kind {
                TokenKind::Dot | TokenKind::Identifier | TokenKind::Underscore => offset += 1,
                TokenKind::Colon => return true,
                _ => return false,
            }

            if offset > 10 {
                // Safety limit
                return false;
            }
        }

        false
    }

    /// Check if the current token could be the start of an assignment statement.
    ///
    /// Handles:
    /// - Simple assignment: `x = value`
    /// - Attribute assignment: `obj.attr = value`
    /// - Subscript assignment: `arr[0] = value` or `dict['key'] = value`
    /// - Tuple unpacking: `a, b = value`
    pub(super) fn is_assignment(&mut self) -> bool {
        // Check if we have an identifier (could be start of assignment target)
        if !self.check(TokenKind::Identifier) {
            return false;
        }

        // Look ahead to find '=' operator
        // Need to handle: `identifier = value`, `identifier.attr = value`, `a, b = value`, etc.
        let mut offset = 1;
        let mut bracket_depth = 0;

        while let Some(token) = self.peek_nth(offset) {
            match token.kind {
                TokenKind::Assign if bracket_depth == 0 => return true,
                TokenKind::LeftBracket => {
                    bracket_depth += 1;
                    offset += 1;
                }
                TokenKind::RightBracket => {
                    bracket_depth -= 1;
                    offset += 1;
                }
                // Continue looking through attribute access, subscripts, and tuple unpacking
                TokenKind::Dot | TokenKind::Identifier | TokenKind::Comma => {
                    offset += 1;
                }
                // Inside brackets, allow any tokens (for subscript expressions like dict['key'] or arr[0])
                _ if bracket_depth > 0 => {
                    offset += 1;
                }
                // Stop if we hit something that can't be part of an assignment target
                _ => return false,
            }

            if offset > 30 {
                // Safety limit to avoid infinite loops
                return false;
            }
        }

        false
    }

    /// Check if the current token could be the start of an augmented assignment statement.
    pub(super) fn is_augmented_assignment(&mut self) -> bool {
        // If we have at least one token ahead
        self.peek_nth(1).is_some_and(|peek_ahead| {
            matches!(
                peek_ahead.kind,
                TokenKind::PlusEqual
                    | TokenKind::MinusEqual
                    | TokenKind::StarEqual
                    | TokenKind::SlashEqual
                    | TokenKind::DoubleSlashEqual
                    | TokenKind::PercentEqual
                    | TokenKind::DoubleStarEqual
                    | TokenKind::LeftShiftEqual
                    | TokenKind::RightShiftEqual
                    | TokenKind::AmpersandEqual
                    | TokenKind::PipeEqual
                    | TokenKind::CaretEqual
                    | TokenKind::AtEqual
            )
        })
    }

    /// Check if the current token could be the start of a variable declaration.
    ///
    /// A variable declaration has the form: `identifier: type [= value]`
    /// This distinguishes it from a simple assignment (`identifier = value`)
    pub(super) fn is_variable_declaration(&self) -> bool {
        // If the current token is an identifier and the next token is ':'
        self.check(TokenKind::Identifier) && self.peek_token().kind == TokenKind::Colon
    }
}
