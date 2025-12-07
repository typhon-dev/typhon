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
                self.skip();
                BinaryOpKind::Add
            }
            TokenKind::Minus => {
                self.skip();
                BinaryOpKind::Sub
            }
            TokenKind::Star => {
                self.skip();
                BinaryOpKind::Mul
            }
            TokenKind::Slash => {
                self.skip();
                BinaryOpKind::Div
            }
            TokenKind::DoubleSlash => {
                self.skip();
                BinaryOpKind::FloorDiv
            }
            TokenKind::Percent => {
                self.skip();
                BinaryOpKind::Mod
            }
            TokenKind::DoubleStar => {
                self.skip();
                BinaryOpKind::Pow
            }
            TokenKind::LeftShift => {
                self.skip();
                BinaryOpKind::LShift
            }
            TokenKind::RightShift => {
                self.skip();
                BinaryOpKind::RShift
            }
            TokenKind::Ampersand => {
                self.skip();
                BinaryOpKind::BitAnd
            }
            TokenKind::Pipe => {
                self.skip();
                BinaryOpKind::BitOr
            }
            TokenKind::Caret => {
                self.skip();
                BinaryOpKind::BitXor
            }
            TokenKind::Equal => {
                self.skip();
                BinaryOpKind::Eq
            }
            TokenKind::NotEqual => {
                self.skip();
                BinaryOpKind::NotEq
            }
            TokenKind::LessThan => {
                self.skip();
                BinaryOpKind::Lt
            }
            TokenKind::LessEqual => {
                self.skip();
                BinaryOpKind::LtEq
            }
            TokenKind::GreaterThan => {
                self.skip();
                BinaryOpKind::Gt
            }
            TokenKind::GreaterEqual => {
                self.skip();
                BinaryOpKind::GtEq
            }
            TokenKind::In => {
                self.skip();
                BinaryOpKind::In
            }
            TokenKind::Is => {
                // Check for "is not"
                self.skip(); // consume 'is'
                if self.current_token().kind() == &TokenKind::Not {
                    self.skip(); // consume 'not'
                    BinaryOpKind::IsNot
                } else {
                    BinaryOpKind::Is
                }
            }
            TokenKind::Not => {
                // This must be "not in"
                self.skip(); // consume 'not'
                if self.current_token().kind() == &TokenKind::In {
                    self.skip(); // consume 'in'
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
                self.skip();
                BinaryOpKind::And
            }
            TokenKind::Or => {
                self.skip();
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
