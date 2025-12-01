//! Helper functions for creating expression nodes

use typhon_ast::nodes::{
    AnyNode,
    BinaryOpExpr,
    BinaryOpKind,
    DictExpr,
    GroupingExpr,
    ListExpr,
    NodeID,
    NodeKind,
    SetExpr,
    TupleExpr,
};
use typhon_source::types::Span;

use crate::diagnostics::{ParseErrorBuilder, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Create a binary operation node with parent pointer updates
    #[inline]
    pub(crate) fn create_binary_op_node(
        &mut self,
        op: BinaryOpKind,
        lhs: NodeID,
        rhs: NodeID,
        start: usize,
    ) -> NodeID {
        // Get end position
        let end =
            self.ast.get_node(rhs).map_or(self.current_token().span().end, |node| node.span.end);
        let span = Span::new(start, end);
        let binary_op = BinaryOpExpr::new(op, lhs, rhs, NodeID::new(0, 0), span);

        let node_id = self.alloc_node(NodeKind::Expression, AnyNode::BinaryOpExpr(binary_op), span);

        // Update parent pointers
        if let Some(left_node) = self.ast.get_node_mut(lhs) {
            left_node.parent = Some(node_id);
        }

        if let Some(right_node) = self.ast.get_node_mut(rhs) {
            right_node.parent = Some(node_id);
        }

        node_id
    }

    /// Create a dictionary literal expression
    #[inline]
    pub(crate) fn create_dict_literal(
        &mut self,
        elements: Vec<(NodeID, NodeID)>,
        start: usize,
        end: usize,
    ) -> NodeID {
        let span = Span::new(start, end);
        let dict = DictExpr::new(elements, NodeID::new(0, 0), span);

        self.alloc_node(NodeKind::Expression, AnyNode::DictExpr(dict), span)
    }

    /// Create a grouping expression
    #[inline]
    pub(crate) fn create_grouping_expr(
        &mut self,
        expr: NodeID,
        start: usize,
        end: usize,
    ) -> NodeID {
        let span = Span::new(start, end);
        let grouping = GroupingExpr::new(expr, NodeID::new(0, 0), span);

        self.alloc_node(NodeKind::Expression, AnyNode::GroupingExpr(grouping), span)
    }

    /// Create a list literal expression
    #[inline]
    pub(crate) fn create_list_literal(
        &mut self,
        elements: Vec<NodeID>,
        start: usize,
        end: usize,
    ) -> NodeID {
        let span = Span::new(start, end);
        let list = ListExpr::new(elements, NodeID::new(0, 0), span);

        self.alloc_node(NodeKind::Expression, AnyNode::ListExpr(list), span)
    }

    /// Create a set literal expression
    #[inline]
    pub(crate) fn create_set_literal(
        &mut self,
        elements: Vec<NodeID>,
        start: usize,
        end: usize,
    ) -> NodeID {
        let span = Span::new(start, end);
        let set = SetExpr::new(elements, NodeID::new(0, 0), span);

        self.alloc_node(NodeKind::Expression, AnyNode::SetExpr(set), span)
    }

    /// Create a tuple literal expression
    #[inline]
    pub(crate) fn create_tuple_literal(&mut self, elements: Vec<NodeID>) -> NodeID {
        let start = if elements.is_empty() {
            self.current_token().span().start
        } else {
            // Get the span of the first element
            let first = elements[0];
            self.ast
                .get_node(first)
                .map_or(self.current_token().span().start, |node| node.span.start)
        };

        let end = if elements.is_empty() {
            self.current_token().span().end
        } else {
            // Get the span of the last element
            let last = *elements.last().unwrap();
            self.ast.get_node(last).map_or(self.current_token().span().end, |node| node.span.end)
        };

        let span = Span::new(start, end);
        let tuple = TupleExpr::new(elements, NodeID::new(0, 0), span);

        // Allocate and return the node
        self.alloc_node(NodeKind::Expression, AnyNode::TupleExpr(tuple), span)
    }

    /// Convert current token to [`BinaryOpKind`] and advance
    ///
    /// Returns error if the token is not a valid binary operator
    pub(crate) fn token_to_binary_op(&mut self) -> ParseResult<BinaryOpKind> {
        // Clone the operator to avoid borrow checker issues
        let op_kind = *self.current_token().kind();

        Ok(match op_kind {
            TokenKind::Plus => {
                let _ = self.advance();
                BinaryOpKind::Add
            }
            TokenKind::Minus => {
                let _ = self.advance();
                BinaryOpKind::Sub
            }
            TokenKind::Star => {
                let _ = self.advance();
                BinaryOpKind::Mul
            }
            TokenKind::Slash => {
                let _ = self.advance();
                BinaryOpKind::Div
            }
            TokenKind::DoubleSlash => {
                let _ = self.advance();
                BinaryOpKind::FloorDiv
            }
            TokenKind::Percent => {
                let _ = self.advance();
                BinaryOpKind::Mod
            }
            TokenKind::DoubleStar => {
                let _ = self.advance();
                BinaryOpKind::Pow
            }
            TokenKind::LeftShift => {
                let _ = self.advance();
                BinaryOpKind::LShift
            }
            TokenKind::RightShift => {
                let _ = self.advance();
                BinaryOpKind::RShift
            }
            TokenKind::Ampersand => {
                let _ = self.advance();
                BinaryOpKind::BitAnd
            }
            TokenKind::Pipe => {
                let _ = self.advance();
                BinaryOpKind::BitOr
            }
            TokenKind::Caret => {
                let _ = self.advance();
                BinaryOpKind::BitXor
            }
            TokenKind::Equal => {
                let _ = self.advance();
                BinaryOpKind::Eq
            }
            TokenKind::NotEqual => {
                let _ = self.advance();
                BinaryOpKind::NotEq
            }
            TokenKind::LessThan => {
                let _ = self.advance();
                BinaryOpKind::Lt
            }
            TokenKind::LessEqual => {
                let _ = self.advance();
                BinaryOpKind::LtEq
            }
            TokenKind::GreaterThan => {
                let _ = self.advance();
                BinaryOpKind::Gt
            }
            TokenKind::GreaterEqual => {
                let _ = self.advance();
                BinaryOpKind::GtEq
            }
            TokenKind::In => {
                let _ = self.advance();
                BinaryOpKind::In
            }
            TokenKind::Is => {
                // Check for "is not"
                let _ = self.advance(); // consume 'is'
                if self.current_token().kind() == &TokenKind::Not {
                    let _ = self.advance(); // consume 'not'
                    BinaryOpKind::IsNot
                } else {
                    BinaryOpKind::Is
                }
            }
            TokenKind::Not => {
                // This must be "not in"
                let _ = self.advance(); // consume 'not'
                if self.current_token().kind() == &TokenKind::In {
                    let _ = self.advance(); // consume 'in'
                    BinaryOpKind::NotIn
                } else {
                    return Err(ParseErrorBuilder::new()
                        .message("Expected 'in' after 'not'")
                        .span(self.create_source_span(
                            self.current_token().span().start,
                            self.current_token().span().end,
                        ))
                        .build());
                }
            }
            TokenKind::And => {
                let _ = self.advance();
                BinaryOpKind::And
            }
            TokenKind::Or => {
                let _ = self.advance();
                BinaryOpKind::Or
            }
            _ => {
                return Err(ParseErrorBuilder::new()
                    .message(format!("Unexpected operator: {op_kind:?}"))
                    .span(self.create_source_span(
                        self.current_token().span().start,
                        self.current_token().span().end,
                    ))
                    .build());
            }
        })
    }
}
