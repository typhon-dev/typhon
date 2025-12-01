//! Statement parsing for the Typhon programming language.
//!
//! This module handles parsing statements of all types,
//! including simple statements, control flow, and compound statements.

use typhon_ast::nodes::{
    AnyNode,
    AssertStmt,
    AssignmentStmt,
    AsyncForStmt,
    AsyncWithStmt,
    AugmentedAssignmentOp,
    AugmentedAssignmentStmt,
    DeleteStmt,
    ExpressionStmt,
    ForStmt,
    GlobalStmt,
    IfStmt,
    NodeID,
    NodeKind,
    NonlocalStmt,
    PassStmt,
    VariableIdent,
    WhileStmt,
    WithStmt,
};
use typhon_source::types::Span;

use super::Parser;
use crate::diagnostics::{ParseError, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::context::{Context, ContextType};

impl Parser<'_> {
    /// Parse a statement.
    ///
    /// This is the main entry point for parsing a statement.
    ///
    /// ## Errors
    ///
    /// Returns a `ParseError` if:
    ///
    /// - The statement syntax is invalid
    /// - An unexpected token is encountered
    /// - Required tokens (like `:`, `=`, keywords) are missing
    /// - Indentation is incorrect
    pub fn parse_statement(&mut self) -> ParseResult<NodeID> {
        // Check for indentation changes
        self.handle_indentation()?;

        let token_kind = self.current_token().kind;

        // Parse the statement based on the current token
        match token_kind {
            // Import statements
            TokenKind::From => self.parse_from_import_statement(),
            TokenKind::Import => self.parse_import_statement(),

            // Simple statements
            TokenKind::Assert => self.parse_assert_statement(),
            TokenKind::Del => self.parse_delete_statement(),
            TokenKind::Global => self.parse_global_statement(),
            TokenKind::Nonlocal => self.parse_nonlocal_statement(),
            TokenKind::Pass => self.parse_pass_statement(),

            // Control flow statements
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Match => self.parse_match_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::With => self.parse_with_statement(),
            TokenKind::Async if self.peek_token().kind == TokenKind::For => {
                self.parse_async_for_statement()
            }
            TokenKind::Async if self.peek_token().kind == TokenKind::With => {
                self.parse_async_with_statement()
            }

            // Assignment statements
            // Look ahead for an equals sign to detect assignment
            _ if self.is_assignment() => self.parse_assignment_statement(),
            // Look ahead for augmented assignment operators
            _ if self.is_augmented_assignment() => self.parse_augmented_assignment_statement(),

            // Expression statements (default case)
            _ => self.parse_expression_statement(),
        }
    }

    /// Parse an augmented assignment statement (e.g. `target += value`).
    fn parse_augmented_assignment_statement(&mut self) -> ParseResult<NodeID> {
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
        let _ = self.advance();

        // Parse the value expression
        let value = self.parse_expression()?;

        // Get the end position
        let end_pos = self.ast.get_node(value).unwrap().span.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an AugmentedAssignmentStmt node
        let stmt = AugmentedAssignmentStmt::new(target, op, value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AugmentedAssignmentStmt(stmt), span);

        // Set parent-child relationships
        let _ = self.ast.set_parent(target, node_id);
        let _ = self.ast.set_parent(value, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an async with statement (e.g. `async with context_manager as var: ...`).
    fn parse_async_with_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'async' token
        let _ = self.advance();

        // Consume the 'with' token
        self.expect(TokenKind::With)?;

        // Parse one or more context managers
        let mut items = Vec::new();

        loop {
            // Parse the context expression
            let context_expr = self.parse_expression()?;

            // Check for 'as' to bind the context to a variable
            let target = if self.check(TokenKind::As) {
                let _ = self.advance(); // consume 'as'
                Some(self.parse_expression()?)
            } else {
                None
            };

            // Add the context manager to the list
            items.push((context_expr, target));

            // Check if there are more context managers
            if self.check(TokenKind::Comma) {
                let _ = self.advance(); // consume ','
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
            self.ast.get_node(*last_stmt).unwrap().span.end
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
            let _ = self.ast.set_parent(*context_expr, node_id);

            if let Some(target) = target_opt {
                let _ = self.ast.set_parent(*target, node_id);
            }
        }

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }

    /// Parse a delete statement (e.g. `del target`).
    fn parse_delete_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'del' token
        let _ = self.advance();

        // Parse one or more targets
        let mut targets = Vec::new();

        loop {
            // Parse the target expression
            let target = self.parse_expression()?;
            targets.push(target);

            // Check if there are more targets
            if self.check(TokenKind::Comma) {
                let _ = self.advance(); // consume ','
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
            self.ast.get_node(*last_target).unwrap().span.end
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a DeleteStmt node
        let delete = DeleteStmt::new(targets.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::DeleteStmt(delete), span);

        // Set parent-child relationships
        for target in &targets {
            let _ = self.ast.set_parent(*target, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a global statement (e.g. `global x, y, z`).
    fn parse_global_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'global' token
        let _ = self.advance();

        // Parse one or more names
        let mut variable_ids = Vec::new();

        loop {
            // Parse the name as an identifier
            if self.check(TokenKind::Identifier) {
                let name_token = self.current_token();
                let name = name_token.lexeme.to_string();
                let var_span = self.token_to_span(name_token);

                // Create a Variable node for each name
                let variable = VariableIdent::new(name, NodeID::placeholder(), var_span);

                // Add the variable node to the AST
                let var_node_id = self.ast.alloc_node(
                    NodeKind::Expression,
                    AnyNode::VariableIdent(variable),
                    var_span,
                );

                // Add the variable node ID to our list
                variable_ids.push(var_node_id);

                let _ = self.advance();
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
                let _ = self.advance(); // consume ','
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

    /// Parse an if statement (e.g. `if condition: ... elif condition: ... else: ...`).
    fn parse_if_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'if' token
        let _ = self.advance();

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

        // Parse elif branches
        let mut elif_branches = Vec::new();
        while self.check(TokenKind::Elif) {
            let _ = self.advance(); // Consume 'elif'

            let elif_condition = self.parse_expression()?;
            let elif_body = self.parse_block()?;

            elif_branches.push((elif_condition, elif_body));
        }

        // Parse optional else branch
        let else_body = if self.check(TokenKind::Else) {
            let _ = self.advance(); // Consume 'else'
            Some(self.parse_block()?)
        } else {
            None
        };

        // Get the end position (end of the entire if statement)
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();
                self.ast.get_node(*last_stmt).unwrap().span.end
            }
            _ if !elif_branches.is_empty() => {
                let (_, stmts) = elif_branches.last().unwrap();
                if stmts.is_empty() {
                    // Fallback to body
                    if body.is_empty() {
                        // Fallback to condition
                        self.ast.get_node(condition).unwrap().span.end
                    } else {
                        let last_stmt = body.last().unwrap();
                        self.ast.get_node(*last_stmt).unwrap().span.end
                    }
                } else {
                    let last_stmt = stmts.last().unwrap();
                    self.ast.get_node(*last_stmt).unwrap().span.end
                }
            }
            _ => {
                if body.is_empty() {
                    // Fallback to condition
                    self.ast.get_node(condition).unwrap().span.end
                } else {
                    let last_stmt = body.last().unwrap();
                    self.ast.get_node(*last_stmt).unwrap().span.end
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
        let _ = self.ast.set_parent(condition, node_id);

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        for (cond, stmts) in &elif_branches {
            let _ = self.ast.set_parent(*cond, node_id);
            for stmt in stmts {
                let _ = self.ast.set_parent(*stmt, node_id);
            }
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                let _ = self.ast.set_parent(*stmt, node_id);
            }
        }

        // Pop the if context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }

    /// Parse a nonlocal statement (e.g. `nonlocal x, y, z`).
    fn parse_nonlocal_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'nonlocal' token
        let _ = self.advance();

        // Parse one or more names
        let mut variable_ids = Vec::new();

        loop {
            // Parse the name as an identifier
            if self.check(TokenKind::Identifier) {
                let name_token = self.current_token();
                let name = name_token.lexeme.to_string();
                let var_span = self.token_to_span(name_token);

                // Create a Variable node for each name
                let variable = VariableIdent::new(name, NodeID::placeholder(), var_span);

                // Add the variable node to the AST
                let var_node_id = self.ast.alloc_node(
                    NodeKind::Expression,
                    AnyNode::VariableIdent(variable),
                    var_span,
                );

                // Add the variable node ID to our list
                variable_ids.push(var_node_id);

                let _ = self.advance();
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
                let _ = self.advance(); // consume ','
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
            self.ast.get_node(*last_var).unwrap().span.end
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

    /// Parse a with statement (e.g. `with context_manager as var: ...`).
    fn parse_with_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'with' token
        let _ = self.advance();

        // Parse one or more context managers
        let mut items = Vec::new();

        loop {
            // Parse the context expression
            let context_expr = self.parse_expression()?;

            // Check for 'as' to bind the context to a variable
            let target = if self.check(TokenKind::As) {
                let _ = self.advance(); // consume 'as'
                Some(self.parse_expression()?)
            } else {
                None
            };

            // Add the context manager to the list
            items.push((context_expr, target));

            // Check if there are more context managers
            if self.check(TokenKind::Comma) {
                let _ = self.advance(); // consume ','
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
            self.ast.get_node(*last_stmt).unwrap().span.end
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
            let _ = self.ast.set_parent(*context_expr, node_id);

            if let Some(target) = target_opt {
                let _ = self.ast.set_parent(*target, node_id);
            }
        }

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        Ok(node_id)
    }

    /// Parse a list of statements (a block)
    ///
    /// This parses statements until a dedent or end of file is reached.
    /// It tracks indentation to properly handle Python's block structure.
    ///
    /// ## Errors
    ///
    /// Returns a `ParseError` if:
    ///
    /// - Any statement in the list fails to parse
    /// - Indentation is inconsistent or incorrect
    /// - An unexpected token prevents parsing a statement
    pub fn parse_statement_list(&mut self) -> ParseResult<Vec<NodeID>> {
        let mut statements = Vec::new();

        // Track the indentation level when we start
        let start_indent = self.context_stack.current_indent_level();

        // Continue parsing statements until we reach a dedent or EOF
        while !self.check(TokenKind::EndOfFile) {
            // Check if we've reached the end of the block (dedent)
            if self.check(TokenKind::Dedent) {
                // Process the dedent token
                let _ = self.advance();

                // If we're back to the original indent level, we're done
                if self.context_stack.current_indent_level() <= start_indent {
                    break;
                }
            }

            // Parse a statement and add it to the list
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            // Skip newlines between statements
            while self.check(TokenKind::Newline) {
                let _ = self.advance();
            }

            // If we hit a dedent or EOF, we're done with this block
            if self.check(TokenKind::Dedent) || self.check(TokenKind::EndOfFile) {
                break;
            }
        }

        Ok(statements)
    }

    /// Parse a block of statements.
    ///
    /// A block starts with a colon, then a newline, then an indented block of statements.
    ///
    /// ## Errors
    ///
    /// Returns a `ParseError` if:
    ///
    /// - The colon (`:`) is missing
    /// - The newline after the colon is missing
    /// - The indent token is missing (block must be indented)
    /// - Any statement in the block fails to parse
    pub fn parse_block(&mut self) -> ParseResult<Vec<NodeID>> {
        // Expect a colon to start the block
        self.expect(TokenKind::Colon)?;

        // Expect a newline after the colon
        self.expect(TokenKind::Newline)?;

        // Expect an indent to start the block
        self.expect(TokenKind::Indent)?;

        // Update indentation level in the context
        let indent_level = self.context_stack.current_indent_level() + 1;
        self.context_stack.current_mut().indent_level = indent_level;

        // Parse the block's statements
        let statements = self.parse_statement_list()?;

        Ok(statements)
    }

    /// Handle indentation tokens (indent/dedent) to track Python's block structure
    pub fn handle_indentation(&mut self) -> ParseResult<()> {
        match self.current_token().kind {
            TokenKind::Indent => {
                // Process indent and update context
                let _ = self.advance();
                let indent_level = self.context_stack.current_indent_level() + 1;
                self.context_stack.current_mut().indent_level = indent_level;
                self.indent_stack.push(indent_level);
            }
            TokenKind::Dedent => {
                // Process dedent and update context
                let _ = self.advance();
                let _ = self.indent_stack.pop();
                let indent_level = match self.indent_stack.last() {
                    Some(&level) => level,
                    None => 0,
                };
                self.context_stack.current_mut().indent_level = indent_level;
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if the current token could be the start of an assignment statement.
    fn is_assignment(&self) -> bool {
        // If the current token is an identifier and the next token is '='
        self.check(TokenKind::Identifier) && self.peek_token().kind == TokenKind::Assign
    }

    /// Check if the current token could be the start of an augmented assignment statement.
    const fn is_augmented_assignment(&mut self) -> bool {
        // If we have at least one token ahead
        if let Some(peek_ahead) = self.peek_n(1) {
            // Check if it's an augmented assignment operator
            matches!(
                peek_ahead.kind,
                TokenKind::PlusEqual
                    | TokenKind::MinusEqual
                    | TokenKind::StarEqual
                    | TokenKind::SlashEqual
                    | TokenKind::DoubleSlashEqual
                    | TokenKind::PercentEqual
                    | TokenKind::DoubleStarEqual
                    | TokenKind::LeftShiftEqual
                    | TokenKind::RightShiftEqual
                    | TokenKind::AmpersandEqual
                    | TokenKind::PipeEqual
                    | TokenKind::CaretEqual
                    | TokenKind::AtEqual
            )
        } else {
            false
        }
    }

    /// Parse a pass statement (e.g. `pass`).
    fn parse_pass_statement(&mut self) -> ParseResult<NodeID> {
        // Get the span of the 'pass' token
        let token = self.current_token().clone();
        let span = self.token_to_span(&token);

        // Consume the 'pass' token
        let _ = self.advance();

        // Create the Pass node
        let pass = PassStmt::new(NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::PassStmt(pass), span);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an assert statement (e.g. `assert condition [, message]`).
    fn parse_assert_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'assert' token
        let _ = self.advance();

        // Parse the condition expression
        let condition = self.parse_expression()?;

        // Check for an optional message (after a comma)
        let message = if self.check(TokenKind::Comma) {
            let _ = self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = match message {
            Some(_) | None => self.ast.get_node(condition).unwrap().span.end,
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an Assert node
        let assert = AssertStmt::new(condition, message, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Statement, AnyNode::AssertStmt(assert), span);

        // Set parent-child relationship
        let _ = self.ast.set_parent(condition, node_id);

        if let Some(msg) = message {
            let _ = self.ast.set_parent(msg, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an assignment statement (e.g. `target = value`).
    fn parse_assignment_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the target expression
        let target = self.parse_expression()?;

        // Expect the assignment operator
        self.expect(TokenKind::Assign)?;

        // Parse the value expression
        let value = self.parse_expression()?;

        // Get the end position
        let end_pos = self.ast.get_node(value).unwrap().span.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an AssignmentStmt node
        let assignment = AssignmentStmt::new(target, value, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::AssignmentStmt(assignment), span);

        // Set parent-child relationships
        let _ = self.ast.set_parent(target, node_id);
        let _ = self.ast.set_parent(value, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse an expression statement.
    fn parse_expression_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the expression
        let expression = self.parse_expression()?;

        // Get the end position
        let end_pos = self.ast.get_node(expression).unwrap().span.end;

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create an ExpressionStmt node
        let expr_stmt = ExpressionStmt::new(expression, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::ExpressionStmt(expr_stmt), span);

        // Set parent-child relationship
        let _ = self.ast.set_parent(expression, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a while statement (e.g. `while condition: ... [else: ...]`).
    fn parse_while_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'while' token
        let _ = self.advance();

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
        let else_body = if self.check(TokenKind::Else) {
            let _ = self.advance(); // Consume 'else'

            Some(self.parse_block()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();
                self.ast.get_node(*last_stmt).unwrap().span.end
            }
            _ => {
                if body.is_empty() {
                    // Fallback to condition
                    self.ast.get_node(condition).unwrap().span.end
                } else {
                    let last_stmt = body.last().unwrap();
                    self.ast.get_node(*last_stmt).unwrap().span.end
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
        let _ = self.ast.set_parent(condition, node_id);

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                let _ = self.ast.set_parent(*stmt, node_id);
            }
        }

        // Pop the while context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }

    /// Parse an async for statement (e.g. `async for target in iterable: ... [else: ...]`).
    ///
    /// Handles Python's async for loop which is used inside async functions:
    /// ```python
    /// async for item in async_iterable:
    ///     # process item asynchronously
    /// ```
    fn parse_async_for_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'async' token
        let _ = self.advance();

        // Consume the 'for' token
        self.expect(TokenKind::For)?;

        // Create a context for the async for statement
        self.context_stack.push(Context::new(
            ContextType::Loop,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the target expression
        let target = self.parse_expression()?;

        // Expect the 'in' keyword
        self.expect(TokenKind::In)?;

        // Parse the iterable expression
        let iter = self.parse_expression()?;

        // Parse the for body
        let body = self.parse_block()?;

        // Parse optional else branch
        let else_body = if self.check(TokenKind::Else) {
            let _ = self.advance(); // Consume 'else'
            Some(self.parse_block()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();
                self.ast.get_node(*last_stmt).unwrap().span.end
            }
            _ => {
                if body.is_empty() {
                    // Fallback to iter
                    self.ast.get_node(iter).unwrap().span.end
                } else {
                    let last_stmt = body.last().unwrap();
                    self.ast.get_node(*last_stmt).unwrap().span.end
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
        let _ = self.ast.set_parent(target, node_id);
        let _ = self.ast.set_parent(iter, node_id);

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                let _ = self.ast.set_parent(*stmt, node_id);
            }
        }

        // Pop the async for context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }

    /// Parse a for statement (e.g. `for target in iterable: ... [else: ...]`).
    fn parse_for_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'for' token
        let _ = self.advance();

        // Create a context for the for statement
        self.context_stack.push(Context::new(
            ContextType::Loop,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the target expression
        let target = self.parse_expression()?;

        // Expect the 'in' keyword
        self.expect(TokenKind::In)?;

        // Parse the iterable expression
        let iter = self.parse_expression()?;

        // Parse the for body
        let body = self.parse_block()?;

        // Parse optional else branch
        let else_body = if self.check(TokenKind::Else) {
            let _ = self.advance(); // Consume 'else'
            Some(self.parse_block()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = match &else_body {
            Some(stmts) if !stmts.is_empty() => {
                let last_stmt = stmts.last().unwrap();
                self.ast.get_node(*last_stmt).unwrap().span.end
            }
            _ => {
                if body.is_empty() {
                    // Fallback to iter
                    self.ast.get_node(iter).unwrap().span.end
                } else {
                    let last_stmt = body.last().unwrap();
                    self.ast.get_node(*last_stmt).unwrap().span.end
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
        let _ = self.ast.set_parent(target, node_id);
        let _ = self.ast.set_parent(iter, node_id);

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        if let Some(stmts) = &else_body {
            for stmt in stmts {
                let _ = self.ast.set_parent(*stmt, node_id);
            }
        }

        // Pop the for context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }
}
