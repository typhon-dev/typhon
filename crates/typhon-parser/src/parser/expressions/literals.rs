//! Literal expression parsing

use std::sync::Arc;

use typhon_ast::nodes::{
    AnyNode,
    FmtStringExpr,
    FmtStringPart,
    LambdaExpr,
    LiteralExpr,
    LiteralValue,
    NodeID,
    NodeKind,
    StarredExpr,
    TemplateStringExpr,
    TemplateStringPart,
    VariableIdent,
};
use typhon_source::types::Span;

use crate::diagnostics::{ParseErrorBuilder, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Parse an expression embedded in an f-string or template string
    fn parse_embedded_expression(&self, expr_str: &str) -> ParseResult<NodeID> {
        // Create a new parser instance for the embedded expression
        let mut expr_parser = Parser::new(expr_str, self.file_id, Arc::clone(&self.source_manager));

        // Parse the expression
        expr_parser.parse_expression()
    }

    /// Parse a formatted string (f-string) expression
    ///
    /// Handles f-strings of the form:
    /// - `f"Hello {name}!"`
    /// - `rf"Raw f-string: {path!r}"`
    pub(crate) fn parse_fmt_string(&mut self) -> ParseResult<NodeID> {
        // Record the start position and check token type
        let start = self.current_token().span().start;
        let end = self.current_token().span().end;

        // Determine if we're handling a regular or multiline f-string
        let is_multiline = match self.current_token().kind() {
            TokenKind::FmtStringLiteral => false,
            TokenKind::MultilineFmtStringLiteral => true,
            _ => {
                return Err(ParseErrorBuilder::new()
                    .message("Expected f-string")
                    .span(self.create_source_span(start, end))
                    .build());
            }
        };

        // Check if it's a raw f-string (starts with 'rf' or 'fr')
        let is_raw = self.current.lexeme.starts_with("rf")
            || self.current.lexeme.starts_with("fr")
            || self.current.lexeme.starts_with("RF")
            || self.current.lexeme.starts_with("FR");

        // Get the lexeme for processing
        let lexeme = if is_multiline {
            &self.current.lexeme[if is_raw { 3 } else { 2 }..self.current.lexeme.len() - 3]
        } else {
            &self.current.lexeme[if is_raw { 3 } else { 2 }..self.current.lexeme.len() - 1]
        };

        // Consume the f-string token
        let _ = self.advance();

        // Parse f-string parts
        let mut parts = Vec::new();

        // Find all expressions in the f-string (between { and })
        let mut start_idx = 0;
        let mut in_expr = false;
        let mut expr_start = 0;

        // Basic parser for f-string parts
        for (i, c) in lexeme.chars().enumerate() {
            if c == '{' && !in_expr {
                // End the current literal part if there is one
                if i > start_idx {
                    let literal = &lexeme[start_idx..i];
                    parts.push(FmtStringPart::Literal(literal.to_string()));
                }
                in_expr = true;
                expr_start = i + 1;
            } else if c == '}' && in_expr {
                // Parse the embedded expression
                let expr_str = &lexeme[expr_start..i];
                let expr_node = self.parse_embedded_expression(expr_str)?;

                parts.push(FmtStringPart::Expression(expr_node));

                in_expr = false;
                start_idx = i + 1;
            }
        }

        // Add any remaining literal text
        if start_idx < lexeme.len() && !in_expr {
            let literal = &lexeme[start_idx..];
            parts.push(FmtStringPart::Literal(literal.to_string()));
        }

        // Create the f-string node
        let span = Span::new(start, self.current_token().span().start);
        let fmt_string = FmtStringExpr::new(parts, is_raw, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id =
            self.alloc_node(NodeKind::Expression, AnyNode::FmtStringExpr(fmt_string), span);

        Ok(node_id)
    }

    /// Parse an identifier expression (variable name)
    pub(crate) fn parse_identifier_expr(&mut self) -> ParseResult<NodeID> {
        if self.current_token().kind() != &TokenKind::Identifier {
            let span = self.current_token().span().clone();

            return Err(ParseErrorBuilder::new()
                .message(format!("Expected identifier, got {:?}", self.current_token().kind()))
                .span(self.create_source_span(span.start, span.end))
                .build());
        }

        // Get the identifier name
        let name = self.current_token().lexeme();
        let start = self.current_token().span().start;
        let end = self.current_token().span().end;
        let span = Span::new(start, end);

        // Advance past the identifier
        let _ = self.advance();

        // Create the variable identifier
        let variable = VariableIdent::new(name.to_string(), NodeID::new(0, 0), span);

        // Allocate the node
        let node_id =
            self.alloc_node(NodeKind::Declaration, AnyNode::VariableIdent(variable), span);

        Ok(node_id)
    }

    /// Parse a lambda expression
    ///
    /// Handles lambda expressions of the form: `lambda args: expression`
    pub(crate) fn parse_lambda_expr(&mut self) -> ParseResult<NodeID> {
        let start = self.current_token().span().start;

        // Expect and consume 'lambda' keyword
        self.expect(TokenKind::Lambda)?;

        // Parse parameters (if any)
        let mut params = Vec::new();

        // Check if there are parameters (before the colon)
        if !self.check(TokenKind::Colon) {
            // Parse first parameter
            let param = self.parse_identifier()?;
            params.push(param);

            // Parse additional parameters
            while self.check(TokenKind::Comma) {
                let _ = self.advance(); // Consume ','
                let param = self.parse_identifier()?;
                params.push(param);
            }
        }

        // Expect colon
        self.expect(TokenKind::Colon)?;

        // Parse the lambda body expression
        let body = self.parse_expression()?;

        // Get the end position
        let end =
            self.ast.get_node(body).map_or(self.current_token().span().end, |node| node.span.end);

        let span = Span::new(start, end);

        // Create the lambda expression
        let lambda = LambdaExpr::new(params, body, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::LambdaExpr(lambda), span);

        Ok(node_id)
    }

    /// Parse a literal expression (numbers, strings, booleans, None)
    pub(crate) fn parse_literal(&mut self) -> ParseResult<NodeID> {
        let start = self.current_token().span().start;
        let end = self.current_token().span().end;
        let span = Span::new(start, end);

        // Get the lexeme of the current token
        let lexeme = self.current_token().lexeme();

        // Create the appropriate literal value based on token kind
        let literal_value = match self.current_token().kind() {
            TokenKind::IntLiteral => {
                // Parse integer literal
                let value_str = lexeme
                    .trim_start_matches("0x")
                    .trim_start_matches("0b")
                    .trim_start_matches("0o");

                if let Ok(v) = value_str.parse::<i64>() {
                    LiteralValue::Int(v)
                } else {
                    return Err(ParseErrorBuilder::new()
                        .message(format!("Failed to parse integer literal: {lexeme}"))
                        .span(self.create_source_span(start, end))
                        .build());
                }
            }
            TokenKind::FloatLiteral => {
                // Parse float literal
                if let Ok(v) = lexeme.parse::<f64>() {
                    LiteralValue::Float(v)
                } else {
                    return Err(ParseErrorBuilder::new()
                        .message(format!("Failed to parse float literal: {lexeme}"))
                        .span(self.create_source_span(start, end))
                        .build());
                }
            }
            TokenKind::StringLiteral
            | TokenKind::MultilineStringLiteral
            | TokenKind::BytesLiteral => {
                // Handle string literals, removing quotes
                let content = self.current_token().lexeme_unquote().to_string();

                LiteralValue::String(content)
            }
            TokenKind::True => LiteralValue::Bool(true),
            TokenKind::False => LiteralValue::Bool(false),
            TokenKind::None => LiteralValue::None,
            _ => {
                return Err(ParseErrorBuilder::new()
                    .message(format!(
                        "Unexpected token in literal: {:?}",
                        self.current_token().kind()
                    ))
                    .span(self.create_source_span(start, end))
                    .build());
            }
        };

        // Advance past the literal token
        let _ = self.advance();

        // Create the Literal node
        let literal = LiteralExpr::new(literal_value, lexeme.to_string(), NodeID::new(0, 0), span);

        // Allocate the node
        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::LiteralExpr(literal), span);

        Ok(node_id)
    }

    /// Parse a starred expression
    ///
    /// Handles both single star (*args) and double star (**kwargs) expressions
    pub(crate) fn parse_starred_expr(&mut self, double_star: bool) -> ParseResult<NodeID> {
        // Record the start position
        let start = self.current_token().span().start;

        // Consume the * or ** token (already done by the caller)

        // Parse the value expression
        let value = self.parse_expression()?;

        // Get the end position from the value expression
        let end = self
            .ast
            .get_node(value)
            .map_or_else(|| self.current_token().span().end, |node| node.span.end);
        let span = Span::new(start, end);

        // Create the starred expression node
        let starred = StarredExpr::new(value, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::StarredExpr(starred), span);

        Ok(node_id)
    }

    /// Parse a template string expression
    ///
    /// Handles template strings of the form:
    /// - `t"User {name} has {count} items"`
    /// - `rt"Raw template with {expr}"`
    pub(crate) fn parse_template_string(&mut self) -> ParseResult<NodeID> {
        // Record the start position and check token type
        let start = self.current_token().span().start;
        let end = self.current_token().span().end;

        // Determine if we're handling a regular or multiline template string
        let is_multiline = match self.current_token().kind() {
            TokenKind::TmplStringLiteral => false,
            TokenKind::MultilineTmplStringLiteral => true,
            _ => {
                return Err(ParseErrorBuilder::new()
                    .message("Expected template string")
                    .span(self.create_source_span(start, end))
                    .build());
            }
        };

        // Check if it's a raw template string (starts with 'r')
        let is_raw = self.current.lexeme.starts_with('r') || self.current.lexeme.starts_with('R');

        // Get the lexeme for processing
        let lexeme = if is_multiline {
            &self.current.lexeme[if is_raw { 2 } else { 1 }..self.current.lexeme.len() - 3]
        } else {
            &self.current.lexeme[if is_raw { 2 } else { 1 }..self.current.lexeme.len() - 1]
        };

        // Consume the template string token
        let _ = self.advance();

        // Parse template parts
        let mut parts = Vec::new();

        // Find all expressions in the template string (between { and })
        let mut start_idx = 0;
        let mut in_expr = false;
        let mut expr_start = 0;

        // Basic parser for template string parts
        for (i, c) in lexeme.chars().enumerate() {
            if c == '{' && !in_expr {
                // End the current literal part if there is one
                if i > start_idx {
                    let literal = &lexeme[start_idx..i];
                    parts.push(TemplateStringPart::Literal(literal.to_string()));
                }
                in_expr = true;
                expr_start = i + 1;
            } else if c == '}' && in_expr {
                // Parse the embedded expression
                let expr_str = &lexeme[expr_start..i];
                let expr_node = self.parse_embedded_expression(expr_str)?;

                parts.push(TemplateStringPart::Expression(expr_node));

                in_expr = false;
                start_idx = i + 1;
            }
        }

        // Add any remaining literal text
        if start_idx < lexeme.len() && !in_expr {
            let literal = &lexeme[start_idx..];
            parts.push(TemplateStringPart::Literal(literal.to_string()));
        }

        // Create the template string node
        let span = Span::new(start, self.current_token().span().start);
        let template_string = TemplateStringExpr::new(parts, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id = self.alloc_node(
            NodeKind::Expression,
            AnyNode::TemplateStringExpr(template_string),
            span,
        );

        Ok(node_id)
    }

    /// Parse a tuple literal
    pub(crate) fn parse_tuple_literal(&mut self, first_element: NodeID) -> ParseResult<NodeID> {
        // Start with the first element
        let mut elements = Vec::new();
        elements.push(first_element);

        // Parse additional elements
        while self.check(TokenKind::Comma) {
            let _ = self.advance(); // Consume ','

            // Check if we're at the end of the tuple (trailing comma)
            if self.check(TokenKind::RightParen) {
                break;
            }

            // Parse the next element
            let element = self.parse_expression()?;
            elements.push(element);
        }

        // Create the tuple literal
        Ok(self.create_tuple_literal(elements))
    }
}
