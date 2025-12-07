//! Container expression parsing (lists, dicts, sets, tuples)

use typhon_ast::nodes::NodeID;

use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Parse a list or list comprehension expression
    pub(crate) fn parse_list_or_list_comprehension(&mut self) -> ParseResult<NodeID> {
        // Consume '['
        let bracket = self.consume(TokenKind::LeftBracket)?;

        // Check if it's an empty list
        if self.expect(TokenKind::RightBracket).is_ok() {
            // Create an empty list
            return Ok(self.create_list_literal(
                Vec::new(),
                bracket.span().start,
                self.current_token().span().end,
            ));
        }

        // Not empty, parse the first element
        let first_expr = self.parse_expression()?;

        // Check if it's a list comprehension
        if self.check(TokenKind::For) {
            // It's a list comprehension
            return self.parse_list_comprehension(first_expr, bracket.span().start);
        }

        // Create the list with the first element
        let mut elements = Vec::new();
        elements.push(first_expr);

        // Parse additional elements
        while self.expect(TokenKind::Comma).is_ok() {
            // Check if we're at the end (trailing comma)
            if self.check(TokenKind::RightBracket) {
                break;
            }

            // Parse the next element
            elements.push(self.parse_expression()?);
        }

        // Expect ']'
        let closing_bracket = self.consume(TokenKind::RightBracket)?;

        // Create the list literal
        Ok(self.create_list_literal(elements, bracket.span().start, closing_bracket.span().end))
    }

    /// Parse a set or dictionary literal
    pub(crate) fn parse_set_or_dict_literal(&mut self) -> ParseResult<NodeID> {
        // Empty dictionary or set
        let start = self.current_token().span().start;
        self.skip(); // Consume '{'

        // Check if it's an empty dictionary or set
        if self.check(TokenKind::RightBrace) {
            self.skip(); // Consume '}'
            let end = self.current_token().span().end;

            // Determine if it's an empty set or an empty dictionary
            // Empty dictionaries are more common, so we'll default to that
            return Ok(self.create_dict_literal(Vec::new(), start, end));
        }

        // Not empty, parse the first key-value pair or set element
        let first_expr = self.parse_comprehension_condition()?;

        // Check if it's a dictionary or a set
        if self.check(TokenKind::Colon) {
            // It's a dictionary
            self.skip(); // Consume ':'

            // Parse the value
            let value = self.parse_expression()?;

            // Check for comprehension
            if self.check(TokenKind::For) {
                // It's a dictionary comprehension
                return self.parse_dict_comprehension(first_expr, value, start);
            }

            // Create the first entry
            let mut entries = Vec::new();
            entries.push((first_expr, value));

            // Parse additional key-value pairs
            while self.check(TokenKind::Comma) {
                self.skip(); // Consume ','

                // Check if we're at the end
                if self.check(TokenKind::RightBrace) {
                    break;
                }

                // Parse the next key
                let key = self.parse_expression()?;

                // Expect ':'
                self.expect(TokenKind::Colon)?;

                // Parse the value
                let value = self.parse_expression()?;

                // Add the entry
                entries.push((key, value));
            }

            // Expect '}'
            self.expect(TokenKind::RightBrace)?;
            let end = self.current_token().span().end;

            // Create the dictionary literal
            Ok(self.create_dict_literal(entries, start, end))
        } else {
            // It's a set
            // Check for comprehension
            if self.check(TokenKind::For) {
                // It's a set comprehension
                return self.parse_set_comprehension(first_expr, start);
            }

            // Create the set with the first element
            let mut elements = Vec::new();
            elements.push(first_expr);

            // Parse additional elements
            while self.check(TokenKind::Comma) {
                self.skip(); // Consume ','

                // Check if we're at the end (trailing comma)
                if self.check(TokenKind::RightBrace) {
                    break;
                }

                // Parse the next element
                let element = self.parse_expression()?;
                elements.push(element);
            }

            // Expect '}'
            self.expect(TokenKind::RightBrace)?;
            let end = self.current_token().span().end;

            // Create the set literal
            Ok(self.create_set_literal(elements, start, end))
        }
    }
}
