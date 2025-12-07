//! Comprehension expression parsing
//!
//! This module handles parsing of:
//!
//! - List comprehensions: `[expr for x in iter if cond]`
//! - Set comprehensions: `{expr for x in iter if cond}`
//! - Dict comprehensions: `{k: v for x in iter if cond}`
//! - Generator expressions: `(expr for x in iter if cond)`

use typhon_ast::nodes::{
    AnyNode,
    ComprehensionFor,
    DictComprehensionExpr,
    GeneratorExpr,
    ListComprehensionExpr,
    NodeID,
    NodeKind,
    SetComprehensionExpr,
};
use typhon_source::types::Span;

use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::{Context, ContextType, Parser};

impl Parser<'_> {
    /// Parse a single comprehension for clause
    ///
    /// Parses: `for target in iterable [if condition]*`
    pub(crate) fn parse_comprehension_for(&mut self) -> ParseResult<ComprehensionFor> {
        let start = self.current_token().span().start;

        // Parse target (stops at 'in' keyword)
        let target = self.parse_for_target()?;

        // Expect 'in'
        self.expect(TokenKind::In)?;

        // Parse iterable (stop at 'if' keyword to avoid parsing ternary expressions)
        let iterable = self.parse_comprehension_condition()?;

        // Parse conditions (if any)
        let mut conditions = Vec::new();

        while self.expect(TokenKind::If).is_ok() {
            // Use parse_comprehension_condition() to exclude ternary expressions
            conditions.push(self.parse_comprehension_condition()?);
        }

        let end = self.current_token().span().end;
        let span = Span::new(start, end);

        // Create the generator
        Ok(ComprehensionFor::new(target, iterable, conditions, span))
    }

    /// Parse comprehension generators (for clauses)
    ///
    /// Parses one or more for clauses in a comprehension
    pub(crate) fn parse_comprehension_generators(&mut self) -> ParseResult<Vec<ComprehensionFor>> {
        let mut generators = Vec::new();

        // Parse the first generator (required)
        let generator = self.parse_comprehension_for()?;
        generators.push(generator);

        // Parse additional generators (optional)
        while self.expect(TokenKind::For).is_ok() {
            generators.push(self.parse_comprehension_for()?);
        }

        Ok(generators)
    }

    /// Generic comprehension parser
    ///
    /// This helper consolidates the common pattern of parsing comprehensions:
    /// 1. Expect 'for' keyword
    /// 2. Parse generators
    /// 3. Expect closing delimiter
    /// 4. Create and allocate the comprehension node
    fn parse_comprehension_expr<F>(
        &mut self,
        start: usize,
        closing_token: TokenKind,
        create_node: F,
    ) -> ParseResult<NodeID>
    where
        F: FnOnce(&mut Self, Vec<ComprehensionFor>, Span) -> (NodeKind, AnyNode),
    {
        // Push comprehension context to prevent ternary expressions in filters
        self.context_stack.push(Context::new(
            ContextType::Comprehension,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Expect 'for'
        self.expect(TokenKind::For)?;

        // Parse generators (for clauses)
        let generators = self.parse_comprehension_generators()?;

        // Expect closing delimiter
        self.expect(closing_token)?;
        let end = self.current_token().span().end;

        // Pop comprehension context
        drop(self.context_stack.pop());

        let span = Span::new(start, end);

        // Create the comprehension node using the provided closure
        let (node_kind, any_node) = create_node(self, generators, span);

        // Allocate and return the node
        Ok(self.alloc_node(node_kind, any_node, span))
    }

    /// Parse a dictionary comprehension expression
    ///
    /// Parses: `{key: value for target in iterable}`
    pub(crate) fn parse_dict_comprehension(
        &mut self,
        key: NodeID,
        value: NodeID,
        start: usize,
    ) -> ParseResult<NodeID> {
        self.parse_comprehension_expr(start, TokenKind::RightBrace, |_, generators, span| {
            let dict_comp =
                DictComprehensionExpr::new(key, value, generators, NodeID::new(0, 0), span);
            (NodeKind::Expression, AnyNode::DictComprehensionExpr(dict_comp))
        })
    }

    /// Parse a generator expression
    ///
    /// Parses: `(expr for target in iterable)`
    pub(crate) fn parse_generator_expr(
        &mut self,
        expr: NodeID,
        start: usize,
    ) -> ParseResult<NodeID> {
        self.parse_comprehension_expr(start, TokenKind::RightParen, |_, generators, span| {
            let generator = GeneratorExpr::new(expr, generators, NodeID::new(0, 0), span);
            (NodeKind::Expression, AnyNode::GeneratorExpr(generator))
        })
    }

    /// Parse a list comprehension expression
    ///
    /// Parses: `[expr for target in iterable]`
    pub(crate) fn parse_list_comprehension(
        &mut self,
        expr: NodeID,
        start: usize,
    ) -> ParseResult<NodeID> {
        self.parse_comprehension_expr(start, TokenKind::RightBracket, |_, generators, span| {
            let list_comp = ListComprehensionExpr::new(expr, generators, NodeID::new(0, 0), span);
            (NodeKind::Expression, AnyNode::ListComprehensionExpr(list_comp))
        })
    }

    /// Parse a set comprehension expression
    ///
    /// Parses: `{expr for target in iterable}`
    pub(crate) fn parse_set_comprehension(
        &mut self,
        expr: NodeID,
        start: usize,
    ) -> ParseResult<NodeID> {
        self.parse_comprehension_expr(start, TokenKind::RightBrace, |_, generators, span| {
            let set_comp = SetComprehensionExpr::new(expr, generators, NodeID::new(0, 0), span);
            (NodeKind::Expression, AnyNode::SetComprehensionExpr(set_comp))
        })
    }
}
