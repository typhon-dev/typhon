//! Core expression parsing logic using Pratt parsing
//!
//! This module contains the main Pratt parser implementation for handling
//! operator precedence and associativity correctly.

use typhon_ast::nodes::NodeID;

use super::operators::infix_binding_power;
use crate::diagnostics::{ParseErrorBuilder, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::{Context, ContextType, Parser};

impl Parser<'_> {
    /// Parse an expression for use as a comprehension condition (excludes ternary `if`).
    ///
    /// This is used in comprehension `if` clauses where we need to parse a filter condition
    /// but NOT allow ternary expressions (since there's no `else` in comprehension filters).
    /// The key difference from regular expression parsing is that we stop if we see `if`
    /// as an operator (to prevent parsing ternary expressions like `x if y else z`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// comprehension_condition: prefix_expr (infix_op_no_if comprehension_condition)*
    /// infix_op_no_if: binary_op | "(" | "[" | "."
    /// binary_op: "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "and" | "or" | ...
    /// ```
    ///
    /// Note: The `if` keyword is explicitly excluded from infix operators.
    ///
    /// ## Examples
    ///
    /// List comprehension with filter:
    ///
    /// ```python
    /// [x for x in range(10) if x > 5]
    /// #                       ^^^^^^^^ parsed by this method
    /// ```
    ///
    /// Complex filter conditions:
    ///
    /// ```python
    /// [x for x in items if x.active and x.count > 0]
    /// #                    ^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// Nested attribute access in filter:
    ///
    /// ```python
    /// [user for user in users if user.profile.verified]
    /// #                          ^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// Multiple comparisons:
    ///
    /// ```python
    /// [n for n in numbers if 0 < n <= 100]
    /// #                      ^^^^^^^^^^^^^
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The filter condition expression is invalid
    /// - A binary operator is missing its right-hand side
    /// - An unexpected token appears in the condition
    pub(crate) fn parse_comprehension_condition(&mut self) -> ParseResult<NodeID> {
        // Parse prefix expression
        let mut lhs = self.parse_prefix_expr()?;

        // Loop while current token is an infix/postfix operator
        // BUT exclude the `if` keyword since ternary expressions aren't allowed here
        loop {
            let op = self.current_token().kind();

            // Stop at the `if` keyword - no ternary expressions in comprehension filters
            if *op == TokenKind::If {
                break;
            }

            // Get operator binding power - returns None if not an infix operator
            let Some((_left_bp, right_bp)) = infix_binding_power(*op) else {
                break;
            };

            // Handle the infix operator based on its type
            lhs = match op {
                // Postfix: function call
                TokenKind::LeftParen => self.parse_call_expr_with_lhs(lhs)?,
                // Postfix: subscript
                TokenKind::LeftBracket => self.parse_subscription_expr_with_lhs(lhs)?,
                // Postfix: attribute access
                TokenKind::Dot => self.parse_attribute_expr_with_lhs(lhs)?,
                // Binary operators (includes 'not in', 'is not', etc.)
                // Use special version that recursively calls parse_comprehension_condition
                _ => self.parse_comprehension_binary_expr(lhs, right_bp)?,
            }
        }

        Ok(lhs)
    }

    /// Parse a comprehension condition with binding power (excludes ternary)
    ///
    /// Like `parse_expression_bp` but stops at 'if' keywords to prevent ternary parsing.
    pub(super) fn parse_comprehension_condition_bp(&mut self, min_bp: u8) -> ParseResult<NodeID> {
        // Parse prefix expression
        let mut lhs = self.parse_prefix_expr()?;

        // Loop while current token is an infix/postfix operator with sufficient binding power
        loop {
            let op = self.current_token().kind();

            // Stop at the `if` keyword - no ternary expressions allowed
            if *op == TokenKind::If {
                break;
            }

            // Get operator binding power - returns None if not an infix operator
            let Some((left_bp, right_bp)) = infix_binding_power(*op) else {
                break;
            };

            // If left binding power < min_bp, this operator binds too weakly
            if left_bp < min_bp {
                break;
            }

            // Handle the infix operator based on its type
            lhs = match op {
                // Postfix: function call
                TokenKind::LeftParen => self.parse_call_expr_with_lhs(lhs)?,
                // Postfix: subscript
                TokenKind::LeftBracket => self.parse_subscription_expr_with_lhs(lhs)?,
                // Postfix: attribute access
                TokenKind::Dot => self.parse_attribute_expr_with_lhs(lhs)?,
                // Binary operators
                _ => self.parse_comprehension_binary_expr(lhs, right_bp)?,
            }
        }

        Ok(lhs)
    }

    /// Parse an expression.
    ///
    /// This is the main entry point for expression parsing. It delegates to the
    /// Pratt parser ([`parse_expression_bp`](Self::parse_expression_bp)) with minimum
    /// binding power of 0, allowing all operators to be parsed with correct precedence.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// expression: prefix_expr (infix_op expression)*
    /// prefix_expr: literal | identifier | unary_op expression | "(" expression ")"
    ///            | "[" expression "]" | "{" expression "}" | lambda | await | yield
    /// infix_op: binary_op | "(" | "[" | "." | "if"
    /// binary_op: "+" | "-" | "*" | "/" | "%" | "**" | "==" | "!=" | "<" | ">"
    ///          | "<=" | ">=" | "and" | "or" | "in" | "is" | ...
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple literals:
    ///
    /// ```python
    /// 42
    /// "hello"
    /// True
    /// ```
    ///
    /// Binary operations:
    ///
    /// ```python
    /// x + y
    /// a * b + c
    /// count >= 10 and active
    /// ```
    ///
    /// Ternary conditional:
    ///
    /// ```python
    /// value if condition else default
    /// x if x > 0 else 0
    /// ```
    ///
    /// Function calls and attribute access:
    ///
    /// ```python
    /// func(arg1, arg2)
    /// obj.method()
    /// data[index]
    /// ```
    ///
    /// Complex expressions:
    ///
    /// ```python
    /// (x + y) * z
    /// [item for item in items if item.active]
    /// lambda x: x * 2
    /// await get_data()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Prefix expression parsing fails (invalid token, malformed literal, etc.)
    /// - Binary operator's right-hand side fails to parse
    /// - Ternary operator missing `else` keyword or has malformed condition/values
    /// - Postfix operators (call, subscript, attribute) fail to parse their arguments
    /// - Unexpected token encountered in expression context
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
    pub(super) fn parse_expression_bp(&mut self, min_bp: u8) -> ParseResult<NodeID> {
        // In type annotations, skip newlines (implicit line continuation)
        if self.context_stack.in_type_annotation() {
            self.skip_newlines();
        }

        // Parse prefix expression (literals, unary ops, grouping, etc.)
        let mut lhs = self.parse_prefix_expr()?;

        // Loop while current token is an infix/postfix operator with sufficient binding power
        loop {
            // In type annotations, skip newlines before checking for next operator
            if self.context_stack.in_type_annotation() {
                self.skip_newlines();
            }

            let op = self.current_token().kind();

            // If we're in a ForTarget context, stop at 'in' keyword
            if self.context_stack.in_for_target() && *op == TokenKind::In {
                break;
            }

            // Get operator binding power - returns None if not an infix operator
            let Some((left_bp, right_bp)) = infix_binding_power(*op) else {
                break;
            };

            // If left binding power < min_bp, this operator binds too weakly
            if left_bp < min_bp {
                break;
            }

            // Handle the infix operator based on its type
            lhs = match op {
                // Postfix: function call
                TokenKind::LeftParen => self.parse_call_expr_with_lhs(lhs)?,
                // Postfix: subscript
                TokenKind::LeftBracket => self.parse_subscription_expr_with_lhs(lhs)?,
                // Postfix: attribute access
                TokenKind::Dot => self.parse_attribute_expr_with_lhs(lhs)?,
                // Ternary: value if condition else other_value
                TokenKind::If => {
                    // Don't parse ternary in comprehension context (filters use 'if' without 'else')
                    if self.context_stack.in_comprehension() {
                        break;
                    }

                    self.parse_ternary_expr(lhs, right_bp)?
                }
                // Binary operators
                _ => self.parse_binary_expr(lhs, right_bp)?,
            }
        }

        Ok(lhs)
    }

    /// Parse an expression for use as a for-loop target (excludes `in` operator)
    ///
    /// This is used in for-loop statements where we need to parse the target variable(s)
    /// but stop before consuming the `in` keyword. For example, in `for i in range(10):`,
    /// this parses just `i` and leaves `in` for the for-statement parser to consume.
    ///
    /// Supports tuple unpacking: `for k, v in dict.items():`
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if the target expression is invalid.
    pub(crate) fn parse_for_target(&mut self) -> ParseResult<NodeID> {
        // Push ForTarget context to help nested parsing functions know we're in a for-target
        self.context_stack.push(Context::new(ContextType::ForTarget, None, 0));

        // Parse prefix expression (the target variable or tuple)
        let mut lhs = self.parse_prefix_expr()?;

        // Check for tuple unpacking (comma-separated targets)
        if self.check(TokenKind::Comma) {
            let result = self.parse_tuple_literal(lhs);
            drop(self.context_stack.pop()); // Pop ForTarget context

            return result;
        }

        // Loop while current token is an infix/postfix operator
        // BUT exclude the `in` keyword since it's part of for-loop syntax
        loop {
            let op = self.current_token().kind();

            // Stop at the `in` keyword - it's not part of the target expression
            if *op == TokenKind::In {
                break;
            }

            // Get operator binding power - returns None if not an infix operator
            let Some((_left_bp, right_bp)) = infix_binding_power(*op) else {
                break;
            };

            // Handle the infix operator based on its type
            lhs = match op {
                // Postfix: function call
                TokenKind::LeftParen => self.parse_call_expr_with_lhs(lhs)?,
                // Postfix: subscript
                TokenKind::LeftBracket => self.parse_subscription_expr_with_lhs(lhs)?,
                // Postfix: attribute access
                TokenKind::Dot => self.parse_attribute_expr_with_lhs(lhs)?,
                // Ternary: value if condition else other_value
                TokenKind::If => {
                    // Don't parse ternary in comprehension context (the 'if' starts a filter clause)
                    if self.context_stack.in_comprehension() {
                        break;
                    }

                    self.parse_ternary_expr(lhs, right_bp)?
                }
                // Binary operators
                _ => self.parse_binary_expr(lhs, right_bp)?,
            }
        }

        drop(self.context_stack.pop()); // Pop ForTarget context

        Ok(lhs)
    }

    /// Parse a prefix expression (literals, unary operators, grouping, etc.)
    pub(crate) fn parse_prefix_expr(&mut self) -> ParseResult<NodeID> {
        match self.current_token().kind() {
            // Literals
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::RawStringLiteral
            | TokenKind::MultilineStringLiteral
            | TokenKind::BytesLiteral
            | TokenKind::True
            | TokenKind::False
            | TokenKind::None
            | TokenKind::Ellipsis => self.parse_literal(),

            // Identifiers
            TokenKind::Identifier => self.parse_identifier_expr(),

            // Soft keywords that can be used as identifiers in expressions
            TokenKind::Type | TokenKind::Match | TokenKind::Case => {
                self.parse_soft_keyword_as_identifier()
            }

            // Unary operators: +, -, ~
            TokenKind::Plus | TokenKind::Minus | TokenKind::Tilde => self.parse_unary_arithmetic(),

            // Not operator (lower binding power - just above comparisons)
            TokenKind::Not => self.parse_unary_not(),

            // Grouping/tuples
            TokenKind::LeftParen => {
                let start = self.current_token().span().start;
                self.skip(); // Consume '('

                // Check for empty parentheses ()
                if self.check(TokenKind::RightParen) {
                    self.skip(); // Consume ')'
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
                self.skip();
                self.parse_starred_expr(false)
            }
            TokenKind::DoubleStar => {
                self.skip();
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
}
