//! Pattern parsing for the Typhon programming language.
//!
//! This module handles parsing statements of all types,
//! including simple statements, control flow, and compound statements.

use typhon_ast::nodes::{
    AnyNode,
    ClassPattern,
    ClassPatternKeyword,
    IdentifierPattern,
    LiteralPattern,
    MappingPattern,
    MappingPatternItem,
    MatchCase,
    MatchStmt,
    NodeID,
    NodeKind,
    SequencePattern,
    VariableExpr,
    WildcardPattern,
};
use typhon_source::types::Span;

use super::Parser;
use crate::diagnostics::{ParseError, ParseResult};
use crate::lexer::TokenKind;

impl Parser<'_> {
    /// Parse a case statement (e.g. `case pattern:` or `case pattern if guard:`).
    ///
    /// Case statements are part of match statements and contain patterns to match against.
    /// They can optionally have a guard condition (an 'if' followed by an expression).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// case_statement: "case" pattern ["if" expression] ":" block
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple pattern:
    ///
    /// ```python
    /// case 42:
    ///     print("Matched 42")
    /// ```
    ///
    /// Pattern with guard:
    ///
    /// ```python
    /// case [x, y] if x > 0:
    ///     print("Matched positive x")
    /// ```
    ///
    /// Complex pattern:
    ///
    /// ```python
    /// case {"name": name, "age": age} if age >= 18:
    ///     print(f"Adult: {name}")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Pattern parsing fails
    /// - Guard expression is invalid (when present)
    /// - Missing `:` after pattern
    /// - Block parsing fails
    fn parse_case_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'case' token
        self.skip();

        // Parse the pattern
        let pattern = self.parse_pattern()?;

        // Check for optional guard condition
        let guard = if self.check(TokenKind::If) {
            self.skip(); // consume 'if'
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Expect a colon after the pattern (and optional guard)
        self.expect(TokenKind::Colon)?;

        // Parse the body of the case statement
        let body = self.parse_block()?;

        // Calculate the end position (end of the body)
        let end_pos = if body.is_empty() {
            // Fallback to current token position
            self.current_token().span.start
        } else {
            // Use the position after the last statement
            let last_stmt = body.last().unwrap();

            self.get_node_span(*last_stmt)?.end
        };

        // Create a span for the entire case statement
        let span = Span::new(start_pos, end_pos);

        // Create a CaseStmt node
        let case_stmt = MatchCase::new(pattern, guard, body.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Pattern, AnyNode::MatchCase(case_stmt), span);

        // Set parent-child relationships
        self.set_parent(pattern, node_id);

        if let Some(guard_expr) = guard {
            self.set_parent(guard_expr, node_id);
        }

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }

    /// Parse a literal pattern (e.g. `case 42:` or `case "string":`).
    ///
    /// Literal patterns match exact values including numbers, strings, booleans,
    /// and None. The value must exactly equal the literal for the pattern to match.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// literal_pattern: int_literal
    ///                | float_literal
    ///                | string_literal
    ///                | "True"
    ///                | "False"
    ///                | "None"
    /// ```
    ///
    /// ## Examples
    ///
    /// Integer literal:
    ///
    /// ```python
    /// case 42:
    ///     print("The answer")
    /// ```
    ///
    /// String literal:
    ///
    /// ```python
    /// case "error":
    ///     handle_error()
    /// ```
    ///
    /// Boolean and None:
    ///
    /// ```python
    /// case True:
    ///     enable_feature()
    /// case None:
    ///     set_default()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Literal expression parsing fails
    /// - Invalid literal value
    fn parse_literal_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the literal expression
        let value = self.parse_expression()?;

        // Get the end position
        let end_pos = self.get_node_span(value)?.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a LiteralPattern node
        let literal_pattern = LiteralPattern::new(value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Pattern, AnyNode::LiteralPattern(literal_pattern), span);

        // Set parent-child relationship
        self.set_parent(value, node_id);

        Ok(node_id)
    }

    /// Parse an identifier pattern (e.g. `case variable:`).
    ///
    /// Identifier patterns bind the matched value to a variable name. The pattern
    /// always matches and assigns the value to the identifier for use in the case body.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// identifier_pattern: identifier
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple binding:
    ///
    /// ```python
    /// case value:
    ///     print(f"Got: {value}")
    /// ```
    ///
    /// With type checking in body:
    ///
    /// ```python
    /// case x:
    ///     if isinstance(x, int):
    ///         process_int(x)
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Identifier parsing fails
    /// - Invalid identifier name
    fn parse_identifier_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the identifier
        let name = self.parse_identifier()?;

        // Get the end position
        let end_pos = self.get_node_span(name)?.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an IdentifierPattern node
        let identifier_pattern = IdentifierPattern::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(
            NodeKind::Pattern,
            AnyNode::IdentifierPattern(identifier_pattern),
            span,
        );

        // Set parent-child relationship
        self.set_parent(name, node_id);

        Ok(node_id)
    }

    /// Parse a wildcard pattern (e.g. `case _:`).
    ///
    /// The wildcard pattern matches any value but doesn't bind it to a name.
    /// It's commonly used as the default case in match statements.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// wildcard_pattern: "_"
    /// ```
    ///
    /// ## Examples
    ///
    /// Default case:
    ///
    /// ```python
    /// match value:
    ///     case 1:
    ///         print("One")
    ///     case _:
    ///         print("Something else")
    /// ```
    ///
    /// ## Errors
    ///
    /// This method does not return errors under normal circumstances.
    #[allow(clippy::unnecessary_wraps)]
    fn parse_wildcard_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the span of the '_' token
        let token = self.current_token().clone();
        let span = self.token_to_span(&token);

        // Consume the '_' token
        self.skip();

        // Create a WildcardPattern node
        let wildcard_pattern = WildcardPattern::new(NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(
            NodeKind::Pattern,
            AnyNode::WildcardPattern(wildcard_pattern),
            span,
        );

        Ok(node_id)
    }

    /// Parse a sequence pattern (e.g. `case [a, b, *rest]:`).
    ///
    /// Sequence patterns match list-like structures. They can capture individual
    /// elements and use a starred pattern to capture remaining elements.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// sequence_pattern: "[" [pattern_list] "]"
    /// pattern_list: pattern ("," pattern)* ["," ["*" pattern]] [","]
    ///             | "*" pattern [","]
    /// ```
    ///
    /// ## Examples
    ///
    /// Fixed-length pattern:
    ///
    /// ```python
    /// case [x, y]:
    ///     print(f"Two elements: {x}, {y}")
    /// ```
    ///
    /// With starred pattern:
    ///
    /// ```python
    /// case [first, *rest]:
    ///     print(f"First: {first}, Rest: {rest}")
    /// ```
    ///
    /// Complex pattern:
    ///
    /// ```python
    /// case [x, y, *middle, z]:
    ///     print(f"Start: {x},{y}, Middle: {middle}, End: {z}")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing `]` to close sequence
    /// - Multiple starred patterns (only one allowed)
    /// - Invalid pattern syntax
    /// - Pattern parsing fails
    fn parse_sequence_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the '[' token
        self.expect(TokenKind::LeftBracket)?;

        let mut patterns = Vec::new();
        let mut starred = None;

        // Parse patterns in the sequence
        if !self.check(TokenKind::RightBracket) {
            loop {
                // Check for a starred pattern
                if self.check(TokenKind::Star) {
                    if starred.is_some() {
                        let span = self.create_source_span(
                            self.current_token().span.start,
                            self.current_token().span.end,
                        );
                        return Err(ParseError::invalid_syntax(
                            "Only one starred expression allowed in a sequence pattern",
                            span,
                        ));
                    }

                    self.skip(); // consume '*'
                    starred = Some(self.parse_pattern()?);
                } else {
                    patterns.push(self.parse_pattern()?);
                }

                // Check if there are more patterns
                if self.check(TokenKind::Comma) {
                    self.skip(); // consume ','

                    // Allow trailing comma
                    if self.check(TokenKind::RightBracket) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Expect closing ']'
        self.expect(TokenKind::RightBracket)?;

        // Get the end position
        let end_pos = self.current_token().span().end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a SequencePattern node
        let sequence_pattern =
            SequencePattern::new(patterns.clone(), starred, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(
            NodeKind::Pattern,
            AnyNode::SequencePattern(sequence_pattern),
            span,
        );

        // Set parent-child relationships
        for pattern in &patterns {
            self.set_parent(*pattern, node_id);
        }

        if let Some(star_pattern) = starred {
            self.set_parent(star_pattern, node_id);
        }

        Ok(node_id)
    }

    /// Parse a mapping pattern (e.g. `case {"key": value, **rest}:`).
    ///
    /// Mapping patterns match dictionary-like structures. They can match specific
    /// key-value pairs and use a double-starred pattern to capture remaining items.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// mapping_pattern: "{" [mapping_items] "}"
    /// mapping_items: mapping_item ("," mapping_item)* ["," ["**" pattern]] [","]
    ///              | "**" pattern [","]
    /// mapping_item: expression ":" pattern
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple key-value pattern:
    ///
    /// ```python
    /// case {"type": "error"}:
    ///     handle_error()
    /// ```
    ///
    /// Multiple keys:
    ///
    /// ```python
    /// case {"name": name, "age": age}:
    ///     print(f"{name} is {age} years old")
    /// ```
    ///
    /// With double-starred pattern:
    ///
    /// ```python
    /// case {"required": value, **rest}:
    ///     print(f"Required: {value}, Rest: {rest}")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing `}` to close mapping
    /// - Multiple double-starred patterns (only one allowed)
    /// - Missing `:` between key and value
    /// - Invalid key expression
    /// - Invalid value pattern
    fn parse_mapping_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the `{` token
        self.expect(TokenKind::LeftBrace)?;

        let mut items = Vec::new();
        let mut starred = None;

        // Parse key-value pairs in the mapping
        if !self.check(TokenKind::RightBrace) {
            loop {
                // Check for a double-starred pattern
                if self.check(TokenKind::DoubleStar) {
                    if starred.is_some() {
                        let span = self.create_source_span(
                            self.current_token().span.start,
                            self.current_token().span.end,
                        );
                        return Err(ParseError::invalid_syntax(
                            "Only one double-starred expression allowed in a mapping pattern",
                            span,
                        ));
                    }

                    self.skip(); // consume '**'
                    starred = Some(self.parse_pattern()?);
                } else {
                    // Parse the key
                    let key = self.parse_expression()?;

                    // Expect ':'
                    self.expect(TokenKind::Colon)?;

                    // Parse the value pattern
                    let value = self.parse_pattern()?;

                    items.push(MappingPatternItem::new(key, value));
                }

                // Check if there are more pairs
                if self.check(TokenKind::Comma) {
                    self.skip(); // consume ','

                    // Allow trailing comma
                    if self.check(TokenKind::RightBrace) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Expect closing '}'
        self.expect(TokenKind::RightBrace)?;

        // Get the end position
        let end_pos = self.current_token().span().end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a MappingPattern node
        let mapping_pattern =
            MappingPattern::new(items.clone(), starred, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Pattern, AnyNode::MappingPattern(mapping_pattern), span);

        // Set parent-child relationships
        for item in &items {
            self.set_parent(item.key, node_id);
            self.set_parent(item.value, node_id);
        }

        if let Some(star_pattern) = starred {
            self.set_parent(star_pattern, node_id);
        }

        Ok(node_id)
    }

    /// Parse a class pattern (e.g. `case Point(x=0, y=0):`).
    ///
    /// Class patterns match instances of classes by checking attributes. They support
    /// both positional and keyword arguments, with positional arguments required to
    /// come before keyword arguments.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// class_pattern: identifier "(" [pattern_arguments] ")"
    /// pattern_arguments: positional_patterns ["," keyword_patterns] [","]
    ///                  | keyword_patterns [","]
    /// positional_patterns: pattern ("," pattern)*
    /// keyword_patterns: keyword_pattern ("," keyword_pattern)*
    /// keyword_pattern: identifier "=" pattern
    /// ```
    ///
    /// ## Examples
    ///
    /// Positional arguments:
    ///
    /// ```python
    /// case Point(0, 0):
    ///     print("Origin")
    /// ```
    ///
    /// Keyword arguments:
    ///
    /// ```python
    /// case Point(x=0, y=0):
    ///     print("Origin with keywords")
    /// ```
    ///
    /// Mixed arguments:
    ///
    /// ```python
    /// case Point(x, y=0):
    ///     print(f"Point on x-axis at {x}")
    /// ```
    ///
    /// Nested patterns:
    ///
    /// ```python
    /// case Rectangle(Point(x1, y1), Point(x2, y2)):
    ///     print(f"Rectangle from ({x1},{y1}) to ({x2},{y2})")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Class name is invalid
    /// - Missing `(` or `)` around arguments
    /// - Positional patterns appear after keyword patterns
    /// - Invalid pattern syntax
    /// - Missing `=` in keyword argument
    fn parse_class_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the class name
        let class_name = self.parse_identifier()?;

        // Expect '('
        self.expect(TokenKind::LeftParen)?;

        let mut patterns = Vec::new();
        let mut keywords = Vec::new();
        let mut seen_keyword = false;

        // Parse arguments
        if !self.check(TokenKind::RightParen) {
            loop {
                // Check if we have a keyword argument (identifier=pattern)
                if self.check(TokenKind::Identifier) && self.peek_token().kind == TokenKind::Equal {
                    seen_keyword = true;

                    // Parse the keyword name
                    let name_token = self.current_token();
                    let name_span = self.token_to_span(name_token);
                    let name_str = name_token.lexeme.to_string();

                    // Create a Variable node for the name
                    let variable = VariableExpr::new(name_str, NodeID::placeholder(), name_span);

                    // Add the variable node to the AST
                    let name_node_id = self.ast.alloc_node(
                        NodeKind::Expression,
                        AnyNode::VariableExpr(variable),
                        name_span,
                    );

                    self.skip(); // consume the identifier
                    self.skip(); // consume the '=' token

                    // Parse the pattern value
                    let pattern = self.parse_pattern()?;

                    // Create the keyword argument
                    keywords.push(ClassPatternKeyword::new(name_node_id, pattern));
                } else if !seen_keyword {
                    // Parse a positional pattern
                    patterns.push(self.parse_pattern()?);
                } else {
                    let span = self.create_source_span(
                        self.current_token().span.start,
                        self.current_token().span.end,
                    );
                    return Err(ParseError::invalid_syntax(
                        "Positional patterns must come before keyword patterns",
                        span,
                    ));
                }

                // Check if there are more arguments
                if self.check(TokenKind::Comma) {
                    self.skip(); // consume ','

                    // Allow trailing comma
                    if self.check(TokenKind::RightParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Expect closing ')'
        self.expect(TokenKind::RightParen)?;

        // Get the end position
        let end_pos = self.current_token().span().end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a ClassPattern node
        let class_pattern = ClassPattern::new(
            class_name,
            patterns.clone(),
            keywords.clone(),
            NodeID::placeholder(),
            span,
        );

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Pattern, AnyNode::ClassPattern(class_pattern), span);

        // Set parent-child relationships
        self.set_parent(class_name, node_id);

        for pattern in &patterns {
            self.set_parent(*pattern, node_id);
        }

        for keyword in &keywords {
            self.set_parent(keyword.name, node_id);
            self.set_parent(keyword.pattern, node_id);
        }

        Ok(node_id)
    }

    /// Parse a match statement (e.g. `match x:`).
    ///
    /// Match statements were introduced in Python 3.10 and provide a way
    /// to do pattern matching on values. The general structure is:
    ///
    /// ```python
    /// match subject:
    ///     case pattern1:
    ///         # code for pattern1
    ///     case pattern2 if condition:
    ///         # code for pattern2 if condition is true
    ///     case _:
    ///         # default case
    /// ```
    pub fn parse_match_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'match' token
        self.skip();

        // Parse the subject expression
        let subject = self.parse_expression()?;

        // Expect a colon after the subject
        self.expect(TokenKind::Colon)?;

        // Parse the body (which should be a block of case statements)
        let mut cases = Vec::new();

        // Parse the block of case statements
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;

        // Update indentation level in the context
        let indent_level = self.context_stack.current_indent_level() + 1;
        self.context_stack.current_mut().indent_level = indent_level;

        // Parse case statements until we hit a dedent
        while !self.check(TokenKind::Dedent) && !self.check(TokenKind::EndOfFile) {
            // Each case statement should start with 'case'
            if self.check(TokenKind::Case) {
                let case = self.parse_case_statement()?;
                cases.push(case);
            } else {
                let token = self.current_token();
                let span = self.create_source_span(token.span.start, token.span.end);
                return Err(ParseError::unexpected_token(token.kind, vec![TokenKind::Case], span));
            }

            // Skip newlines between case statements
            self.skip_newlines();
        }

        // Expect a dedent at the end of the match block
        self.expect(TokenKind::Dedent)?;

        // Calculate the end position (end of the last case statement)
        let end_pos = if cases.is_empty() {
            // Fallback to current token position
            self.current_token().span.start
        } else {
            // Use the end position of the last case statement
            let last_case = cases.last().unwrap();

            self.get_node_span(*last_case)?.end
        };

        // Create a span for the entire match statement
        let span = Span::new(start_pos, end_pos);

        // Create a MatchStmt node
        let match_stmt = MatchStmt::new(subject, cases.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::MatchStmt(match_stmt), span);

        // Set parent-child relationships
        self.set_parent(subject, node_id);
        for case in &cases {
            self.set_parent(*case, node_id);
        }

        Ok(node_id)
    }

    /// Parse a pattern node used in pattern matching.
    ///
    /// This is the main entry point for parsing patterns in case statements.
    /// It dispatches to more specific pattern parsing methods based on the current token.
    fn parse_pattern(&mut self) -> ParseResult<NodeID> {
        match self.current_token().kind {
            // Literal patterns: numbers, strings, booleans, None
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::None
            | TokenKind::True
            | TokenKind::False => self.parse_literal_pattern(),

            // Wildcard pattern: '_'
            TokenKind::Underscore => self.parse_wildcard_pattern(),

            // Identifier pattern: variable names
            TokenKind::Identifier => {
                // Could be a simple identifier or a class pattern
                if self.peek_token().kind == TokenKind::LeftParen {
                    self.parse_class_pattern()
                } else {
                    self.parse_identifier_pattern()
                }
            }

            // Sequence pattern: [a, b, *rest]
            TokenKind::LeftBracket => self.parse_sequence_pattern(),

            // Mapping pattern: {"key": value, **rest}
            TokenKind::LeftBrace => self.parse_mapping_pattern(),

            // Should never get here if the parser is working correctly
            _ => {
                let token = self.current_token();
                let span = self.create_source_span(token.span.start, token.span.end);
                Err(ParseError::unexpected_token(
                    token.kind,
                    vec![
                        TokenKind::IntLiteral,
                        TokenKind::StringLiteral,
                        TokenKind::Identifier,
                        TokenKind::Underscore,
                        TokenKind::LeftBracket,
                        TokenKind::LeftBrace,
                    ],
                    span,
                ))
            }
        }
    }
}
