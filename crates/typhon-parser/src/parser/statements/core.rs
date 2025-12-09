//! Statement parsing entry points (`parse_statement`, `parse_block`, `parse_statement_list`, `parse_variable_declaration`).

use typhon_ast::nodes::{AnyNode, NodeID, NodeKind, VariableDecl};
use typhon_source::types::Span;

use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::parser::context::{Context, ContextType};

impl Parser<'_> {
    /// Parse a block of statements.
    ///
    /// A block starts with a colon, then a newline, then an indented block of statements.
    /// Used in function bodies, class bodies, if/while/for blocks, etc.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// block: ":" NEWLINE INDENT statement+ DEDENT
    /// ```
    ///
    /// ## Examples
    ///
    /// Function body:
    ///
    /// ```python
    /// def foo():
    ///     x = 1
    ///     return x
    /// ```
    ///
    /// If statement body:
    ///
    /// ```python
    /// if condition:
    ///     do_something()
    ///     do_more()
    /// ```
    ///
    /// Class body:
    ///
    /// ```python
    /// class MyClass:
    ///     def __init__(self):
    ///         self.value = 0
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
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

        // Skip any blank lines (additional newlines) before the indent
        self.skip_newlines();

        // Expect an indent to start the block
        self.expect(TokenKind::Indent)?;

        // Update indentation level in the context
        let indent_level = self.context_stack.current_indent_level() + 1;
        self.context_stack.current_mut().indent_level = indent_level;

        // Parse the block's statements
        let statements = self.parse_statement_list()?;

        Ok(statements)
    }

    /// Parse a statement.
    ///
    /// This is the main entry point for parsing a statement. It dispatches to
    /// specific statement parsers based on the current token.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// statement: simple_stmt | compound_stmt | declaration
    /// simple_stmt: expression_stmt | assignment | import | assert | del | pass | return | raise | break | continue | global | nonlocal
    /// compound_stmt: if_stmt | while_stmt | for_stmt | try_stmt | with_stmt | match_stmt | function_def | class_def
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple statement:
    ///
    /// ```python
    /// x = 42
    /// ```
    ///
    /// Compound statement:
    ///
    /// ```python
    /// if x > 0:
    ///     print("positive")
    /// ```
    ///
    /// Declaration:
    ///
    /// ```python
    /// def foo():
    ///     pass
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The statement syntax is invalid
    /// - An unexpected token is encountered
    /// - Required tokens (like `:`, `=`, keywords) are missing
    /// - Indentation is incorrect
    pub fn parse_statement(&mut self) -> ParseResult<NodeID> {
        // Check for indentation changes
        self.handle_indentation()?;

        // Skip any leading newlines (e.g., from comment-only lines)
        self.skip_newlines();

        // Skip any stray indent/dedent tokens that might appear due to indentation tracking issues
        // This can happen when transitioning between different indentation contexts
        self.skip_while(&[TokenKind::Indent, TokenKind::Dedent]);

        let token_kind = self.current_token().kind;

        // Parse the statement based on the current token
        match token_kind {
            // Declarations (can appear as statements in Python)
            TokenKind::At => {
                let decl = self.parse_declaration()?;

                // After parsing a declaration as a statement, consume any dedent tokens
                // that close the declaration's body (e.g., function body, class body)
                self.skip_while(&[TokenKind::Dedent]);

                Ok(decl)
            }

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
            TokenKind::Try => self.parse_try_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Raise => self.parse_raise_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Async if self.peek_token().kind == TokenKind::For => {
                self.parse_async_for_statement()
            }
            TokenKind::Async if self.peek_token().kind == TokenKind::With => {
                self.parse_async_with_statement()
            }
            TokenKind::Async if self.peek_token().kind == TokenKind::Def => {
                // Parse async function declaration - it returns a NodeID
                let decl = self.parse_function_declaration()?;

                // After parsing a declaration as a statement, consume any dedent tokens
                // that close the declaration's body (e.g., function body, class body)
                self.skip_while(&[TokenKind::Dedent]);

                Ok(decl)
            }

            TokenKind::Class => {
                let decl = self.parse_class_declaration()?;

                // After parsing a declaration as a statement, consume any dedent tokens
                // that close the declaration's body (e.g., function body, class body)
                self.skip_while(&[TokenKind::Dedent]);

                Ok(decl)
            }

            TokenKind::Def => {
                let decl = self.parse_function_declaration()?;

                // After parsing a declaration as a statement, consume any dedent tokens
                // that close the declaration's body (e.g., function body, class body)
                self.skip_while(&[TokenKind::Dedent]);

                Ok(decl)
            }

            TokenKind::Type => {
                let decl = self.parse_type_declaration()?;
                // Type declarations don't have bodies, so no dedent to consume

                Ok(decl)
            }

            // Variable declarations with type annotations (e.g., `name: type = value`)
            _ if self.is_variable_declaration() => self.parse_variable_declaration(),
            // Annotated assignment statements (e.g., `self.x: type = value`)
            _ if self.is_annotated_assignment() => self.parse_annotated_assignment_statement(),
            // Assignment statements
            // Look ahead for an equals sign to detect assignment
            _ if self.is_assignment() => self.parse_assignment_statement(),
            // Look ahead for augmented assignment operators
            _ if self.is_augmented_assignment() => self.parse_augmented_assignment_statement(),

            // Expression statements (default case)
            _ => self.parse_expression_statement(),
        }
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
                self.skip();

                // If we're back to the original indent level, we're done
                if self.context_stack.current_indent_level() <= start_indent {
                    break;
                }
            }

            // Parse a statement and add it to the list
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            // Skip newlines between statements
            self.skip_newlines();
        }

        Ok(statements)
    }

    /// Parse a variable declaration (e.g. `name: type = value` or `name: type`).
    ///
    /// This handles module-level and class-level variable declarations with type annotations.
    /// Note: This currently requires a type annotation (the `:type` part).
    /// Without it, the statement would be parsed as a regular assignment.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// variable_decl: identifier `:` type [`=` expression]
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple variable declaration with type:
    ///
    /// ```python
    /// x: int
    /// ```
    ///
    /// Variable declaration with type and initial value:
    ///
    /// ```python
    /// count: int = 0
    /// ```
    ///
    /// Class attribute declaration:
    ///
    /// ```python
    /// class Point:
    ///     x: float
    ///     y: float = 0.0
    /// ```
    ///
    /// Module-level variable with complex type:
    ///
    /// ```python
    /// cache: dict[str, list[int]] = {}
    /// ```
    ///
    /// Multiple declarations:
    ///
    /// ```python
    /// name: str
    /// age: int = 25
    /// active: bool = True
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The variable name is missing or invalid
    /// - The colon `:` is missing
    /// - The type annotation is missing or invalid after `:`
    /// - The initial value is invalid (if present)
    /// - The statement terminator is missing
    pub(super) fn parse_variable_declaration(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Parse the variable name
        let name = if self.check(TokenKind::Identifier) {
            self.current_token().lexeme.to_string()
        } else {
            return Err(self.error("Expected variable name"));
        };
        self.skip(); // Consume the identifier

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

        // Parse optional initial value
        let value = if self.check(TokenKind::Assign) {
            self.skip(); // Consume '='
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Get the end position
        let end_pos = match value {
            Some(val) => self.get_node_span(val)?.end,
            None => self.get_node_span(type_annotation)?.end,
        };

        // Create a span
        let span = Span::new(start_pos, end_pos);

        // Create a VariableDecl node
        let mut var_decl = VariableDecl::new(name, NodeID::placeholder(), span);
        var_decl = var_decl.with_type(type_annotation);

        if let Some(val) = value {
            var_decl = var_decl.with_value(val);
        }

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::VariableDecl(var_decl), span);

        // Set parent-child relationships
        self.set_parent(type_annotation, node_id);

        if let Some(val) = value {
            self.set_parent(val, node_id);
        }

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }
}
