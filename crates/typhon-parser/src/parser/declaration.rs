//! Declaration parsing for the Typhon programming language.
//!
//! This module handles parsing declarations of all types,
//! including function definitions, class definitions, and type definitions.

use typhon_ast::nodes::{
    AnyNode,
    AsyncFunctionDecl,
    ClassDecl,
    FunctionDecl,
    NodeID,
    NodeKind,
    ParameterIdent,
    TypeDecl,
};
use typhon_source::types::Span;

use super::Parser;
use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;
use crate::parser::context::{Context, ContextFlags, ContextType, FunctionModifiers};

impl Parser<'_> {
    /// Parse a declaration.
    ///
    /// This is the main entry point for parsing a declaration.
    /// It dispatches to specific declaration parsers based on the current token.
    ///
    /// ## Errors
    ///
    /// Returns an error if there is a syntax error in the declaration.
    pub fn parse_declaration(&mut self) -> ParseResult<NodeID> {
        // Check the current token to determine the type of declaration
        match self.current_token().kind {
            TokenKind::Def => self.parse_function_declaration(),
            TokenKind::Class => self.parse_class_declaration(),
            TokenKind::Identifier
                if self.current_token().lexeme == "type"
                    && self.peek_token().kind == TokenKind::Identifier =>
            {
                self.parse_type_declaration()
            }
            _ => Err(self.error("Expected declaration (function, class, or type)")),
        }
    }

    /// Parse a function declaration.
    ///
    /// This parses a function declaration of the form:
    /// ```
    /// [async] def name([parameters])[-> return_type]: body
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an error if there is a syntax error in the function declaration.
    fn parse_function_declaration(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Check for async keyword
        let is_async = if self.check(TokenKind::Async) {
            let _ = self.advance(); // Consume 'async'
            true
        } else {
            false
        };

        // Expect the 'def' keyword
        self.expect(TokenKind::Def)?;

        // Create a context for the function declaration
        let function_modifiers = FunctionModifiers { is_async, ..Default::default() };

        let context_flags = ContextFlags { fn_modifiers: function_modifiers, ..Default::default() };

        self.context_stack.push(Context::with_flags(
            ContextType::Function,
            None,
            self.context_stack.current_indent_level(),
            context_flags,
        ));

        // Parse the function name (identifier)
        let name = if self.check(TokenKind::Identifier) {
            self.current_token().lexeme.to_string()
        } else {
            return Err(self.error("Expected function name"));
        };
        let _ = self.advance(); // Consume the identifier

        // Parse the parameter list
        self.expect(TokenKind::LeftParen)?;
        let parameters = self.parse_parameter_list()?;
        self.expect(TokenKind::RightParen)?;

        // Parse optional return type annotation
        let return_type = if self.check(TokenKind::Arrow) {
            let _ = self.advance(); // Consume '->'
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse the function body
        let body = self.parse_block()?;

        // Get the end position
        let end_pos = if body.is_empty() {
            // If no body, use the end of the function signature
            if let Some(rt) = return_type {
                self.ast.get_node(rt).unwrap().span.end
            } else {
                self.current_token().span.end
            }
        } else {
            // Use the end of the last statement in the body
            let last_stmt = body.last().unwrap();
            self.ast.get_node(*last_stmt).unwrap().span.end
        };

        // Create the span for the function declaration
        let span = Span::new(start_pos, end_pos);

        // Create the appropriate function node based on the async flag
        let node_id = if is_async {
            // Create an AsyncFunctionDef node for async functions
            let mut async_function_def = AsyncFunctionDecl::new(
                name,
                parameters.clone(),
                body.clone(),
                NodeID::placeholder(),
                span,
            );

            if let Some(rt) = return_type {
                async_function_def = async_function_def.with_return_type(rt);
            }

            // Allocate the node in the AST
            self.ast.alloc_node(
                NodeKind::Declaration,
                AnyNode::AsyncFunctionDecl(async_function_def),
                span,
            )
        } else {
            // Create a regular FunctionDef node for non-async functions
            let mut function_def = FunctionDecl::new(
                name,
                parameters.clone(),
                body.clone(),
                NodeID::placeholder(),
                span,
            );

            if let Some(rt) = return_type {
                function_def = function_def.with_return_type(rt);
            }

            // Allocate the node in the AST
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::FunctionDecl(function_def), span)
        };

        // Set parent-child relationships
        for param in &parameters {
            let _ = self.ast.set_parent(*param, node_id);
        }

        if let Some(rt) = return_type {
            let _ = self.ast.set_parent(rt, node_id);
        }

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        // Pop the function context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }

    /// Parse a parameter list for a function declaration.
    ///
    /// This parses parameters of the form:
    /// ```
    /// name: type = default_value, ...
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an error if there is a syntax error in the parameter list.
    fn parse_parameter_list(&mut self) -> ParseResult<Vec<NodeID>> {
        let mut parameters = Vec::new();

        // Empty parameter list
        if self.check(TokenKind::RightParen) {
            return Ok(parameters);
        }

        // Loop until we hit the closing parenthesis
        loop {
            // Get the start position of the parameter
            let start_pos = self.current_token().span.start;

            // Parse parameter name
            let name = if self.check(TokenKind::Identifier) {
                self.current_token().lexeme.to_string()
            } else {
                return Err(self.error("Expected parameter name"));
            };
            let _ = self.advance(); // Consume the parameter name

            // Parse optional type annotation
            let type_annotation = if self.check(TokenKind::Colon) {
                let _ = self.advance(); // Consume ':'

                // Set the flag to indicate we're in a type annotation
                let old_in_type_annotation =
                    self.context_stack.current_mut().flags.in_type_annotation;
                self.context_stack.current_mut().flags.in_type_annotation = true;

                let type_expr = self.parse_expression()?;

                // Restore the flag
                self.context_stack.current_mut().flags.in_type_annotation = old_in_type_annotation;

                Some(type_expr)
            } else {
                None
            };

            // Parse optional default value
            let default_value = if self.check(TokenKind::Assign) {
                let _ = self.advance(); // Consume '='

                // Set the flag to indicate we're in a default argument
                let old_in_default_arg = self.context_stack.current_mut().flags.in_default_arg;
                self.context_stack.current_mut().flags.in_default_arg = true;

                let default_expr = self.parse_expression()?;

                // Restore the flag
                self.context_stack.current_mut().flags.in_default_arg = old_in_default_arg;

                Some(default_expr)
            } else {
                None
            };

            // Create the span for the parameter
            let end_pos = match (type_annotation, default_value) {
                (_, Some(def)) => self.ast.get_node(def).unwrap().span.end,
                (Some(typ), None) => self.ast.get_node(typ).unwrap().span.end,
                (None, None) => start_pos + name.len(),
            };
            let span = Span::new(start_pos, end_pos);

            // Create the parameter node
            let mut param = ParameterIdent::new(name, NodeID::placeholder(), span);

            if let Some(typ) = type_annotation {
                param = param.with_type(typ);
            }

            if let Some(def) = default_value {
                param = param.with_default(def);
            }

            // Allocate the node in the AST
            let param_id =
                self.ast.alloc_node(NodeKind::Identifier, AnyNode::ParameterIdent(param), span);

            // Set parent-child relationships for type annotation and default value
            if let Some(typ) = type_annotation {
                let _ = self.ast.set_parent(typ, param_id);
            }

            if let Some(def) = default_value {
                let _ = self.ast.set_parent(def, param_id);
            }

            // Add the parameter to the list
            parameters.push(param_id);

            // Check if we have more parameters
            if self.check(TokenKind::Comma) {
                let _ = self.advance(); // Consume ','

                // If we see a right paren after a comma, it's a trailing comma
                if self.check(TokenKind::RightParen) {
                    break;
                }
            } else {
                // If we don't see a comma, we should be at the end of the parameter list
                break;
            }
        }

        Ok(parameters)
    }

    /// Parse a class declaration.
    ///
    /// This parses a class declaration of the form:
    /// ```
    /// class Name[(BaseClass1, BaseClass2, ...)]: body
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an error if there is a syntax error in the class declaration.
    fn parse_class_declaration(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Expect the 'class' keyword
        self.expect(TokenKind::Class)?;

        // Create a context for the class declaration
        self.context_stack.push(Context::new(
            ContextType::Class,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the class name (identifier)
        let name = if self.check(TokenKind::Identifier) {
            self.current_token().lexeme.to_string()
        } else {
            return Err(self.error("Expected class name"));
        };
        let _ = self.advance(); // Consume the identifier

        // Parse optional base classes
        let bases = if self.check(TokenKind::LeftParen) {
            let _ = self.advance(); // Consume '('
            self.parse_base_classes()?
        } else {
            Vec::new()
        };

        // Parse the class body
        let body = self.parse_block()?;

        // Get the end position
        let end_pos = if body.is_empty() {
            // If no body, use the end of the class signature
            self.current_token().span.end
        } else {
            // Use the end of the last statement in the body
            let last_stmt = body.last().unwrap();
            self.ast.get_node(*last_stmt).unwrap().span.end
        };

        // Create the span for the class declaration
        let span = Span::new(start_pos, end_pos);

        // Create the ClassDef node
        let class_def =
            ClassDecl::new(name, bases.clone(), body.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::ClassDecl(class_def), span);

        // Set parent-child relationships
        for base in &bases {
            let _ = self.ast.set_parent(*base, node_id);
        }

        for stmt in &body {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        // Pop the class context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }

    /// Parse base classes for a class declaration.
    ///
    /// This parses a list of base classes separated by commas.
    ///
    /// ## Errors
    ///
    /// Returns an error if there is a syntax error in the base class list.
    fn parse_base_classes(&mut self) -> ParseResult<Vec<NodeID>> {
        let mut bases = Vec::new();

        // Empty base class list
        if self.check(TokenKind::RightParen) {
            let _ = self.advance(); // Consume ')'
            return Ok(bases);
        }

        // Loop until we hit the closing parenthesis
        loop {
            // Parse base class expression (typically an identifier, but could be more complex)
            let base = self.parse_expression()?;
            bases.push(base);

            // Check if we have more base classes
            if self.check(TokenKind::Comma) {
                let _ = self.advance(); // Consume ','

                // If we see a right paren after a comma, it's a trailing comma
                if self.check(TokenKind::RightParen) {
                    let _ = self.advance(); // Consume ')'
                    break;
                }
            } else {
                // If we don't see a comma, we should be at the end of the base class list
                self.expect(TokenKind::RightParen)?;
                break;
            }
        }

        Ok(bases)
    }

    /// Parse a type declaration.
    ///
    /// This parses a type declaration of the form:
    /// ```
    /// type Name = OriginalType
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an error if there is a syntax error in the type declaration.
    fn parse_type_declaration(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'type' keyword (which is an identifier)
        let _ = self.advance();

        // Parse the type name (identifier)
        let name = if self.check(TokenKind::Identifier) {
            self.current_token().lexeme.to_string()
        } else {
            return Err(self.error("Expected type name"));
        };

        let _ = self.advance(); // Consume the identifier

        // Expect the '=' token
        self.expect(TokenKind::Assign)?;

        // Set the flag to indicate we're in a type annotation
        let old_in_type_annotation = self.context_stack.current_mut().flags.in_type_annotation;
        self.context_stack.current_mut().flags.in_type_annotation = true;

        // Parse the original type expression
        let original_type = self.parse_expression()?;

        // Restore the flag
        self.context_stack.current_mut().flags.in_type_annotation = old_in_type_annotation;

        // Get the end position
        let end_pos = self.ast.get_node(original_type).unwrap().span.end;

        // Create the span for the type declaration
        let span = Span::new(start_pos, end_pos);

        // Create the TypeDef node
        let type_def = TypeDecl::new(name, original_type, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Declaration, AnyNode::TypeDecl(type_def), span);

        // Set parent-child relationship
        let _ = self.ast.set_parent(original_type, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }
}
