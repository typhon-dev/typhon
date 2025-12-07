//! Context manager statement parsing (with, async with).

use typhon_ast::nodes::{AnyNode, AsyncWithStmt, NodeID, NodeKind, WithStmt};
use typhon_source::types::Span;

use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser<'_> {
    /// Parse an async with statement (e.g. `async with context_manager as var: ...`).
    ///
    /// Async with statements are used for asynchronous context managers that support
    /// `__aenter__` and `__aexit__` methods. They ensure proper resource management
    /// in async contexts.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// async_with_stmt: `async` `with` with_item (',' with_item)* ':' suite
    /// with_item: expression [`as` target]
    /// ```
    ///
    /// ## Examples
    ///
    /// Single async context manager:
    ///
    /// ```python
    /// async with lock:
    ///     await do_work()
    /// ```
    ///
    /// Async context manager with target binding:
    ///
    /// ```python
    /// async with aiofiles.open('file.txt') as f:
    ///     content = await f.read()
    /// ```
    ///
    /// Multiple async context managers:
    ///
    /// ```python
    /// async with lock1, lock2 as l2:
    ///     await critical_section()
    /// ```
    ///
    /// Nested resource management:
    ///
    /// ```python
    /// async with client.session() as session, session.get(url) as response:
    ///     data = await response.json()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `async` keyword is not followed by `with`
    /// - The context expression is invalid
    /// - The target expression after `as` is invalid
    /// - The colon `:` before the body is missing
    /// - The body block is malformed
    pub(super) fn parse_async_with_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'async' token
        self.skip();

        // Consume the 'with' token
        self.expect(TokenKind::With)?;

        // Parse one or more context managers
        let mut items = Vec::new();

        loop {
            // Parse the context expression
            let context_expr = self.parse_expression()?;

            // Check for 'as' to bind the context to a variable
            let target = if self.check(TokenKind::As) {
                self.skip(); // consume 'as'
                Some(self.parse_expression()?)
            } else {
                None
            };

            // Add the context manager to the list
            items.push((context_expr, target));

            // Check if there are more context managers
            if self.check(TokenKind::Comma) {
                self.skip(); // consume ','
                continue;
            }

            break;
        }

        // Parse the body of the async with statement
        let body = self.parse_block()?;

        // Calculate the end position (end of the body)
        let end_pos = if body.is_empty() {
            // If no body, use the position after the colon
            // Since previous_token() doesn't exist, use current token position instead
            self.current_token().span.end
        } else {
            // Otherwise, use the position after the last statement
            let last_stmt = body.last().unwrap();

            self.get_node_span(*last_stmt)?.end
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an AsyncWithStmt node
        let items_clone = items.clone();
        let async_with = AsyncWithStmt::new(items, body.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AsyncWithStmt(async_with), span);

        // Set parent-child relationships
        for (context_expr, target_opt) in &items_clone {
            self.set_parent(*context_expr, node_id);

            if let Some(target) = target_opt {
                self.set_parent(*target, node_id);
            }
        }

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }

    /// Parse a with statement (e.g. `with context_manager as var: ...`).
    ///
    /// With statements are used for context managers that support `__enter__` and
    /// `__exit__` methods. They ensure proper setup and cleanup of resources.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// with_stmt: `with` with_item (',' with_item)* ':' suite
    /// with_item: expression [`as` target]
    /// ```
    ///
    /// ## Examples
    ///
    /// File handling with automatic cleanup:
    ///
    /// ```python
    /// with open('file.txt') as f:
    ///     content = f.read()
    /// ```
    ///
    /// Multiple context managers:
    ///
    /// ```python
    /// with lock1, lock2:
    ///     critical_section()
    /// ```
    ///
    /// Context manager without target binding:
    ///
    /// ```python
    /// with timer():
    ///     expensive_operation()
    /// ```
    ///
    /// Database transaction management:
    ///
    /// ```python
    /// with db.transaction() as tx:
    ///     tx.execute("INSERT ...")
    ///     tx.commit()
    /// ```
    ///
    /// Nested resource management:
    ///
    /// ```python
    /// with open('input.txt') as infile, open('output.txt', 'w') as outfile:
    ///     outfile.write(infile.read().upper())
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The context expression is invalid
    /// - The target expression after `as` is invalid
    /// - The colon `:` before the body is missing
    /// - The body block is malformed
    pub(super) fn parse_with_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'with' token
        self.skip();

        // Parse one or more context managers
        let mut items = Vec::new();

        loop {
            // Parse the context expression
            let context_expr = self.parse_expression()?;

            // Check for 'as' to bind the context to a variable
            let target = if self.check(TokenKind::As) {
                self.skip(); // consume 'as'
                Some(self.parse_expression()?)
            } else {
                None
            };

            // Add the context manager to the list
            items.push((context_expr, target));

            // Check if there are more context managers
            if self.check(TokenKind::Comma) {
                self.skip(); // consume ','
                continue;
            }

            break;
        }

        // Parse the body of the with statement
        let body = self.parse_block()?;

        // Calculate the end position (end of the body)
        let end_pos = if body.is_empty() {
            // If no body, use the current token position as fallback
            self.current_token().span.start
        } else {
            // Otherwise, use the position after the last statement
            let last_stmt = body.last().unwrap();

            self.get_node_span(*last_stmt)?.end
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a WithStmt node
        let items_clone = items.clone();
        let with_stmt = WithStmt::new(items_clone, body.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::WithStmt(with_stmt), span);

        // Set parent-child relationships
        for (context_expr, target_opt) in &items {
            self.set_parent(*context_expr, node_id);

            if let Some(target) = target_opt {
                self.set_parent(*target, node_id);
            }
        }

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }
}
