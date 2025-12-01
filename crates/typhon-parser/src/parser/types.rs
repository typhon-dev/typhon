//! Type annotation parsing for the Typhon programming language.
//!
//! This module handles parsing different types of type annotations,
//! including callable, generic, literal, tuple, and union type.

use typhon_ast::nodes::{
    AnyNode,
    CallableType,
    GenericType,
    LiteralType,
    NodeID,
    NodeKind,
    TupleType,
    UnionType,
};
use typhon_source::types::Span;

use super::Parser;
use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;

impl Parser<'_> {
    /// Parse a type expression (any type annotation).
    ///
    /// This method handles simple types, generic types, union types,
    /// callable types, tuple types, and literal types.
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    pub fn parse_type_expression(&mut self) -> ParseResult<NodeID> {
        // Special case for type literals
        if self.current.kind == TokenKind::Identifier && self.current.lexeme == "Callable" {
            return self.parse_callable_type();
        }

        if self.current.kind == TokenKind::Identifier && self.current.lexeme == "Literal" {
            return self.parse_literal_type();
        }

        if self.current.kind == TokenKind::Identifier && self.current.lexeme == "tuple" {
            // Special case for tuple types (e.g., tuple[int, str])
            let id = self.parse_type_identifier()?;

            if self.check(TokenKind::LeftBracket) {
                return self.parse_tuple_type(id);
            }

            // If no bracket follows, treat as a simple type
            return Ok(id);
        }

        // Start with a simple type
        let type_id = self.parse_type_identifier()?;

        // Check if this is a generic type
        if self.check(TokenKind::LeftBracket) {
            return self.parse_generic_type(type_id);
        }

        // Check if this is a union type
        if self.check(TokenKind::Pipe) {
            return self.parse_union_type(type_id);
        }

        // Simple type
        Ok(type_id)
    }

    /// Parse a literal type (e.g. `Literal["red", "green", "blue"]`).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    fn parse_literal_type(&mut self) -> ParseResult<NodeID> {
        // Get the start span
        let start_span = self.token_to_span(&self.current);

        // Consume "Literal"
        let _ = self.advance();

        // Expect left bracket
        if !self.check(TokenKind::LeftBracket) {
            return Err(self.error("Expected '[' after 'Literal'"));
        }

        let _ = self.advance();

        // Parse literal values
        let mut value_ids = Vec::new();

        // Handle empty literal list
        if !self.check(TokenKind::RightBracket) {
            // Parse first literal value
            let value_id = self.parse_expression()?;
            value_ids.push(value_id);

            // Parse additional values separated by commas
            while self.check(TokenKind::Comma) {
                let _ = self.advance(); // Skip comma

                // Check for trailing comma
                if self.check(TokenKind::RightBracket) {
                    break;
                }

                let value_id = self.parse_expression()?;
                value_ids.push(value_id);
            }
        }

        // Expect closing bracket
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close Literal type"));
        }

        let _ = self.advance();

        // Get the end span
        let end_span = self.token_to_span(&self.current);
        let span = Span::new(start_span.start, end_span.end);

        // Create the literal type node
        let literal_type = LiteralType::new(value_ids.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::LiteralType(literal_type), span);

        // Set parent-child relationships
        for value_id in &value_ids {
            let _ = self.ast.set_parent(*value_id, node_id);
        }

        Ok(node_id)
    }

    /// Parse a tuple type (e.g. `tuple[int, str, float]`).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    fn parse_tuple_type(&mut self, base_id: NodeID) -> ParseResult<NodeID> {
        // Consume the left bracket
        let _ = self.advance();

        // Get the base type span
        let base_span = self.ast.get_node(base_id).unwrap().span;

        // Parse type arguments
        let mut element_type_ids = Vec::new();

        // Handle empty tuple
        if !self.check(TokenKind::RightBracket) {
            // Parse first element type
            let element_id = self.parse_type_expression()?;
            element_type_ids.push(element_id);

            // Parse additional element types separated by commas
            while self.check(TokenKind::Comma) {
                let _ = self.advance(); // Skip comma

                // Check for trailing comma
                if self.check(TokenKind::RightBracket) {
                    break;
                }

                let element_id = self.parse_type_expression()?;
                element_type_ids.push(element_id);
            }
        }

        // Expect closing bracket
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close tuple type"));
        }

        let _ = self.advance();

        // Get the end span
        let end_span = self.token_to_span(&self.current);
        let span = Span::new(base_span.start, end_span.end);

        // Create the tuple type node
        let tuple_type = TupleType::new(element_type_ids.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::TupleType(tuple_type), span);

        // Set parent-child relationships
        for element_id in &element_type_ids {
            let _ = self.ast.set_parent(*element_id, node_id);
        }

        Ok(node_id)
    }

    /// Parse a generic type (with type parameters).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    fn parse_generic_type(&mut self, base_id: NodeID) -> ParseResult<NodeID> {
        // Consume the left bracket
        let _ = self.advance();

        // Get the base type span
        let base_span = self.ast.get_node(base_id).unwrap().span;

        // Parse type arguments
        let mut arg_ids = Vec::new();

        // Handle empty brackets
        if !self.check(TokenKind::RightBracket) {
            // Parse first type argument
            let arg_id = self.parse_type_expression()?;
            arg_ids.push(arg_id);

            // Parse additional arguments separated by commas
            while self.check(TokenKind::Comma) {
                let _ = self.advance(); // Skip comma
                let arg_id = self.parse_type_expression()?;
                arg_ids.push(arg_id);
            }
        }

        // Expect closing bracket
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close generic type argument list"));
        }

        // Consume the right bracket
        let _ = self.advance();

        // Get the end position from the current token
        let end_span = self.token_to_span(&self.current);
        let span = Span::new(base_span.start, end_span.end);

        // Create the generic type node
        let generic_type = GenericType {
            base_id,
            arg_ids: arg_ids.clone(),
            id: NodeID::placeholder(),
            parent: None,
            span,
        };

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::GenericType(generic_type), span);

        // Set parent-child relationships
        let _ = self.ast.set_parent(base_id, node_id);
        for arg_id in &arg_ids {
            let _ = self.ast.set_parent(*arg_id, node_id);
        }

        Ok(node_id)
    }

    /// Parse a callable type (e.g. `Callable[[int, str], bool]`).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    fn parse_callable_type(&mut self) -> ParseResult<NodeID> {
        // Get the start span
        let start_span = self.token_to_span(&self.current);

        // Consume "Callable"
        let _ = self.advance();

        // Expect first left bracket
        if !self.check(TokenKind::LeftBracket) {
            return Err(self.error("Expected '[' after 'Callable'"));
        }

        let _ = self.advance();

        // Expect second left bracket for parameter list
        if !self.check(TokenKind::LeftBracket) {
            return Err(self.error("Expected '[' for parameter list in Callable type"));
        }

        let _ = self.advance();

        // Parse parameter types
        let mut param_ids = Vec::new();

        // Handle empty parameter list
        if !self.check(TokenKind::RightBracket) {
            // Parse first parameter type
            let param_id = self.parse_type_expression()?;
            param_ids.push(param_id);

            // Parse additional parameters separated by commas
            while self.check(TokenKind::Comma) {
                let _ = self.advance(); // Skip comma
                let param_id = self.parse_type_expression()?;
                param_ids.push(param_id);
            }
        }

        // Expect closing bracket for parameter list
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close parameter list in Callable type"));
        }

        let _ = self.advance();

        // Expect comma between parameter list and return type
        if !self.check(TokenKind::Comma) {
            return Err(self.error("Expected ',' after parameter list in Callable type"));
        }

        let _ = self.advance();

        // Parse return type
        let return_type_id = self.parse_type_expression()?;

        // Expect closing bracket
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close Callable type"));
        }

        let _ = self.advance();

        // Get the end span
        let end_span = self.token_to_span(&self.current);
        let span = Span::new(start_span.start, end_span.end);

        // Create the function type node
        let function_type = CallableType {
            param_ids: param_ids.clone(),
            return_type_id,
            id: NodeID::placeholder(),
            parent: None,
            span,
        };

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Type, AnyNode::CallableType(function_type), span);

        // Set parent-child relationships
        for param_id in &param_ids {
            let _ = self.ast.set_parent(*param_id, node_id);
        }

        let _ = self.ast.set_parent(return_type_id, node_id);

        Ok(node_id)
    }

    /// Parse a union type (e.g. int | str).
    ///
    /// ## Errors
    ///
    /// Returns a `ParserError` if the parsing fails.
    fn parse_union_type(&mut self, first_type_id: NodeID) -> ParseResult<NodeID> {
        // Get the first type span
        let first_span = self.ast.get_node(first_type_id).unwrap().span;

        // Store the first type ID in the list of types
        let mut type_ids = vec![first_type_id];

        // Parse additional types in the union
        while self.check(TokenKind::Pipe) {
            // Consume the pipe
            let _ = self.advance();

            // Parse the next type in the union
            let next_type_id = self.parse_type_expression()?;
            type_ids.push(next_type_id);
        }

        // Get the end span from the most recently parsed type
        let last_type_span = self.ast.get_node(*type_ids.last().unwrap()).unwrap().span;
        let span = Span::new(first_span.start, last_type_span.end);

        // Create the union type node
        let union_type =
            UnionType { type_ids: type_ids.clone(), id: NodeID::placeholder(), parent: None, span };

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::UnionType(union_type), span);

        // Set parent-child relationships
        for type_id in &type_ids {
            let _ = self.ast.set_parent(*type_id, node_id);
        }

        Ok(node_id)
    }
}
