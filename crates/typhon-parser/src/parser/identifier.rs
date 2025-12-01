//! Identifier parsing for the Typhon programming language.
//!
//! This module handles parsing different types of identifiers,
//! including variable identifiers, parameter identifiers, and type identifiers.

use typhon_ast::nodes::{AnyNode, BasicIdent, NodeID, NodeKind, ParameterIdent, VariableIdent};

use super::Parser;
use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;

impl Parser<'_> {
    /// Parse an identifier (dispatches based on context).
    ///
    /// This method parses a standard identifier token and creates the appropriate
    /// AST node based on the context. The naming convention (private, constant, etc.)
    /// is determined by the identifier's name string, not by the token type.
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the current token is not a valid identifier or
    /// if there is another parsing error.
    pub fn parse_identifier(&mut self) -> ParseResult<NodeID> {
        if self.current.kind != TokenKind::Identifier {
            let message = format!("Expected identifier, found {}", self.current.kind);

            return Err(self.error(&message));
        }

        // Get the identifier name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the identifier token
        let _ = self.advance();

        // Create the basic identifier node
        let identifier = BasicIdent::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::BasicIdent(identifier), span);

        Ok(node_id)
    }

    /// Parse a variable identifier (standard name in expressions).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the current token is not a standard identifier.
    pub fn parse_variable_identifier(&mut self) -> ParseResult<NodeID> {
        if self.current.kind != TokenKind::Identifier {
            return Err(self.error("Expected identifier"));
        }

        // Get the identifier name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the identifier token
        let _ = self.advance();

        // Create the variable node
        let variable = VariableIdent { name, id: NodeID::placeholder(), parent: None, span };

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Expression, AnyNode::VariableIdent(variable), span);

        Ok(node_id)
    }

    /// Parse a parameter identifier (in function declarations).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    pub fn parse_parameter_identifier(&mut self) -> ParseResult<NodeID> {
        // Parse the parameter name
        if self.current.kind != TokenKind::Identifier {
            return Err(self.error("Expected parameter name"));
        }

        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the parameter name
        let _ = self.advance();

        // Create the parameter node
        let mut parameter = ParameterIdent {
            name,
            id: NodeID::placeholder(),
            parent: None,
            type_annotation: None,
            default_value: None,
            span,
        };

        // Check for type annotation
        if self.check(TokenKind::Colon) {
            let _ = self.advance(); // Skip the colon

            // Set the in_type_annotation flag
            let old_in_type = self.context_stack.current_mut().flags.in_type_annotation;
            self.context_stack.current_mut().flags.in_type_annotation = true;

            // Parse the type expression
            let type_annotation = self.parse_type_expression()?;
            parameter.type_annotation = Some(type_annotation);

            // Restore the flag
            self.context_stack.current_mut().flags.in_type_annotation = old_in_type;
        }

        // Check for default value
        if self.check(TokenKind::Assign) {
            let _ = self.advance(); // Skip the equals sign

            // Set the in_default_arg flag
            let old_default = self.context_stack.current_mut().flags.in_default_arg;
            self.context_stack.current_mut().flags.in_default_arg = true;

            // Parse the default value
            let default_value = self.parse_expression()?;
            parameter.default_value = Some(default_value);

            // Restore the flag
            self.context_stack.current_mut().flags.in_default_arg = old_default;
        }

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::ParameterIdent(parameter), span);

        Ok(node_id)
    }

    /// Parse a type identifier (name used in type annotations).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    pub fn parse_type_identifier(&mut self) -> ParseResult<NodeID> {
        // Ensure we have an identifier
        if self.current.kind != TokenKind::Identifier {
            return Err(self.error("Expected type name"));
        }

        // Get the identifier name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the identifier token
        let _ = self.advance();

        // Create the identifier node
        let identifier = BasicIdent::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::BasicIdent(identifier), span);

        Ok(node_id)
    }

    /// Parse a module name identifier (for import statements).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    pub fn parse_module_identifier(&mut self) -> ParseResult<NodeID> {
        // Ensure we have an identifier
        if self.current.kind != TokenKind::Identifier {
            return Err(self.error("Expected module name"));
        }

        // Get the identifier name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the module name
        let _ = self.advance();

        // Create the identifier node
        let identifier = BasicIdent::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::BasicIdent(identifier), span);

        Ok(node_id)
    }
}
