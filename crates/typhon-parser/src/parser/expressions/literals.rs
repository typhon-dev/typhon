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
    VariableExpr,
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
        self.skip();

        // Parse f-string parts with proper handling of format
        // specifiers, conversions, and nested braces
        let mut parts = Vec::new();
        let mut literal_start = 0;
        let mut brace_depth = 0;
        let mut expr_start = 0;
        let mut expr_end = 0;

        let chars: Vec<(usize, char)> = lexeme.char_indices().collect();
        let mut i = 0;

        while i < chars.len() {
            let (idx, c) = chars[i];

            match c {
                '{' => {
                    // Check for escaped brace {{
                    if i + 1 < chars.len() && chars[i + 1].1 == '{' {
                        // Escaped brace - skip both
                        i += 2;

                        continue;
                    }

                    if brace_depth == 0 {
                        // Starting a new expression
                        // Add any literal text before this
                        if idx > literal_start {
                            let literal = &lexeme[literal_start..idx];
                            parts.push(FmtStringPart::Literal(literal.to_string()));
                        }

                        expr_start = idx + 1;
                        expr_end = idx + 1;
                    }

                    brace_depth += 1;
                }
                '}' => {
                    // Check for escaped brace }}
                    if i + 1 < chars.len() && chars[i + 1].1 == '}' {
                        // Escaped brace - skip both
                        i += 2;

                        continue;
                    }

                    if brace_depth > 0 {
                        brace_depth -= 1;

                        if brace_depth == 0 {
                            // End of expression - parse it
                            let expr_str = &lexeme[expr_start..expr_end];

                            // Parse the expression (without format spec or conversion)
                            let expr_node = self.parse_embedded_expression(expr_str)?;
                            parts.push(FmtStringPart::Expression(expr_node));

                            literal_start = idx + 1;
                        }
                    }
                }
                '!' if brace_depth == 1 => {
                    // Conversion specifier at depth 1 - stop collecting expression
                    // The conversion (!r, !s, !a) and everything after is ignored
                    expr_end = idx;
                }
                ':' if brace_depth == 1 => {
                    // Format specifier at depth 1 - stop collecting expression
                    // The format spec (:...) is ignored
                    if expr_end == expr_start {
                        // Only set expr_end if we haven't hit '!' yet
                        expr_end = idx;
                    }
                }
                _ => {
                    // Regular character - extend expression if we're collecting
                    if brace_depth > 0 && expr_end == idx {
                        expr_end = idx + 1;
                    }
                }
            }

            i += 1;
        }

        // Add any remaining literal text
        if literal_start < lexeme.len() {
            let literal = &lexeme[literal_start..];
            if !literal.is_empty() {
                parts.push(FmtStringPart::Literal(literal.to_string()));
            }
        }

        // Create the f-string node
        let span = Span::new(start, self.current_token().span().start);
        let fmt_string = FmtStringExpr::new(parts, is_raw, NodeID::new(0, 0), span);

        // Allocate the node
        let node_id =
            self.alloc_node(NodeKind::Expression, AnyNode::FmtStringExpr(fmt_string), span);

        Ok(node_id)
    }

    /// Parse an identifier expression (variable name).
    ///
    /// Creates a [`VariableExpr`] node representing a reference to a variable,
    /// function, class, or other named entity in the current scope.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// identifier_expr: IDENTIFIER
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple variable reference:
    ///
    /// ```python
    /// x
    /// my_variable
    /// _private
    /// ```
    ///
    /// In expressions:
    ///
    /// ```python
    /// result = x + y
    /// print(name)
    /// obj.method()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Current token is not an identifier
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
        self.skip();

        // Create the variable identifier
        let variable = VariableExpr::new(name.to_string(), NodeID::new(0, 0), span);

        // Allocate the node - VariableExpr is an Expression, not a Declaration
        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::VariableExpr(variable), span);

        Ok(node_id)
    }

    /// Parse a lambda expression.
    ///
    /// Lambda expressions are anonymous functions that can capture variables from
    /// their enclosing scope. They consist of optional parameters and a single expression body.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// lambda_expr: "lambda" [parameter_list] ":" expression
    /// parameter_list: identifier ("," identifier)*
    /// ```
    ///
    /// ## Examples
    ///
    /// No parameters:
    ///
    /// ```python
    /// lambda: 42
    /// ```
    ///
    /// Single parameter:
    ///
    /// ```python
    /// lambda x: x * 2
    /// ```
    ///
    /// Multiple parameters:
    ///
    /// ```python
    /// lambda x, y: x + y
    /// lambda a, b, c: a * b + c
    /// ```
    ///
    /// In higher-order functions:
    ///
    /// ```python
    /// map(lambda x: x ** 2, numbers)
    /// sorted(items, key=lambda item: item.priority)
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing colon `:` after parameters
    /// - Parameter is not a valid identifier
    /// - Body expression is invalid
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
                self.skip(); // Consume ','
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

    /// Parse a literal expression (numbers, strings, booleans, None, ellipsis).
    ///
    /// Handles all Python literal types including integers, floats, strings (with various
    /// prefixes like raw strings), booleans, None, and the ellipsis (`...`) literal.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// literal: INT_LITERAL | FLOAT_LITERAL | STRING_LITERAL
    ///        | RAW_STRING_LITERAL | MULTILINE_STRING_LITERAL | BYTES_LITERAL
    ///        | "True" | "False" | "None" | "..."
    /// ```
    ///
    /// ## Examples
    ///
    /// Integer literals:
    ///
    /// ```python
    /// 42
    /// 0x2A
    /// 0b101010
    /// 0o52
    /// ```
    ///
    /// Float literals:
    ///
    /// ```python
    /// 3.14
    /// 2.5e-3
    /// 1.0
    /// ```
    ///
    /// String literals:
    ///
    /// ```python
    /// "hello"
    /// 'world'
    /// r"raw\nstring"
    /// """multiline
    /// string"""
    /// b"bytes"
    /// ```
    ///
    /// Boolean and None:
    ///
    /// ```python
    /// True
    /// False
    /// None
    /// ```
    ///
    /// Ellipsis:
    ///
    /// ```python
    /// ...
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Integer literal cannot be parsed (invalid format, out of range)
    /// - Float literal cannot be parsed (invalid format)
    /// - Current token is not a valid literal type
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
            | TokenKind::RawStringLiteral
            | TokenKind::MultilineStringLiteral
            | TokenKind::BytesLiteral => {
                // Handle string literals, removing quotes
                let content = self.current_token().lexeme_unquote().to_string();

                LiteralValue::String(content)
            }
            TokenKind::True => LiteralValue::Bool(true),
            TokenKind::False => LiteralValue::Bool(false),
            TokenKind::None => LiteralValue::None,
            TokenKind::Ellipsis => LiteralValue::Ellipsis,
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
        self.skip();

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
        self.skip();

        // Parse template parts
        let mut parts = Vec::new();

        // Find all expressions in the template string (between { and })
        let mut start_idx = 0;
        let mut in_expr = false;
        let mut expr_start = 0;

        // Basic parser for template string parts
        for (i, c) in lexeme.char_indices() {
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
            self.skip(); // Consume ','

            // Check if we're at the end of the tuple (trailing comma)
            if self.check(TokenKind::RightParen) {
                break;
            }

            // If we're parsing a for-loop target, stop at 'in' keyword
            // This handles cases like: `for x, y in items`
            if self.context_stack.in_for_target() && self.check(TokenKind::In) {
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
