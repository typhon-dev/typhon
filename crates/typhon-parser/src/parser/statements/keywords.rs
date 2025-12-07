//! Builtin keyword statement parsing (`assert`, `del`, `global`, `nonlocal`, `pass`, `expression_stmt`, `variable_decl`).
//!
//! This module contains parsers for Python's core statement keywords that match
//! the statement types defined in `typhon_ast::nodes::statements_core`.

use typhon_ast::nodes::{
    AnyNode,
    AssertStmt,
    DeleteStmt,
    ExpressionStmt,
    GlobalStmt,
    NodeID,
    NodeKind,
    NonlocalStmt,
    PassStmt,
    VariableExpr,
};
use typhon_source::types::Span;

use crate::diagnostics::{ParseError, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Parse an assert statement (e.g. `assert condition [, message]`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// assert_stmt: 'assert' expression [',' message]
    /// ```
    ///
    /// ## Examples
    ///
    /// Basic assertion:
    ///
    /// ```python
    /// assert x > 0
    /// ```
    ///
    /// With custom error message:
    ///
    /// ```python
    /// assert len(items) > 0, "List cannot be empty"
    /// ```
    ///
    /// Complex condition:
    ///
    /// ```python
    /// assert user.is_authenticated(), f"User {user.id} not authenticated"
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The condition expression is invalid
    /// - The message expression is invalid (if present)
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_assert_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'assert' token
        self.skip();

        // Parse the condition expression
        let condition = self.parse_expression()?;

        // Check for an optional message (after a comma)
        let message = if self.check(TokenKind::Comma) {
            self.skip();

            Some(self.parse_expression()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = match message {
            Some(_) | None => self.get_node_span(condition)?.end,
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an Assert node
        let assert = AssertStmt::new(condition, message, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::AssertStmt(assert), span);

        // Set parent-child relationship
        self.set_parent(condition, node_id);

        if let Some(msg) = message {
            self.set_parent(msg, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a delete statement (e.g. `del target`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// del_stmt: 'del' expression (',' expression)*
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete a variable:
    ///
    /// ```python
    /// del x
    /// ```
    ///
    /// Delete multiple targets:
    ///
    /// ```python
    /// del x, y, z
    /// ```
    ///
    /// Delete list element:
    ///
    /// ```python
    /// del items[0]
    /// ```
    ///
    /// Delete dictionary key:
    ///
    /// ```python
    /// del data["key"]
    /// ```
    ///
    /// Delete attribute:
    ///
    /// ```python
    /// del obj.attribute
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Any target expression is invalid
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_delete_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'del' token
        self.skip();

        // Parse one or more targets
        let mut targets = Vec::new();

        loop {
            // Parse the target expression
            let target = self.parse_expression()?;
            targets.push(target);

            // Check if there are more targets
            if self.check(TokenKind::Comma) {
                self.skip(); // consume ','
                continue;
            }

            break;
        }

        // Get the end position
        let end_pos = if targets.is_empty() {
            // Fallback to current token position
            self.current_token().span.start
        } else {
            let last_target = targets.last().unwrap();

            self.get_node_span(*last_target)?.end
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a DeleteStmt node
        let delete = DeleteStmt::new(targets.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::DeleteStmt(delete), span);

        // Set parent-child relationships
        for target in &targets {
            self.set_parent(*target, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an expression statement.
    ///
    /// An expression statement is any expression that appears as a standalone statement.
    /// This is typically used for function calls, method calls, or other expressions with
    /// side effects.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// expression_stmt: expression
    /// ```
    ///
    /// ## Examples
    ///
    /// Function call:
    ///
    /// ```python
    /// print("Hello, World!")
    /// ```
    ///
    /// Method call:
    ///
    /// ```python
    /// items.append(42)
    /// ```
    ///
    /// Standalone expression:
    ///
    /// ```python
    /// user.save()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The expression is invalid
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_expression_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the expression
        let expression = self.parse_expression()?;

        // Get the end position
        let end_pos = self.get_node_span(expression)?.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an ExpressionStmt node
        let expr_stmt = ExpressionStmt::new(expression, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::ExpressionStmt(expr_stmt), span);

        // Set parent-child relationship
        self.set_parent(expression, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a global statement (e.g. `global x, y, z`).
    ///
    /// The `global` keyword declares that variables should refer to the global scope,
    /// allowing functions to modify global variables.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// global_stmt: 'global' identifier (',' identifier)*
    /// ```
    ///
    /// ## Examples
    ///
    /// Single global variable:
    ///
    /// ```python
    /// counter = 0
    ///
    /// def increment():
    ///     global counter
    ///     counter += 1
    /// ```
    ///
    /// Multiple global variables:
    ///
    /// ```python
    /// x, y, z = 0, 0, 0
    ///
    /// def update():
    ///     global x, y, z
    ///     x, y, z = 1, 2, 3
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - No identifier is provided after `global`
    /// - Any identifier name is invalid
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_global_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'global' token
        self.skip();

        // Parse one or more names
        let mut variable_ids = Vec::new();

        loop {
            // Parse the name as an identifier
            if self.check(TokenKind::Identifier) {
                let name_token = self.current_token();
                let name = name_token.lexeme.to_string();
                let var_span = self.token_to_span(name_token);

                // Create a Variable node for each name
                let variable = VariableExpr::new(name, NodeID::placeholder(), var_span);

                // Add the variable node to the AST
                let var_node_id = self.ast.alloc_node(
                    NodeKind::Expression,
                    AnyNode::VariableExpr(variable),
                    var_span,
                );

                // Add the variable node ID to our list
                variable_ids.push(var_node_id);

                self.skip();
            } else {
                let token = self.current_token();
                let span: Span = token.span.clone().into();

                return Err(ParseError::unexpected_token(
                    token.kind,
                    vec![TokenKind::Identifier],
                    span.into(),
                ));
            }

            // Check if there are more names
            if self.check(TokenKind::Comma) {
                self.skip(); // consume ','
                continue;
            }

            break;
        }

        // Get the end position - use the last token position as end
        let end_pos = self.current_token().span.start;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create the global statement
        let global = GlobalStmt::new(variable_ids, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::GlobalStmt(global), span);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a nonlocal statement (e.g. `nonlocal x, y, z`).
    ///
    /// The `nonlocal` keyword declares that variables should refer to the nearest enclosing
    /// scope (not global), allowing nested functions to modify variables from outer functions.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// nonlocal_stmt: 'nonlocal' identifier (',' identifier)*
    /// ```
    ///
    /// ## Examples
    ///
    /// Single nonlocal variable:
    ///
    /// ```python
    /// def outer():
    ///     count = 0
    ///     def inner():
    ///         nonlocal count
    ///         count += 1
    ///     return inner
    /// ```
    ///
    /// Multiple nonlocal variables:
    ///
    /// ```python
    /// def outer():
    ///     x, y = 0, 0
    ///     def inner():
    ///         nonlocal x, y
    ///         x, y = 1, 2
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - No identifier is provided after `nonlocal`
    /// - Any identifier name is invalid
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_nonlocal_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'nonlocal' token
        self.skip();

        // Parse one or more names
        let mut variable_ids = Vec::new();

        loop {
            // Parse the name as an identifier
            if self.check(TokenKind::Identifier) {
                let name_token = self.current_token();
                let name = name_token.lexeme.to_string();
                let var_span = self.token_to_span(name_token);

                // Create a Variable node for each name
                let variable = VariableExpr::new(name, NodeID::placeholder(), var_span);

                // Add the variable node to the AST
                let var_node_id = self.ast.alloc_node(
                    NodeKind::Expression,
                    AnyNode::VariableExpr(variable),
                    var_span,
                );

                // Add the variable node ID to our list
                variable_ids.push(var_node_id);

                self.skip();
            } else {
                let token = self.current_token();
                let span: Span = token.span.clone().into();

                return Err(ParseError::unexpected_token(
                    token.kind,
                    vec![TokenKind::Identifier],
                    span.into(),
                ));
            }

            // Check if there are more names
            if self.check(TokenKind::Comma) {
                self.skip(); // consume ','
                continue;
            }

            break;
        }

        // Get the end position
        let end_pos = if variable_ids.is_empty() {
            // Fallback to current token position
            self.current_token().span.start
        } else {
            let last_var = variable_ids.last().unwrap();

            self.get_node_span(*last_var)?.end
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create the NonlocalStmt with the variable IDs
        let nonlocal = NonlocalStmt::new(variable_ids, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::NonlocalStmt(nonlocal), span);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a pass statement (e.g. `pass`).
    ///
    /// The `pass` statement is a null operation that does nothing. It's used as a placeholder
    /// where syntax requires a statement but no action is needed.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// pass_stmt: 'pass'
    /// ```
    ///
    /// ## Examples
    ///
    /// Empty function body:
    ///
    /// ```python
    /// def placeholder():
    ///     pass
    /// ```
    ///
    /// Empty class:
    ///
    /// ```python
    /// class EmptyClass:
    ///     pass
    /// ```
    ///
    /// Placeholder in conditional:
    ///
    /// ```python
    /// if condition:
    ///     pass  # TODO: implement later
    /// else:
    ///     handle_case()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if the statement terminator (newline or semicolon) is missing.
    pub(super) fn parse_pass_statement(&mut self) -> ParseResult<NodeID> {
        // Get the span of the 'pass' token
        let token = self.current_token().clone();
        let span = self.token_to_span(&token);

        // Consume the 'pass' token
        self.skip();

        // Create the Pass node
        let pass = PassStmt::new(NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::PassStmt(pass), span);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }
}
