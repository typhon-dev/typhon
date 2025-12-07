//! Special expression parsing
//!
//! This module handles parsing of:
//! - Await expressions: `await coro`
//! - Yield expressions: `yield value`, `yield from iterable`
//! - Call expressions (postfix): `func(args)`
//! - Attribute access (postfix): `obj.attr`
//! - Subscription (postfix): `obj[index]`, slices: `obj[start:stop:step]`

use typhon_ast::nodes::{
    AnyNode,
    ArgumentExpr,
    AttributeExpr,
    AwaitExpr,
    CallExpr,
    NodeID,
    NodeKind,
    SliceExpr,
    SubscriptionExpr,
    TupleExpr,
    YieldExpr,
    YieldFromExpr,
};
use typhon_source::types::Span;

use super::operators::infix_binding_power;
use crate::diagnostics::{ParseErrorBuilder, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Parse an attribute access expression with a given object expression
    ///
    /// This is used by the Pratt parser when we already have the object expression.
    /// Parses: `obj.attr`
    pub(crate) fn parse_attribute_expr_with_lhs(&mut self, object: NodeID) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(object)
            .map_or(self.current_token().span().start, |node| node.span.start);

        // Expect and consume '.'
        self.expect(TokenKind::Dot)?;

        // Parse the attribute name (must be an identifier or soft keyword)
        // Soft keywords (match, case, type, _) can be used as attribute names
        if !self.matches(&[
            TokenKind::Identifier,
            TokenKind::Match,
            TokenKind::Case,
            TokenKind::Type,
            TokenKind::Underscore,
        ]) {
            return Err(ParseErrorBuilder::new()
                .message("Expected identifier after '.'")
                .span(self.create_source_span(
                    self.current_token().span().start,
                    self.current_token().span().end,
                ))
                .build());
        }

        // Get the attribute name
        let attr_name = self.current_token().lexeme().to_string();
        let end = self.current_token().span().end;

        // Consume the identifier
        self.skip();

        // Create the attribute expression
        let span = Span::new(start, end);
        let attribute = AttributeExpr::new(object, attr_name, NodeID::new(0, 0), span);

        // Allocate the node using MemberExpression kind
        let node_id =
            self.alloc_node(NodeKind::Expression, AnyNode::AttributeExpr(attribute), span);

        // Update the parent pointer
        if let Some(object_node) = self.ast.get_node_mut(object) {
            object_node.parent = Some(node_id);
        }

        Ok(node_id)
    }

    /// Parse an await expression.
    ///
    /// Await expressions suspend execution of an async function until the awaited
    /// coroutine completes. Can only be used inside async functions.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// await_expr: "await" expression
    /// ```
    ///
    /// ## Examples
    ///
    /// Basic await:
    ///
    /// ```python
    /// await coro()
    /// ```
    ///
    /// Await with attribute access:
    ///
    /// ```python
    /// await asyncio.sleep(1)
    /// await client.fetch_data()
    /// ```
    ///
    /// Await in assignment:
    ///
    /// ```python
    /// result = await get_result()
    /// ```
    ///
    /// Chained await:
    ///
    /// ```python
    /// data = await (await get_connection()).fetch()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The awaited expression is invalid or missing
    pub(crate) fn parse_await_expr(&mut self) -> ParseResult<NodeID> {
        let start = self.current_token().span().start;
        self.skip(); // Consume 'await'

        // Parse the value to await
        let value = self.parse_expression()?;

        // Get the end position
        let end =
            self.ast.get_node(value).map_or(self.current_token().span().end, |node| node.span.end);

        let span = Span::new(start, end);
        let await_expr = AwaitExpr::new(value, NodeID::new(0, 0), span);

        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::AwaitExpr(await_expr), span);
        Ok(node_id)
    }

    /// Parse a function call expression with a given function expression.
    ///
    /// This is used by the Pratt parser when we already have the function expression
    /// (left-hand side). Handles both positional and keyword arguments, with support
    /// for trailing commas.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// call_expr: expression "(" [argument_list] ")"
    /// argument_list: argument ("," argument)* [","]
    /// argument: expression | identifier "=" expression
    /// ```
    ///
    /// ## Examples
    ///
    /// No arguments:
    ///
    /// ```python
    /// func()
    /// ```
    ///
    /// Positional arguments:
    ///
    /// ```python
    /// print("hello", "world")
    /// max(1, 2, 3)
    /// ```
    ///
    /// Keyword arguments:
    ///
    /// ```python
    /// open("file.txt", mode="r", encoding="utf-8")
    /// dict(name="Alice", age=30)
    /// ```
    ///
    /// Mixed arguments:
    ///
    /// ```python
    /// func(1, 2, key1="value", key2=42)
    /// ```
    ///
    /// Trailing comma:
    ///
    /// ```python
    /// func(
    ///     arg1,
    ///     arg2,
    /// )
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing closing parenthesis `)`
    /// - Keyword argument name is not an identifier
    /// - Positional argument appears after keyword argument
    /// - Argument expression is invalid
    pub(crate) fn parse_call_expr_with_lhs(&mut self, func: NodeID) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(func)
            .map_or(self.current_token().span().start, |node| node.span.start);

        // Expect and consume '('
        self.expect(TokenKind::LeftParen)?;

        // Parse arguments
        let mut args = Vec::new();
        let mut keywords = Vec::new();

        // Check if we have arguments (not an empty parameter list)
        if !self.check(TokenKind::RightParen) {
            // Parse the first argument
            let arg = self.parse_expression()?;
            let arg_node = self.ast.get_node_mut(arg).unwrap().clone();

            // Determine if it's a positional or keyword argument
            if self.check(TokenKind::Assign) {
                // It's a keyword argument
                self.skip(); // Consume '='
                let value = self.parse_expression()?;

                // Create a named argument
                let name = if let Some(node) = self.ast.get_node(arg) {
                    // If the argument is a variable, use its name
                    if let AnyNode::VariableExpr(var) = &node.data {
                        var.name.clone()
                    } else {
                        return Err(ParseErrorBuilder::new()
                            .message("Expected identifier for keyword argument")
                            .span(node.span.into())
                            .build());
                    }
                } else {
                    return Err(ParseErrorBuilder::new()
                        .message("Expected identifier for keyword argument")
                        .span(self.create_source_span(start, self.current_token().span().end))
                        .build());
                };

                // Create the argument
                let span = Span::new(
                    arg_node.span.start,
                    self.ast
                        .get_node(value)
                        .map_or_else(|| self.current_token().span().end, |node| node.span.end),
                );

                let argument = ArgumentExpr::new(name, value, NodeID::new(0, 0), span);
                let arg_id =
                    self.alloc_node(NodeKind::Expression, AnyNode::ArgumentExpr(argument), span);
                keywords.push(arg_id);
            } else {
                // It's a positional argument
                args.push(arg);
            }

            // Parse additional arguments
            while self.check(TokenKind::Comma) {
                self.skip(); // Consume ','

                // Check for trailing comma
                if self.check(TokenKind::RightParen) {
                    break;
                }

                // Parse the next argument
                let arg = self.parse_expression()?;

                // Determine if it's a positional or keyword argument
                if self.check(TokenKind::Assign) {
                    // It's a keyword argument
                    self.skip(); // Consume '='
                    let value = self.parse_expression()?;

                    // Create a named argument
                    let arg_node = self.ast.get_node(arg).ok_or_else(|| {
                        ParseErrorBuilder::new()
                            .message("Expected identifier for keyword argument")
                            .span(self.create_source_span(start, self.current_token().span().end))
                            .build()
                    })?;

                    // If the argument is a variable, use its name
                    let name = if let AnyNode::VariableExpr(var) = arg_node.data.clone() {
                        var.name.clone()
                    } else {
                        return Err(ParseErrorBuilder::new()
                            .message("Expected identifier for keyword argument")
                            .span(arg_node.span.into())
                            .build());
                    };

                    // Create the argument
                    let span = Span::new(
                        arg_node.span.start,
                        self.ast
                            .get_node(value)
                            .map_or_else(|| self.current_token().span().end, |node| node.span.end),
                    );

                    let argument = ArgumentExpr::new(name, value, NodeID::new(0, 0), span);
                    let arg_id = self.alloc_node(
                        NodeKind::Expression,
                        AnyNode::ArgumentExpr(argument),
                        span,
                    );
                    keywords.push(arg_id);
                } else {
                    // It's a positional argument
                    args.push(arg);
                }
            }
        }

        // Expect ')'
        self.expect(TokenKind::RightParen)?;
        let end = self.current_token().span().end;

        // Create the call expression
        let span = Span::new(start, end);
        let call = CallExpr::new(func, args, keywords, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::CallExpr(call), span);

        Ok(node_id)
    }

    /// Parse an expression but stop at slice/subscript delimiters (colons, commas, brackets)
    ///
    /// This is used when parsing inside subscripts to avoid consuming delimiters.
    /// Also stops at `if` keywords when in comprehension context to prevent ternary parsing.
    fn parse_expression_until_delimiter(&mut self) -> ParseResult<NodeID> {
        // Parse prefix expression
        let mut lhs = self.parse_prefix_expr()?;

        // Loop while current token is an infix/postfix operator
        loop {
            let op = self.current_token().kind();

            // Stop at delimiters used in subscripts/slices
            if matches!(op, TokenKind::Colon | TokenKind::Comma | TokenKind::RightBracket) {
                break;
            }

            // Get operator binding power - returns None if not an infix operator
            let Some((_left_bp, right_bp)) = infix_binding_power(*op) else { break };

            // Handle the infix operator based on its type
            lhs = match op {
                TokenKind::LeftParen => self.parse_call_expr_with_lhs(lhs)?,
                TokenKind::LeftBracket => self.parse_subscription_expr_with_lhs(lhs)?,
                TokenKind::Dot => self.parse_attribute_expr_with_lhs(lhs)?,
                // Only parse ternary if NOT in comprehension context
                TokenKind::If => {
                    if self.context_stack.in_comprehension() {
                        // Stop here - don't consume the 'if' token
                        break;
                    }
                    self.parse_ternary_expr(lhs, right_bp)?
                }
                _ => self.parse_binary_expr(lhs, right_bp)?,
            }
        }

        Ok(lhs)
    }

    /// Parse a slice starting from a colon (no start expression)
    ///
    /// Handles: `[:stop]` and `[:stop:step]`
    #[allow(clippy::similar_names)]
    fn parse_slice_from_colon(&mut self, start_pos: usize) -> ParseResult<NodeID> {
        // Consume the first colon
        self.skip();

        // Parse stop expression (or None if another colon or close bracket)
        let stop = if self.check(TokenKind::Colon) || self.check(TokenKind::RightBracket) {
            None
        } else {
            Some(self.parse_expression_until_delimiter()?)
        };

        // Check for step (second colon)
        let step = if self.check(TokenKind::Colon) {
            self.skip(); // Consume second colon

            // Parse step expression (or None if close bracket)
            if self.check(TokenKind::RightBracket) {
                None
            } else {
                Some(self.parse_expression_until_delimiter()?)
            }
        } else {
            None
        };

        // Create slice node
        let end = self.current_token().span().end;
        let span = Span::new(start_pos, end);
        let slice = SliceExpr::new(None, stop, step, NodeID::new(0, 0), span);
        Ok(self.alloc_node(NodeKind::Expression, AnyNode::SliceExpr(slice), span))
    }

    /// Parse a slice that has a start expression
    ///
    /// Handles: `[start:stop]` and `[start:stop:step]` and `[start:]`
    #[allow(clippy::similar_names)]
    fn parse_slice_with_start(
        &mut self,
        start_expr: NodeID,
        start_pos: usize,
    ) -> ParseResult<NodeID> {
        // Consume the colon
        self.skip();

        // Parse stop expression (or None if another colon or close bracket)
        let stop = if self.check(TokenKind::Colon) || self.check(TokenKind::RightBracket) {
            None
        } else {
            Some(self.parse_expression_until_delimiter()?)
        };

        // Check for step (second colon)
        let step = if self.check(TokenKind::Colon) {
            self.skip(); // Consume second colon

            // Parse step expression (or None if close bracket)
            if self.check(TokenKind::RightBracket) {
                None
            } else {
                Some(self.parse_expression_until_delimiter()?)
            }
        } else {
            None
        };

        // Create slice node
        let end = self.current_token().span().end;
        let span = Span::new(start_pos, end);
        let slice = SliceExpr::new(Some(start_expr), stop, step, NodeID::new(0, 0), span);
        Ok(self.alloc_node(NodeKind::Expression, AnyNode::SliceExpr(slice), span))
    }

    /// Parse a subscription expression with a given object expression
    ///
    /// This is used by the Pratt parser when we already have the object expression.
    ///
    /// Parses:
    /// - `obj[index]` - simple subscription
    /// - `obj[arg1, arg2, ...]` - multiple indices (for generic types)
    /// - `obj[start:stop]` - slice
    /// - `obj[start:stop:step]` - slice with step
    /// - `obj[:stop]`, `obj[start:]`, `obj[:]` - partial slices
    pub(crate) fn parse_subscription_expr_with_lhs(
        &mut self,
        object: NodeID,
    ) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(object)
            .map_or(self.current_token().span().start, |node| node.span.start);

        // Expect and consume '['
        self.expect(TokenKind::LeftBracket)?;

        // Check for immediate colon (empty start in slice like [:stop])
        let index = if self.check(TokenKind::Colon) {
            // This is a slice with no start: [:stop] or [:stop:step]
            self.parse_slice_from_colon(start)?
        } else {
            // Parse the first expression, stopping at colons
            let first_expr = self.parse_expression_until_delimiter()?;

            // Check what comes next
            if self.check(TokenKind::Colon) {
                // This is a slice: [start:stop] or [start:stop:step]
                self.parse_slice_with_start(first_expr, start)?
            } else if self.check(TokenKind::Comma) {
                // Multiple elements - create a tuple to hold them
                let mut elements = vec![first_expr];

                while self.check(TokenKind::Comma) {
                    self.skip(); // Consume ','

                    // Check for trailing comma
                    if self.check(TokenKind::RightBracket) {
                        break;
                    }

                    // Parse the next element
                    let element = self.parse_expression_until_delimiter()?;
                    elements.push(element);
                }

                // Create a tuple expression to hold all elements
                let tuple_end = self.current_token().span().end;
                let tuple_span = Span::new(
                    self.ast.get_node(first_expr).map_or(start, |node| node.span.start),
                    tuple_end,
                );
                let tuple = TupleExpr::new(elements, NodeID::new(0, 0), tuple_span);
                self.alloc_node(NodeKind::Expression, AnyNode::TupleExpr(tuple), tuple_span)
            } else {
                // Single element - simple subscription
                first_expr
            }
        };

        // Expect the closing bracket
        self.expect(TokenKind::RightBracket)?;

        let end = self.current_token().span().end;
        let span = Span::new(start, end);

        // Create the subscription expression
        let subscription = SubscriptionExpr::new(object, index, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id =
            self.alloc_node(NodeKind::Expression, AnyNode::SubscriptionExpr(subscription), span);

        // Update parent pointers
        if let Some(object_node) = self.ast.get_node_mut(object) {
            object_node.parent = Some(node_id);
        }

        if let Some(index_node) = self.ast.get_node_mut(index) {
            index_node.parent = Some(node_id);
        }

        Ok(node_id)
    }

    /// Parse a yield expression
    ///
    /// Handles both `yield` and `yield <expr>` forms
    ///
    /// ## Examples
    ///
    /// ```typhon
    /// yield
    /// yield x + y
    /// ```
    pub(crate) fn parse_yield_expr(&mut self) -> ParseResult<NodeID> {
        let start = self.current_token().span().start;
        self.skip(); // Consume 'yield'

        // Check if this is "yield from" expression
        if self.check(TokenKind::From) {
            return self.parse_yield_from_expr(start);
        }

        // Check if there's a value to yield
        let value = if [
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::RightParen,
            TokenKind::RightBracket,
            TokenKind::RightBrace,
            TokenKind::Colon,
            TokenKind::Comma,
        ]
        .contains(self.current_token().kind())
        {
            None // No value to yield
        } else {
            // Parse the value to yield
            Some(self.parse_expression()?)
        };

        let end = match &value {
            Some(expr_id) => self
                .ast
                .get_node(*expr_id)
                .map_or_else(|| self.current_token().span().end, |node| node.span.end),
            None => self.current_token().span().end,
        };

        let span = Span::new(start, end);
        let yield_expr = YieldExpr::new(value, NodeID::new(0, 0), span);

        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::YieldExpr(yield_expr), span);
        Ok(node_id)
    }

    /// Parse a yield from expression
    ///
    /// Handles `yield from <expr>` form
    ///
    /// ## Examples
    ///
    /// ```typhon
    /// yield from generator()
    /// yield from [1, 2, 3]
    /// ```
    pub(crate) fn parse_yield_from_expr(&mut self, start: usize) -> ParseResult<NodeID> {
        self.skip(); // Consume 'from'

        // Parse the expression to yield from
        let expr = self.parse_expression()?;

        // Get the end position
        let end = self
            .ast
            .get_node(expr)
            .map_or_else(|| self.current_token().span().end, |node| node.span.end);

        let span = Span::new(start, end);
        let yield_from = YieldFromExpr::new(expr, NodeID::new(0, 0), span);

        let node_id =
            self.alloc_node(NodeKind::Expression, AnyNode::YieldFromExpr(yield_from), span);
        Ok(node_id)
    }
}
