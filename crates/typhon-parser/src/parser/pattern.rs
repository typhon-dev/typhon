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
    VariableIdent,
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
    /// ```python
    /// case [x, y] if x > 0:  # Pattern with guard
    ///     # code executed when pattern matches and guard is true
    /// ```
    fn parse_case_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'case' token
        let _ = self.advance();

        // Parse the pattern
        let pattern = self.parse_pattern()?;

        // Check for optional guard condition
        let guard = if self.check(TokenKind::If) {
            let _ = self.advance(); // consume 'if'
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
            self.ast.get_node(*last_stmt).unwrap().span.end
        };

        // Create a span for the entire case statement
        let span = Span::new(start_pos, end_pos);

        // Create a CaseStmt node
        let case_stmt = MatchCase::new(pattern, guard, body.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Pattern, AnyNode::MatchCase(case_stmt), span);

        // Set parent-child relationships
        let _ = self.ast.set_parent(pattern, node_id);

        if let Some(guard_expr) = guard {
            let _ = self.ast.set_parent(guard_expr, node_id);
        }

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }

    /// Parse a literal pattern (e.g. `case 42:` or `case "string":`).
    fn parse_literal_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the literal expression
        let value = self.parse_expression()?;

        // Get the end position
        let end_pos = self.ast.get_node(value).unwrap().span.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a LiteralPattern node
        let literal_pattern = LiteralPattern::new(value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Pattern, AnyNode::LiteralPattern(literal_pattern), span);

        // Set parent-child relationship
        let _ = self.ast.set_parent(value, node_id);

        Ok(node_id)
    }

    /// Parse an identifier pattern (e.g. `case variable:`).
    fn parse_identifier_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the identifier
        let name = self.parse_identifier()?;

        // Get the end position
        let end_pos = self.ast.get_node(name).unwrap().span.end;

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
        let _ = self.ast.set_parent(name, node_id);

        Ok(node_id)
    }

    /// Parse a wildcard pattern (e.g. `case _:`).
    #[allow(clippy::unnecessary_wraps)]
    fn parse_wildcard_pattern(&mut self) -> ParseResult<NodeID> {
        // Get the span of the '_' token
        let token = self.current_token().clone();
        let span = self.token_to_span(&token);

        // Consume the '_' token
        let _ = self.advance();

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

                    let _ = self.advance(); // consume '*'
                    starred = Some(self.parse_pattern()?);
                } else {
                    patterns.push(self.parse_pattern()?);
                }

                // Check if there are more patterns
                if self.check(TokenKind::Comma) {
                    let _ = self.advance(); // consume ','

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
            let _ = self.ast.set_parent(*pattern, node_id);
        }

        if let Some(star_pattern) = starred {
            let _ = self.ast.set_parent(star_pattern, node_id);
        }

        Ok(node_id)
    }

    /// Parse a mapping pattern (e.g. `case {"key": value, **rest}:`).
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

                    let _ = self.advance(); // consume '**'
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
                    let _ = self.advance(); // consume ','

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
            let _ = self.ast.set_parent(item.key, node_id);
            let _ = self.ast.set_parent(item.value, node_id);
        }

        if let Some(star_pattern) = starred {
            let _ = self.ast.set_parent(star_pattern, node_id);
        }

        Ok(node_id)
    }

    /// Parse a class pattern (e.g. `case Point(x=0, y=0`):').
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
                    let variable = VariableIdent::new(name_str, NodeID::placeholder(), name_span);

                    // Add the variable node to the AST
                    let name_node_id = self.ast.alloc_node(
                        NodeKind::Expression,
                        AnyNode::VariableIdent(variable),
                        name_span,
                    );

                    let _ = self.advance(); // consume the identifier
                    let _ = self.advance(); // consume the '=' token

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
                    let _ = self.advance(); // consume ','

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
        let _ = self.ast.set_parent(class_name, node_id);

        for pattern in &patterns {
            let _ = self.ast.set_parent(*pattern, node_id);
        }

        for keyword in &keywords {
            let _ = self.ast.set_parent(keyword.name, node_id);
            let _ = self.ast.set_parent(keyword.pattern, node_id);
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
        let _ = self.advance();

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
            while self.check(TokenKind::Newline) {
                let _ = self.advance();
            }
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
            self.ast.get_node(*last_case).unwrap().span.end
        };

        // Create a span for the entire match statement
        let span = Span::new(start_pos, end_pos);

        // Create a MatchStmt node
        let match_stmt = MatchStmt::new(subject, cases.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::MatchStmt(match_stmt), span);

        // Set parent-child relationships
        let _ = self.ast.set_parent(subject, node_id);
        for case in &cases {
            let _ = self.ast.set_parent(*case, node_id);
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
