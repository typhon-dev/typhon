//! Assignment statement parsing (assignment, augmented assignment, annotated assignment).

use typhon_ast::nodes::{
    AnyNode,
    AssignmentStmt,
    AugmentedAssignmentOp,
    AugmentedAssignmentStmt,
    NodeID,
    NodeKind,
};
use typhon_source::types::Span;

use crate::diagnostics::{ParseError, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::parser::context::{Context, ContextType};

impl Parser<'_> {
    /// Parse an annotated assignment statement (e.g. `self.x: type = value`).
    ///
    /// This handles annotated assignments with complex targets like attribute access.
    /// Unlike variable declarations, these can have targets other than simple identifiers.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// annotated_assignment: target ':' type_annotation '=' expression
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple annotated assignment:
    ///
    /// ```python
    /// x: int = 42
    /// ```
    ///
    /// Attribute with type annotation:
    ///
    /// ```python
    /// self.count: int = 0
    /// ```
    ///
    /// Complex type annotation:
    ///
    /// ```python
    /// items: list[str] = ["a", "b", "c"]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The target expression is invalid
    /// - The type annotation is missing or invalid after `:`
    /// - The assignment value is missing after `=`
    pub(super) fn parse_annotated_assignment_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the target expression (e.g., self.x, obj.attr, etc.)
        let target = self.parse_expression()?;

        // Expect a colon for type annotation
        self.expect(TokenKind::Colon)?;

        // Push type annotation context
        self.context_stack.push(Context::new(
            ContextType::TypeAnnotation,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the type annotation
        let type_annotation = self.parse_expression()?;

        // Pop type annotation context
        drop(self.context_stack.pop());

        // Expect the assignment operator
        self.expect(TokenKind::Assign)?;

        // Parse the value expression
        let value = self.parse_expression()?;

        // Get the end position
        let end_pos = self.get_node_span(value)?.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an AssignmentStmt node (we don't have a separate AnnotatedAssignmentStmt)
        // The type information is implicit in having both target and value
        let assignment = AssignmentStmt::new(target, value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AssignmentStmt(assignment), span);

        // Set parent-child relationships
        self.set_parent(target, node_id);
        self.set_parent(type_annotation, node_id);
        self.set_parent(value, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an assignment statement (e.g. `target = value`).
    ///
    /// Supports:
    /// - Simple assignment: `x = value`
    /// - Tuple unpacking: `a, b = value`
    /// - Implicit tuple on right: `x = a, b, c`
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// assignment_stmt: target '=' expression
    ///                | target (',' target)* '=' expression
    ///                | target '=' expression (',' expression)*
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple assignment:
    ///
    /// ```python
    /// x = 42
    /// ```
    ///
    /// Tuple unpacking:
    ///
    /// ```python
    /// a, b = (1, 2)
    /// x, y, z = get_coordinates()
    /// ```
    ///
    /// Implicit tuple on right (no parentheses):
    ///
    /// ```python
    /// point = 10, 20
    /// values = 1, 2, 3
    /// ```
    ///
    /// Combined unpacking:
    ///
    /// ```python
    /// first, *rest = items
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The target expression is invalid
    /// - The assignment operator `=` is missing
    /// - The value expression is invalid
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_assignment_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the first target expression
        let mut target = self.parse_expression()?;

        // Check for tuple unpacking on the left side (multiple targets)
        // Example: `a, b = expression` or `scalar, unit = str_match.groups()`
        if self.check(TokenKind::Comma) {
            // We have comma-separated targets (tuple unpacking)
            let mut targets = vec![target];

            while self.check(TokenKind::Comma) {
                self.skip(); // consume comma

                // Check if we've reached the assignment operator
                if self.check(TokenKind::Assign) {
                    break;
                }

                // Parse the next target
                let next_target = self.parse_expression()?;
                targets.push(next_target);
            }

            // Create an implicit tuple from the targets
            target = self.create_tuple_literal(targets);
        }

        // Expect the assignment operator
        self.expect(TokenKind::Assign)?;

        // Parse the value expression
        let mut value = self.parse_expression()?;

        // Check for implicit tuple creation on the right side (comma-separated values without parentheses)
        // Example: __slots__ = 'a', 'b', 'c'
        if self.check(TokenKind::Comma) {
            // We have a comma, so this is an implicit tuple
            let mut elements = vec![value];

            while self.check(TokenKind::Comma) {
                self.skip(); // consume comma

                // Check if we're at the end of the statement (trailing comma)
                if self.matches(&[TokenKind::Newline, TokenKind::Semicolon, TokenKind::EndOfFile]) {
                    break;
                }

                // Parse the next element
                let element = self.parse_expression()?;
                elements.push(element);
            }

            // Create an implicit tuple from the elements
            value = self.create_tuple_literal(elements);
        }

        // Get the end position
        let end_pos = self.get_node_span(value)?.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an AssignmentStmt node
        let assignment = AssignmentStmt::new(target, value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AssignmentStmt(assignment), span);

        // Set parent-child relationships
        self.set_parent(target, node_id);
        self.set_parent(value, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an augmented assignment statement (e.g. `target += value`).
    ///
    /// Augmented assignments combine an operation with assignment,
    /// modifying a variable in place.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// augmented_assignment: target augop expression
    /// augop: '+=' | '-=' | '*=' | '/=' | '//=' | '%=' | '**='
    ///      | '<<=' | '>>=' | '&=' | '|=' | '^=' | '@='
    /// ```
    ///
    /// ## Examples
    ///
    /// Arithmetic augmented assignment:
    ///
    /// ```python
    /// count += 1
    /// total -= discount
    /// value *= 2
    /// ```
    ///
    /// Bitwise augmented assignment:
    ///
    /// ```python
    /// flags |= READONLY
    /// mask &= 0xFF
    /// bits ^= 0x10
    /// ```
    ///
    /// Sequence operations:
    ///
    /// ```python
    /// items += [new_item]
    /// text += " more"
    /// ```
    ///
    /// Matrix multiplication (NumPy):
    ///
    /// ```python
    /// matrix @= transform
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The target expression is invalid
    /// - The augmented assignment operator is unrecognized
    /// - The value expression is invalid
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_augmented_assignment_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the target expression
        let target = self.parse_expression()?;

        // Get the augmented assignment operator
        let op_token = self.current_token().clone();

        // Convert token to operator string
        let op = match op_token.kind {
            TokenKind::PlusEqual => AugmentedAssignmentOp::Add,
            TokenKind::MinusEqual => AugmentedAssignmentOp::Sub,
            TokenKind::StarEqual => AugmentedAssignmentOp::Mul,
            TokenKind::SlashEqual => AugmentedAssignmentOp::Div,
            TokenKind::DoubleSlashEqual => AugmentedAssignmentOp::FloorDiv,
            TokenKind::PercentEqual => AugmentedAssignmentOp::Mod,
            TokenKind::DoubleStarEqual => AugmentedAssignmentOp::Pow,
            TokenKind::LeftShiftEqual => AugmentedAssignmentOp::LShift,
            TokenKind::RightShiftEqual => AugmentedAssignmentOp::RShift,
            TokenKind::AmpersandEqual => AugmentedAssignmentOp::BitAnd,
            TokenKind::PipeEqual => AugmentedAssignmentOp::BitOr,
            TokenKind::CaretEqual => AugmentedAssignmentOp::BitXor,
            TokenKind::AtEqual => AugmentedAssignmentOp::MatMul,
            _ => {
                let span = self.create_source_span(op_token.span.start, op_token.span.end);
                return Err(ParseError::unexpected_token(
                    op_token.kind,
                    vec![TokenKind::PlusEqual, TokenKind::MinusEqual], // Example of expected tokens
                    span,
                ));
            }
        };

        // Consume the operator token
        self.skip();

        // Parse the value expression
        let value = self.parse_expression()?;

        // Get the end position
        let end_pos = self.get_node_span(value)?.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an AugmentedAssignmentStmt node
        let stmt = AugmentedAssignmentStmt::new(target, op, value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AugmentedAssignmentStmt(stmt), span);

        // Set parent-child relationships
        self.set_parent(target, node_id);
        self.set_parent(value, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }
}
