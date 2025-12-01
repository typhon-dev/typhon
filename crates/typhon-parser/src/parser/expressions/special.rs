//! Special expression parsing
//!
//! This module handles parsing of:
//! - Await expressions: `await coro`
//! - Yield expressions: `yield value`, `yield from iterable`
//! - Call expressions (postfix): `func(args)`
//! - Attribute access (postfix): `obj.attr`
//! - Subscription (postfix): `obj[index]`

use typhon_ast::nodes::{
    AnyNode,
    ArgumentExpr,
    AttributeExpr,
    AwaitExpr,
    CallExpr,
    NodeID,
    NodeKind,
    SubscriptionExpr,
    YieldExpr,
    YieldFromExpr,
};
use typhon_source::types::Span;

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

        // Parse the attribute name (must be an identifier)
        if !self.check(TokenKind::Identifier) {
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
        let _ = self.advance();

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

    /// Parse an await expression
    ///
    /// Handles `await <expr>` form
    ///
    /// ## Examples
    ///
    /// ```typhon
    /// await coro
    /// await asyncio.sleep(1)
    /// ```
    pub(crate) fn parse_await_expr(&mut self) -> ParseResult<NodeID> {
        let start = self.current_token().span().start;
        let _ = self.advance(); // Consume 'await'

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

    /// Parse a function call expression with a given function expression
    ///
    /// This is used by the Pratt parser when we already have the function expression.
    /// Parses: `func(arg1, arg2, key=value)`
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
            if self.check(TokenKind::Equal) {
                // It's a keyword argument
                let _ = self.advance(); // Consume '='
                let value = self.parse_expression()?;

                // Create a named argument
                let name = if let Some(node) = self.ast.get_node(arg) {
                    // If the argument is a variable, use its name
                    if let AnyNode::VariableIdent(var) = &node.data {
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
                let _ = self.advance(); // Consume ','

                // Check for trailing comma
                if self.check(TokenKind::RightParen) {
                    break;
                }

                // Parse the next argument
                let arg = self.parse_expression()?;

                // Determine if it's a positional or keyword argument
                if self.check(TokenKind::Equal) {
                    // It's a keyword argument
                    let _ = self.advance(); // Consume '='
                    let value = self.parse_expression()?;

                    // Create a named argument
                    let arg_node = self.ast.get_node(arg).ok_or_else(|| {
                        ParseErrorBuilder::new()
                            .message("Expected identifier for keyword argument")
                            .span(self.create_source_span(start, self.current_token().span().end))
                            .build()
                    })?;

                    // If the argument is a variable, use its name
                    let name = if let AnyNode::VariableIdent(var) = arg_node.data.clone() {
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

    /// Parse a subscription expression with a given object expression
    ///
    /// This is used by the Pratt parser when we already have the object expression.
    /// Parses: `obj[index]`
    ///
    /// TODO: Handle proper slice syntax (start:stop:step)
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

        // Parse the index expression
        // In Python, this can be a simple expression or a slice
        // For simplicity, we'll just parse it as a regular expression for now
        let index = self.parse_expression_bp(0)?;

        // Expect the closing bracket
        self.expect(TokenKind::RightBracket)?;

        let end = self.current_token().span().end;
        let span = Span::new(start, end);

        // Create the subscription expression
        let subscription = SubscriptionExpr::new(object, index, NodeID::new(0, 0), span);

        // Allocate the node using BinaryExpression kind
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
        let _ = self.advance(); // Consume 'yield'

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
        let _ = self.advance(); // Consume 'from'

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
