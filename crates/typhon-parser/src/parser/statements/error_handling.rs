//! Error handling statement parsing (try, except, raise).

use typhon_ast::nodes::{
    AnyNode,
    ExceptHandler,
    NodeID,
    NodeKind,
    RaiseStmt,
    TryStmt,
    VariableExpr,
};
use typhon_source::types::Span;

use crate::diagnostics::{ParseError, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Parse an except handler (e.g. `except Exception as e: ...` or `except: ...`).
    ///
    /// Handles:
    ///
    /// - Bare except: `except:`
    /// - Typed except: `except Exception:`
    /// - Named except: `except Exception as e:`
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// except_handler: 'except' [expression ['as' identifier]] ':' block
    /// ```
    ///
    /// ## Examples
    ///
    /// Bare except (catches all exceptions):
    ///
    /// ```python
    /// try:
    ///     risky_operation()
    /// except:
    ///     print("Something went wrong")
    /// ```
    ///
    /// Typed except:
    ///
    /// ```python
    /// try:
    ///     int("not a number")
    /// except ValueError:
    ///     print("Invalid value")
    /// ```
    ///
    /// Named except (access exception object):
    ///
    /// ```python
    /// try:
    ///     risky_operation()
    /// except ValueError as e:
    ///     print(f"Error: {e}")
    /// ```
    ///
    /// Multiple exception types:
    ///
    /// ```python
    /// try:
    ///     operation()
    /// except (ValueError, TypeError) as e:
    ///     print(f"Type or value error: {e}")
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The except keyword is missing
    /// - The exception type is invalid (if present)
    /// - The variable name is invalid (if present)
    /// - The handler body is missing or invalid
    pub(super) fn parse_except_handler(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'except' token
        self.expect(TokenKind::Except)?;

        // Parse optional exception type
        let exception_type = if self.check(TokenKind::Colon) {
            // Bare except:
            None
        } else {
            // Parse the exception type expression
            Some(self.parse_expression()?)
        };

        // Parse optional 'as name' binding
        let name = if self.consume(TokenKind::As).is_ok() {
            // Parse the variable name
            if self.check(TokenKind::Identifier) {
                let name_token = self.current_token();
                let name_str = name_token.lexeme.to_string();
                let var_span = self.token_to_span(name_token);

                // Create a VariableExpr node
                let variable = VariableExpr::new(name_str, NodeID::placeholder(), var_span);

                // Add the variable node to the AST
                let var_node_id = self.ast.alloc_node(
                    NodeKind::Expression,
                    AnyNode::VariableExpr(variable),
                    var_span,
                );

                self.skip(); // Consume the identifier

                Some(var_node_id)
            } else {
                let token = self.current_token();
                let span = self.token_to_span(token);

                return Err(ParseError::unexpected_token(
                    token.kind,
                    vec![TokenKind::Identifier],
                    span.into(),
                ));
            }
        } else {
            None
        };

        // Parse the handler body
        let body = self.parse_block()?;

        // Calculate the end position
        let end_pos = if body.is_empty() {
            self.current_token().span.start
        } else {
            let last_stmt = body.last().unwrap();

            self.get_node_span(*last_stmt)?.end
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an ExceptHandler node
        let mut handler = ExceptHandler::new(NodeID::placeholder(), body.clone(), span);

        if let Some(exc_type) = exception_type {
            handler = handler.with_exception_type(exc_type);
        }

        if let Some(name_id) = name {
            handler = handler.with_name(name_id);
        }

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::ExceptHandler(handler), span);

        // Set parent-child relationships
        if let Some(exc_type) = exception_type {
            self.set_parent(exc_type, node_id);
        }

        if let Some(name_id) = name {
            self.set_parent(name_id, node_id);
        }

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }

    /// Parse a raise statement (e.g. `raise Exception` or `raise Exception from cause`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// raise_stmt: 'raise' [expression ['from' expression]]
    /// ```
    ///
    /// ## Examples
    ///
    /// Basic raise:
    ///
    /// ```python
    /// raise ValueError("Invalid input")
    /// ```
    ///
    /// Bare raise (re-raises current exception in except handler):
    ///
    /// ```python
    /// try:
    ///     risky_operation()
    /// except Exception:
    ///     log_error()
    ///     raise  # Re-raises the caught exception
    /// ```
    ///
    /// Raise with cause (exception chaining):
    ///
    /// ```python
    /// try:
    ///     process_data()
    /// except ValueError as e:
    ///     raise ProcessingError("Failed to process") from e
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The exception expression is invalid (if present)
    /// - The cause expression after `from` is invalid (if present)
    /// - The statement terminator (newline or semicolon) is missing
    pub(super) fn parse_raise_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'raise' token
        self.skip();

        // Parse optional exception expression
        let exception =
            if self.matches(&[TokenKind::Newline, TokenKind::Semicolon, TokenKind::EndOfFile]) {
                // Bare raise (re-raises current exception)
                None
            } else {
                // Parse the exception expression
                Some(self.parse_expression()?)
            };

        // Parse optional 'from' clause
        let cause = if self.consume(TokenKind::From).is_ok() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = if let Some(c) = cause {
            self.get_node_span(c)?.end
        } else if let Some(exc) = exception {
            self.get_node_span(exc)?.end
        } else {
            start_pos + 5 // Length of "raise"
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a `RaiseStmt` node
        let raise_stmt =
            RaiseStmt::new(NodeID::placeholder(), span).with_cause(cause).with_exception(exception);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::RaiseStmt(raise_stmt), span);

        // Set parent-child relationships
        if let Some(exc) = exception {
            self.set_parent(exc, node_id);
        }

        if let Some(cause) = cause {
            self.set_parent(cause, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a try statement (e.g. `try: ... except: ... else: ... finally: ...`).
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// try_stmt: 'try' ':' block
    ///           except_handler+
    ///           ['else' ':' block]
    ///           ['finally' ':' block]
    /// ```
    ///
    /// ## Examples
    ///
    /// Basic try-except:
    ///
    /// ```python
    /// try:
    ///     risky_operation()
    /// except Exception:
    ///     handle_error()
    /// ```
    ///
    /// Try-except-else-finally:
    ///
    /// ```python
    /// try:
    ///     file = open("data.txt")
    ///     process(file)
    /// except FileNotFoundError:
    ///     print("File not found")
    /// except PermissionError:
    ///     print("Permission denied")
    /// else:
    ///     print("Success!")
    /// finally:
    ///     file.close()
    /// ```
    ///
    /// Multiple exception handlers:
    ///
    /// ```python
    /// try:
    ///     result = divide(a, b)
    /// except ZeroDivisionError:
    ///     result = 0
    /// except TypeError as e:
    ///     print(f"Invalid types: {e}")
    ///     result = None
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The try block is missing or invalid
    /// - No except handlers are provided
    /// - Any except handler is malformed
    /// - The else or finally blocks are malformed
    pub(super) fn parse_try_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'try' token
        self.skip();

        // Parse the try body
        let body = self.parse_block()?;

        // Consume dedent token after the try body
        self.skip_if(&[TokenKind::Dedent]);

        // Skip any blank lines
        self.skip_newlines();

        // Parse except handlers (at least one is required)
        let mut handlers = Vec::new();

        if !self.check(TokenKind::Except) {
            let token = self.current_token();
            let span = self.token_to_span(token);

            return Err(ParseError::unexpected_token(
                token.kind,
                vec![TokenKind::Except],
                span.into(),
            ));
        }

        while self.check(TokenKind::Except) {
            let handler = self.parse_except_handler()?;
            handlers.push(handler);

            // Consume dedent token after each except body
            self.skip_if(&[TokenKind::Dedent]);

            // Skip any blank lines
            self.skip_newlines();
        }

        // Parse optional else branch
        let else_body = if self.consume(TokenKind::Else).is_ok() {
            let body = self.parse_block()?;

            // Consume dedent token after the else body
            self.skip_if(&[TokenKind::Dedent]);

            // Skip any blank lines
            self.skip_newlines();

            Some(body)
        } else {
            None
        };

        // Parse optional finally branch
        let finally_body = if self.consume(TokenKind::Finally).is_ok() {
            let body = self.parse_block()?;

            // Consume dedent token after the finally body
            self.skip_if(&[TokenKind::Dedent]);

            Some(body)
        } else {
            None
        };

        // Get the end position
        let end_pos = if let Some(ref finally_stmts) = finally_body {
            if finally_stmts.is_empty() {
                self.current_token().span.start
            } else {
                let last_stmt = finally_stmts.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
        } else if let Some(ref else_stmts) = else_body {
            if else_stmts.is_empty() {
                self.current_token().span.start
            } else {
                let last_stmt = else_stmts.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
        } else if !handlers.is_empty() {
            // Get the end of the last handler
            let last_handler = handlers.last().unwrap();

            self.get_node_span(*last_handler)?.end
        } else {
            // Fallback to body
            if body.is_empty() {
                self.current_token().span.start
            } else {
                let last_stmt = body.last().unwrap();

                self.get_node_span(*last_stmt)?.end
            }
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a TryStmt node
        let mut try_stmt =
            TryStmt::new(NodeID::placeholder(), body.clone(), handlers.clone(), span);

        if let Some(else_stmts) = else_body.clone() {
            try_stmt = try_stmt.with_else_body(else_stmts);
        }

        if let Some(finally_stmts) = finally_body.clone() {
            try_stmt = try_stmt.with_finally_body(finally_stmts);
        }

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::TryStmt(try_stmt), span);

        // Set parent-child relationships
        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        for handler in &handlers {
            self.set_parent(*handler, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        if let Some(stmts) = &finally_body {
            for stmt in stmts {
                self.set_parent(*stmt, node_id);
            }
        }

        Ok(node_id)
    }
}
