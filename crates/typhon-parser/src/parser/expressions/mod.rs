//! Expression parsing module
//!
//! This module implements expression parsing using Pratt parsing for correct
//! operator precedence and associativity.

use typhon_ast::nodes::{AnyNode, NodeID, NodeKind, TernaryExpr, UnaryOpExpr, UnaryOpKind};
use typhon_source::types::Span;

use crate::diagnostics::{ParseErrorBuilder, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;

mod comprehensions;
mod containers;
mod helpers;
mod literals;
mod special;

impl Parser<'_> {
    /// Get the binding power for infix operators
    ///
    /// Returns (`left_binding_power`, `right_binding_power`) for operators.
    /// Left-associative operators have `right_bp = left_bp + 1`
    /// Right-associative operators have `right_bp = left_bp - 1`
    #[inline]
    pub(crate) const fn infix_binding_power(&self, op: &TokenKind) -> Option<(u8, u8)> {
        Some(match op {
            // Ternary (if-else) - right-associative
            TokenKind::If => (2, 1),

            // Logical OR
            TokenKind::Or => (3, 4),

            // Logical AND
            TokenKind::And => (5, 6),

            // Comparisons (can chain: a < b < c)
            TokenKind::Equal
            | TokenKind::NotEqual
            | TokenKind::LessThan
            | TokenKind::LessEqual
            | TokenKind::GreaterThan
            | TokenKind::GreaterEqual
            | TokenKind::In
            | TokenKind::Is => (7, 8),

            // Bitwise OR
            TokenKind::Pipe => (9, 10),

            // Bitwise XOR
            TokenKind::Caret => (11, 12),

            // Bitwise AND
            TokenKind::Ampersand => (13, 14),

            // Bit shifts
            TokenKind::LeftShift | TokenKind::RightShift => (15, 16),

            // Addition and subtraction
            TokenKind::Plus | TokenKind::Minus => (17, 18),

            // Multiplication, division, modulo
            TokenKind::Star | TokenKind::Slash | TokenKind::DoubleSlash | TokenKind::Percent => {
                (19, 20)
            }

            // Power - RIGHT-ASSOCIATIVE!
            TokenKind::DoubleStar => (22, 21),

            // Postfix operators (highest precedence)
            TokenKind::Dot | TokenKind::LeftParen | TokenKind::LeftBracket => (30, 31),

            _ => return None,
        })
    }

    /// Parse a binary operator expression
    #[inline]
    fn parse_binary_expr(&mut self, lhs: NodeID, right_bp: u8) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(lhs)
            .map_or(self.current_token().span().start, |node| node.span.start);

        // Convert token to binary operator kind and advance
        let bin_op = self.token_to_binary_op()?;

        // Parse right-hand side with right binding power
        let rhs = self.parse_expression_bp(right_bp)?;

        // Create binary operation node
        Ok(self.create_binary_op_node(bin_op, lhs, rhs, start))
    }

    /// Parse an expression
    ///
    /// This is the main entry point for expression parsing.
    /// It delegates to the Pratt parser for correct operator precedence.
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Prefix expression parsing fails (invalid token, malformed literal, etc.)
    /// - Binary operator's right-hand side fails to parse
    /// - Ternary operator missing 'else' keyword or malformed condition/values
    /// - Postfix operators (call, subscript, attribute) fail to parse their arguments
    pub fn parse_expression(&mut self) -> ParseResult<NodeID> {
        // Use the Pratt parser with minimum binding power of 0
        // This handles all operators with correct precedence and associativity
        self.parse_expression_bp(0)
    }

    /// Parse an expression with Pratt parsing (operator precedence)
    ///
    /// This is the core of the expression parser. It uses binding power
    /// to correctly handle operator precedence and associativity.
    ///
    /// ## Parameters
    ///
    /// - `min_bp`: Minimum binding power - operators with lower binding power will stop parsing
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Prefix expression parsing fails (invalid token, malformed literal, etc.)
    /// - Binary operator's right-hand side fails to parse
    /// - Ternary operator missing 'else' keyword or malformed condition/values
    /// - Postfix operators (call, subscript, attribute) fail to parse their arguments
    pub fn parse_expression_bp(&mut self, min_bp: u8) -> ParseResult<NodeID> {
        // Step 1: Parse prefix expression (literals, unary ops, grouping, etc.)
        let mut lhs = self.parse_prefix_expr()?;

        // Step 2: Loop while current token is an infix/postfix operator with sufficient binding power
        loop {
            let op = self.current_token().kind();

            // Get operator binding power - returns None if not an infix operator
            let Some((left_bp, right_bp)) = self.infix_binding_power(op) else { break };

            // If left binding power < min_bp, this operator binds too weakly
            if left_bp < min_bp {
                break;
            }

            // Step 3: Handle the infix operator based on its type
            match op {
                // Postfix: function call
                TokenKind::LeftParen => {
                    lhs = self.parse_call_expr_with_lhs(lhs)?;
                }

                // Postfix: subscript
                TokenKind::LeftBracket => {
                    lhs = self.parse_subscription_expr_with_lhs(lhs)?;
                }

                // Postfix: attribute access
                TokenKind::Dot => {
                    lhs = self.parse_attribute_expr_with_lhs(lhs)?;
                }

                // Ternary: value if condition else other_value
                TokenKind::If => {
                    lhs = self.parse_ternary_expr(lhs, right_bp)?;
                }

                // Binary operators
                _ => {
                    lhs = self.parse_binary_expr(lhs, right_bp)?;
                }
            }
        }

        Ok(lhs)
    }

    /// Parse a prefix expression (literals, unary operators, grouping, etc.)
    pub(crate) fn parse_prefix_expr(&mut self) -> ParseResult<NodeID> {
        match self.current_token().kind() {
            // Literals
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::MultilineStringLiteral
            | TokenKind::BytesLiteral
            | TokenKind::True
            | TokenKind::False
            | TokenKind::None => self.parse_literal(),

            // Identifiers
            TokenKind::Identifier => self.parse_identifier_expr(),

            // Unary operators: +, -, ~
            TokenKind::Plus | TokenKind::Minus | TokenKind::Tilde => {
                let start = self.current_token().span().start;
                let op = match self.current_token().kind() {
                    TokenKind::Plus => UnaryOpKind::Pos,
                    TokenKind::Minus => UnaryOpKind::Neg,
                    TokenKind::Tilde => UnaryOpKind::BitNot,
                    _ => unreachable!(),
                };
                let _ = self.advance();

                // Parse operand with high binding power (23)
                let operand = self.parse_expression_bp(23)?;

                let end = self
                    .ast
                    .get_node(operand)
                    .map_or(self.current_token().span().end, |node| node.span.end);

                let span = Span::new(start, end);
                let unary_op = UnaryOpExpr::new(op, operand, NodeID::new(0, 0), span);

                let node_id =
                    self.alloc_node(NodeKind::Expression, AnyNode::UnaryOpExpr(unary_op), span);

                // Update parent pointer
                if let Some(operand_node) = self.ast.get_node_mut(operand) {
                    operand_node.parent = Some(node_id);
                }

                Ok(node_id)
            }

            // Not operator (lower binding power - just above comparisons)
            TokenKind::Not => {
                // Check if this is "not in" (handled in infix)
                if self.peek_token().kind == TokenKind::In {
                    return Err(ParseErrorBuilder::new()
                        .message("Unexpected 'not' at start of expression (use parentheses for 'not in')")
                        .span(self.create_source_span(
                            self.current_token().span().start,
                            self.current_token().span().end))
                        .build());
                }

                let start = self.current_token().span().start;
                let _ = self.advance();

                // Parse operand with binding power just above comparisons (8)
                let operand = self.parse_expression_bp(8)?;

                let end = self
                    .ast
                    .get_node(operand)
                    .map_or(self.current_token().span().end, |node| node.span.end);

                let span = Span::new(start, end);
                let unary_op = UnaryOpExpr::new(UnaryOpKind::Not, operand, NodeID::new(0, 0), span);

                let node_id =
                    self.alloc_node(NodeKind::Expression, AnyNode::UnaryOpExpr(unary_op), span);

                // Update parent pointer
                if let Some(operand_node) = self.ast.get_node_mut(operand) {
                    operand_node.parent = Some(node_id);
                }

                Ok(node_id)
            }

            // Grouping/tuples
            TokenKind::LeftParen => {
                let start = self.current_token().span().start;
                let _ = self.advance(); // Consume '('

                // Check for empty parentheses ()
                if self.check(TokenKind::RightParen) {
                    let _ = self.advance(); // Consume ')'
                    // Empty tuple
                    return Ok(self.create_tuple_literal(vec![]));
                }

                // Parse the first expression
                let expr = self.parse_expression_bp(0)?;

                // Check if it's a generator expression
                if self.check(TokenKind::For) {
                    return self.parse_generator_expr(expr, start);
                }

                // Check for a tuple by looking for comma
                if self.check(TokenKind::Comma) {
                    return self.parse_tuple_literal(expr);
                }

                // It's a regular parenthesized expression
                self.expect(TokenKind::RightParen)?;

                // Create a grouping expression to preserve the parentheses in the AST
                Ok(self.create_grouping_expr(expr, start, self.current_token().span().end))
            }

            // Lists
            TokenKind::LeftBracket => self.parse_list_or_list_comprehension(),

            // Dicts and sets
            TokenKind::LeftBrace => self.parse_set_or_dict_literal(),

            // Lambda
            TokenKind::Lambda => self.parse_lambda_expr(),

            // Await
            TokenKind::Await => self.parse_await_expr(),

            // Yield
            TokenKind::Yield => self.parse_yield_expr(),

            // Starred expressions
            TokenKind::Star => {
                let _ = self.advance();
                self.parse_starred_expr(false)
            }
            TokenKind::DoubleStar => {
                let _ = self.advance();
                self.parse_starred_expr(true)
            }

            // F-strings and template strings
            TokenKind::FmtStringLiteral | TokenKind::MultilineFmtStringLiteral => {
                self.parse_fmt_string()
            }
            TokenKind::TmplStringLiteral | TokenKind::MultilineTmplStringLiteral => {
                self.parse_template_string()
            }

            _ => Err(ParseErrorBuilder::new()
                .message(format!(
                    "Unexpected token in expression: {:?}",
                    self.current_token().kind()
                ))
                .span(self.create_source_span(
                    self.current_token().span().start,
                    self.current_token().span().end,
                ))
                .build()),
        }
    }

    /// Parse a ternary conditional expression: `value if condition else other_value`
    #[inline]
    fn parse_ternary_expr(&mut self, value: NodeID, right_bp: u8) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(value)
            .map_or(self.current_token().span().start, |node| node.span.start);

        let _ = self.advance(); // consume 'if'

        // Parse condition with low precedence
        let condition = self.parse_expression_bp(0)?;

        // Expect 'else'
        self.expect(TokenKind::Else)?;

        // Parse else value with right binding power
        let else_value = self.parse_expression_bp(right_bp)?;

        // Get end position
        let end = self
            .ast
            .get_node(else_value)
            .map_or(self.current_token().span().end, |node| node.span.end);

        let span = Span::new(start, end);
        let ternary = TernaryExpr::new(value, condition, else_value, NodeID::new(0, 0), span);

        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::TernaryExpr(ternary), span);

        // Update parent pointers
        if let Some(value_node) = self.ast.get_node_mut(value) {
            value_node.parent = Some(node_id);
        }
        if let Some(cond_node) = self.ast.get_node_mut(condition) {
            cond_node.parent = Some(node_id);
        }
        if let Some(else_node) = self.ast.get_node_mut(else_value) {
            else_node.parent = Some(node_id);
        }

        Ok(node_id)
    }
}
