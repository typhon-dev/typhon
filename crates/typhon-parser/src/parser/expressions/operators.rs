//! Operator parsing and binding power
//!
//! This module contains operator-related parsing including:
//! - Binding power (operator precedence)
//! - Binary operator parsing
//! - Ternary operator parsing (if-else)
//! - Unary operator parsing

use typhon_ast::nodes::{AnyNode, NodeID, NodeKind, TernaryExpr, UnaryOpExpr, UnaryOpKind};
use typhon_source::types::Span;

use crate::diagnostics::{ParseErrorBuilder, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::{Context, ContextType, Parser};

/// Get the binding power for infix operators
///
/// Returns (`left_binding_power`, `right_binding_power`) for operators.
/// - Left-associative operators have `right_bp = left_bp + 1`
/// - Right-associative operators have `right_bp = left_bp - 1`
///
/// Lower binding power means lower precedence (binds less tightly).
///
/// ## Operator Precedence (lowest to highest)
///
/// 1. Ternary (`if`-`else`) - 2/1 (right-associative)
/// 2. Logical OR (`or`) - 3/4
/// 3. Logical AND (`and`) - 5/6
/// 4. Comparisons (`==`, `!=`, `<`, `>`, etc.) - 7/8
/// 5. Bitwise OR (`|`) - 9/10
/// 6. Bitwise XOR (`^`) - 11/12
/// 7. Bitwise AND (`&`) - 13/14
/// 8. Bit shifts (`<<`, `>>`) - 15/16
/// 9. Addition/Subtraction (`+`, `-`) - 17/18
/// 10. Multiplication/Division (`*`, `/`, `//`, `%`) - 19/20
/// 11. Exponentiation (`**`) - 22/21 (right-associative)
/// 12. Postfix (`(`, `[`, `.`) - 30/31
#[inline]
pub(super) const fn infix_binding_power(op: TokenKind) -> Option<(u8, u8)> {
    Some(match op {
        // Ternary (if-else) - right-associative
        TokenKind::If => (2, 1),

        // Logical OR
        TokenKind::Or => (3, 4),

        // Logical AND
        TokenKind::And => (5, 6),

        // Comparisons (can chain: a < b < c)
        // Note: TokenKind::Not is included here for "not in" operator
        TokenKind::Equal
        | TokenKind::NotEqual
        | TokenKind::LessThan
        | TokenKind::LessEqual
        | TokenKind::GreaterThan
        | TokenKind::GreaterEqual
        | TokenKind::In
        | TokenKind::Is
        | TokenKind::Not => (7, 8),

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

impl Parser<'_> {
    /// Parse a binary operator expression
    #[inline]
    pub(super) fn parse_binary_expr(&mut self, lhs: NodeID, right_bp: u8) -> ParseResult<NodeID> {
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

    /// Parse a binary operator expression for comprehensions (excludes ternary in RHS)
    ///
    /// This is like `parse_binary_expr` but recursively uses `parse_comprehension_condition`
    /// for the RHS instead of `parse_expression_bp`, ensuring ternary expressions are
    /// excluded throughout the entire expression tree.
    pub(super) fn parse_comprehension_binary_expr(
        &mut self,
        lhs: NodeID,
        right_bp: u8,
    ) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(lhs)
            .map_or(self.current_token().span().start, |node| node.span.start);

        // Convert token to binary operator kind and advance
        let bin_op = self.token_to_binary_op()?;

        // Parse right-hand side using parse_comprehension_condition_bp
        // to exclude ternary expressions
        let rhs = self.parse_comprehension_condition_bp(right_bp)?;

        // Create binary operation node
        Ok(self.create_binary_op_node(bin_op, lhs, rhs, start))
    }

    /// Parse a ternary conditional expression: `value if condition else other_value`
    #[inline]
    pub(super) fn parse_ternary_expr(
        &mut self,
        value: NodeID,
        right_bp: u8,
    ) -> ParseResult<NodeID> {
        let start = self
            .ast
            .get_node(value)
            .map_or(self.current_token().span().start, |node| node.span.start);

        self.expect(TokenKind::If)?; // consume 'if'

        // Push TernaryExpr context to help nested parsing know we're in a ternary
        self.context_stack.push(Context::new(ContextType::TernaryExpr, None, 0));

        // Parse condition with low precedence
        let condition = self.parse_expression_bp(0)?;

        // Expect 'else'
        self.expect(TokenKind::Else)?;

        // Parse else value with right binding power
        let else_value = self.parse_expression_bp(right_bp)?;

        // Pop TernaryExpr context before returning
        drop(self.context_stack.pop());

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

    /// Parse unary arithmetic operators: `+expr`, `-expr`, `~expr`
    ///
    /// These operators have high binding power (23).
    pub(super) fn parse_unary_arithmetic(&mut self) -> ParseResult<NodeID> {
        let start = self.current_token().span().start;
        let op = match self.current_token().kind() {
            TokenKind::Plus => UnaryOpKind::Pos,
            TokenKind::Minus => UnaryOpKind::Neg,
            TokenKind::Tilde => UnaryOpKind::BitNot,
            _ => unreachable!(),
        };

        self.skip();

        // Parse operand with high binding power (23)
        let operand = self.parse_expression_bp(23)?;

        let end = self
            .ast
            .get_node(operand)
            .map_or(self.current_token().span().end, |node| node.span.end);

        let span = Span::new(start, end);
        let unary_op = UnaryOpExpr::new(op, operand, NodeID::new(0, 0), span);

        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::UnaryOpExpr(unary_op), span);

        // Update parent pointer
        if let Some(operand_node) = self.ast.get_node_mut(operand) {
            operand_node.parent = Some(node_id);
        }

        Ok(node_id)
    }

    /// Parse `not` operator: `not expr`
    ///
    /// The `not` operator has lower binding power (8) than arithmetic unary operators.
    pub(super) fn parse_unary_not(&mut self) -> ParseResult<NodeID> {
        // Check if this is "not in" (handled in infix)
        if self.peek_token().kind == TokenKind::In {
            return Err(ParseErrorBuilder::new()
                .message("Unexpected 'not' at start of expression (use parentheses for 'not in')")
                .span(self.create_source_span(
                    self.current_token().span().start,
                    self.current_token().span().end,
                ))
                .build());
        }

        let start = self.current_token().span().start;
        self.skip();

        // Parse operand with binding power just above comparisons (8)
        let operand = self.parse_expression_bp(8)?;

        let end = self
            .ast
            .get_node(operand)
            .map_or(self.current_token().span().end, |node| node.span.end);

        let span = Span::new(start, end);
        let unary_op = UnaryOpExpr::new(UnaryOpKind::Not, operand, NodeID::new(0, 0), span);

        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::UnaryOpExpr(unary_op), span);

        // Update parent pointer
        if let Some(operand_node) = self.ast.get_node_mut(operand) {
            operand_node.parent = Some(node_id);
        }

        Ok(node_id)
    }
}
