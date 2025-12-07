//! Control flow statement parsing (if, while, for, break, continue, return).

use typhon_ast::nodes::{
    AnyNode,
    AsyncForStmt,
    BreakStmt,
    ContinueStmt,
    ForStmt,
    IfStmt,
    NodeID,
    NodeKind,
    ReturnStmt,
    WhileStmt,
};
use typhon_source::types::Span;

use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::parser::context::{Context, ContextType};

impl Parser<'_> {
    /// Parse an async for statement (e.g. `async for target in iterable: ... [else: ...]`).
    ///
    /// Handles Python's async for loop which is used inside async functions.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// async_for_stmt: 'async' 'for' target 'in' iter ':' block ['else' ':' block]
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// async for item in async_iterable:
    ///     await process(item)
    /// ```
    ///
    /// With optional else clause:
    ///
    /// ```python
    /// async for item in async_generator():
    ///     if item.is_complete():
    ///         break
    /// else:
    ///     print("Completed without breaking")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `for` keyword is missing after `async`
    /// - The `in` keyword is missing between target and iterable
    /// - The colon after the iterable expression is missing
    /// - The target expression is invalid
    /// - The iterable expression is invalid
    /// - The block body is malformed (indentation errors, etc.)
    /// - The optional else clause is malformed
    pub(super) fn parse_async_for_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'async' token
        self.skip();

        // Consume the 'for' token
        self.expect(TokenKind::For)?;

        // Create a context for the async for statement
        self.context_stack.push(Context::new(
            ContextType::Loop,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the target expression (stops at 'in' keyword)
        let target = self.parse_for_target()?;

        // Expect the 'in' keyword
        self.expect(TokenKind::In)?;

        // Parse the iterable expression
        let iter = self.parse_expression()?;

        // Parse the for body
        let body = self.parse_block()?;

        // Parse optional else branch
        let else_body = if self.consume(TokenKind::Else).is_ok() {
            let body = self.parse_block()?;

            // Consume dedent token after the else body
            if self.check(TokenKind::Dedent) {
                self.skip();
            }

            Some(body)
        } else {
            None
        };

        // Get the end position
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
            _ => {
                if body.is_empty() {
                    // Fallback to iter
                    self.get_node_span(iter)?.end
                } else {
                    let last_stmt = body.last().unwrap();

                    self.get_node_span(*last_stmt)?.end
                }
            }
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create the AsyncForStmt node (with or without an else body)
        let mut async_for_stmt =
            AsyncForStmt::new(NodeID::placeholder(), target, iter, body.clone(), span);

        if let Some(else_stmts) = else_body.clone() {
            async_for_stmt = async_for_stmt.with_else_body(else_stmts);
        }

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AsyncForStmt(async_for_stmt), span);

        // Set parent-child relationships
        self.set_parent(target, node_id);
        self.set_parent(iter, node_id);

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        // Pop the async for context
        drop(self.context_stack.pop());

        Ok(node_id)
    }

    /// Parse a break statement.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// break_stmt: 'break'
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// for i in range(10):
    ///     if i == 5:
    ///         break
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if the statement terminator (newline or semicolon) is missing.
    pub(super) fn parse_break_statement(&mut self) -> ParseResult<NodeID> {
        // Get the span of the 'break' token
        let token = self.current_token().clone();
        let span = self.token_to_span(&token);

        // Consume the 'break' token
        self.skip();

        // Create the BreakStmt node
        let break_stmt = BreakStmt::new(NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::BreakStmt(break_stmt), span);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a continue statement.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// continue_stmt: 'continue'
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// for i in range(10):
    ///     if i % 2 == 0:
    ///         continue
    ///     print(i)  # Only prints odd numbers
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if the statement terminator (newline or semicolon) is missing.
    pub(super) fn parse_continue_statement(&mut self) -> ParseResult<NodeID> {
        // Get the span of the 'continue' token
        let token = self.current_token().clone();
        let span = self.token_to_span(&token);

        // Consume the 'continue' token
        self.skip();

        // Create the ContinueStmt node
        let continue_stmt = ContinueStmt::new(NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::ContinueStmt(continue_stmt), span);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a for statement (e.g. `for target in iterable: ... [else: ...]`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// for_stmt: 'for' target 'in' iter ':' block ['else' ':' block]
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// for x in range(10):
    ///     print(x)
    /// ```
    ///
    /// With tuple unpacking:
    ///
    /// ```python
    /// for key, value in dict.items():
    ///     print(f"{key}: {value}")
    /// ```
    ///
    /// With optional else clause:
    ///
    /// ```python
    /// for item in items:
    ///     if item.is_valid():
    ///         break
    /// else:
    ///     print("No valid items found")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `in` keyword is missing between target and iterable
    /// - The colon after the iterable expression is missing
    /// - The target expression is invalid
    /// - The iterable expression is invalid
    /// - The block body is malformed (indentation errors, etc.)
    /// - The optional else clause is malformed
    pub(super) fn parse_for_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'for' token
        self.skip();

        // Create a context for the for statement
        self.context_stack.push(Context::new(
            ContextType::Loop,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the target expression (stops at 'in' keyword)
        let target = self.parse_for_target()?;

        // Expect the 'in' keyword
        self.expect(TokenKind::In)?;

        // Parse the iterable expression
        let iter = self.parse_expression()?;

        // Parse the for body
        let body = self.parse_block()?;

        // Parse optional else branch
        let else_body = if self.consume(TokenKind::Else).is_ok() {
            let body = self.parse_block()?;

            // Consume dedent token after the else body
            if self.check(TokenKind::Dedent) {
                self.skip();
            }

            Some(body)
        } else {
            None
        };

        // Get the end position
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
            _ => {
                if body.is_empty() {
                    // Fallback to iter
                    self.get_node_span(iter)?.end
                } else {
                    let last_stmt = body.last().unwrap();

                    self.get_node_span(*last_stmt)?.end
                }
            }
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create the For node (with or without an else body)
        let mut for_stmt = ForStmt::new(NodeID::placeholder(), target, iter, body.clone(), span);

        if let Some(else_stmts) = else_body.clone() {
            for_stmt = for_stmt.with_else_body(else_stmts);
        }

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::ForStmt(for_stmt), span);

        // Set parent-child relationships
        self.set_parent(target, node_id);
        self.set_parent(iter, node_id);

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        // Pop the for context
        drop(self.context_stack.pop());

        Ok(node_id)
    }

    /// Parse an if statement (e.g. `if condition: ... elif condition: ... else: ...`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// if_stmt: 'if' condition ':' block ('elif' condition ':' block)* ['else' ':' block]
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// if x > 0:
    ///     print("positive")
    /// ```
    ///
    /// With elif and else:
    ///
    /// ```python
    /// if x > 0:
    ///     print("positive")
    /// elif x < 0:
    ///     print("negative")
    /// else:
    ///     print("zero")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The condition expression is invalid
    /// - The colon after the condition is missing
    /// - The block body is malformed (indentation errors, etc.)
    /// - Any elif condition or block is malformed
    /// - The optional else block is malformed
    pub(super) fn parse_if_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'if' token
        self.skip();

        // Create a context for the if statement
        self.context_stack.push(Context::new(
            ContextType::Conditional,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the condition expression
        let condition = self.parse_expression()?;

        // Parse the if body
        let body = self.parse_block()?;

        // Consume dedent token after the if body
        self.skip_if(&[TokenKind::Dedent]);

        // Skip any blank lines between if body and elif/else
        self.skip_newlines();

        // Parse elif branches
        let mut elif_branches = Vec::new();
        while self.check(TokenKind::Elif) {
            self.skip(); // Consume 'elif'

            let elif_condition = self.parse_expression()?;
            let elif_body = self.parse_block()?;

            elif_branches.push((elif_condition, elif_body));

            // Consume dedent token after each elif body
            self.skip_if(&[TokenKind::Dedent]);

            // Skip any blank lines between elif body and next elif/else
            self.skip_newlines();
        }

        // Parse optional else branch
        let else_body = if self.consume(TokenKind::Else).is_ok() {
            let body = self.parse_block()?;

            // Consume dedent token after the else body
            self.skip_if(&[TokenKind::Dedent]);

            Some(body)
        } else {
            None
        };

        // Get the end position (end of the entire if statement)
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
            _ if !elif_branches.is_empty() => {
                let (_, stmts) = elif_branches.last().unwrap();
                if stmts.is_empty() {
                    // Fallback to body
                    if body.is_empty() {
                        // Fallback to condition
                        self.get_node_span(condition)?.end
                    } else {
                        let last_stmt = body.last().unwrap();

                        self.get_node_span(*last_stmt)?.end
                    }
                } else {
                    let last_stmt = stmts.last().unwrap();

                    self.get_node_span(*last_stmt)?.end
                }
            }
            _ => {
                if body.is_empty() {
                    // Fallback to condition
                    self.get_node_span(condition)?.end
                } else {
                    let last_stmt = body.last().unwrap();

                    self.get_node_span(*last_stmt)?.end
                }
            }
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an If node
        let if_stmt = IfStmt::new(
            condition,
            body.clone(),
            elif_branches.clone(),
            else_body.clone(),
            NodeID::placeholder(),
            span,
        );

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::IfStmt(if_stmt), span);

        // Set parent-child relationships
        self.set_parent(condition, node_id);

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        for (cond, stmts) in &elif_branches {
            self.set_parent(*cond, node_id);
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        // Pop the if context
        drop(self.context_stack.pop());

        Ok(node_id)
    }

    /// Parse a return statement (e.g. `return [value]`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// return_stmt: 'return' [expression]
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// def get_value():
    ///     return 42
    /// ```
    ///
    /// Without a value:
    ///
    /// ```python
    /// def early_exit():
    ///     if condition:
    ///         return
    ///     # more code...
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The return value expression is invalid (if present)
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_return_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'return' token
        self.skip();

        // Parse optional return value
        let value = if self.check(TokenKind::Newline)
            || self.check(TokenKind::Semicolon)
            || self.check(TokenKind::EndOfFile)
        {
            // No return value
            None
        } else {
            // Parse the return value expression
            Some(self.parse_expression()?)
        };

        // Get the end position
        let end_pos = match value {
            Some(val) => self.get_node_span(val)?.end,
            None => start_pos + 6, // Length of "return"
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a ReturnStmt node
        let return_stmt = ReturnStmt::new(value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::ReturnStmt(return_stmt), span);

        // Set parent-child relationship if there's a value
        if let Some(val) = value {
            self.set_parent(val, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a while statement (e.g. `while condition: ... [else: ...]`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// while_stmt: 'while' condition ':' block ['else' ':' block]
    /// ```
    ///
    /// ## Examples
    ///
    /// ```python
    /// while x > 0:
    ///     x -= 1
    /// ```
    ///
    /// With optional else clause:
    ///
    /// ```python
    /// while x > 0:
    ///     if should_break():
    ///         break
    ///     x -= 1
    /// else:
    ///     print("Completed without breaking")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The condition expression is invalid
    /// - The colon after the condition is missing
    /// - The block body is malformed (indentation errors, etc.)
    /// - The optional else clause is malformed
    pub(super) fn parse_while_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'while' token
        self.skip();

        // Create a context for the while statement
        self.context_stack.push(Context::new(
            ContextType::Loop,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the condition expression
        let condition = self.parse_expression()?;

        // Parse the while body
        let body = self.parse_block()?;

        // Parse optional else branch
        let else_body = if self.consume(TokenKind::Else).is_ok() {
            let body = self.parse_block()?;

            // Consume dedent token after the else body
            if self.check(TokenKind::Dedent) {
                self.skip();
            }

            Some(body)
        } else {
            None
        };

        // Get the end position
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
            _ => {
                if body.is_empty() {
                    // Fallback to condition
                    self.get_node_span(condition)?.end
                } else {
                    let last_stmt = body.last().unwrap();

                    self.get_node_span(*last_stmt)?.end
                }
            }
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create the While node (with or without an else body)
        let mut while_stmt = WhileStmt::new(NodeID::placeholder(), condition, body.clone(), span); // Already cloning body here

        if let Some(else_stmts) = else_body.clone() {
            while_stmt = while_stmt.with_else_body(else_stmts);
        }

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::WhileStmt(while_stmt), span);

        // Set parent-child relationships
        self.set_parent(condition, node_id);

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        // Pop the while context
        drop(self.context_stack.pop());

        Ok(node_id)
    }
}
