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
    /// Parse a callable type (e.g. `Callable[[int, str], bool]`).
    ///
    /// Callable types represent function signatures, specifying the parameter
    /// types and return type. The syntax uses nested brackets: the outer brackets
    /// wrap the entire type, and the inner brackets contain the parameter list.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// callable_type: "Callable" "[" "[" param_types "]" "," return_type "]"
    /// param_types: type_expression ("," type_expression)*
    ///            | ε
    /// return_type: type_expression
    /// ```
    ///
    /// ## Examples
    ///
    /// No parameters:
    ///
    /// ```python
    /// factory: Callable[[], int]
    /// ```
    ///
    /// Single parameter:
    ///
    /// ```python
    /// transformer: Callable[[str], int]
    /// ```
    ///
    /// Multiple parameters:
    ///
    /// ```python
    /// callback: Callable[[int, str], bool]
    /// processor: Callable[[list[int], dict[str, str]], None]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing first `[` after `Callable` keyword
    /// - Missing second `[` for parameter list
    /// - Invalid type expression in parameter list
    /// - Missing closing `]` for parameter list
    /// - Missing `,` between parameter list and return type
    /// - Invalid return type expression
    /// - Missing closing `]` for callable type
    fn parse_callable_type(&mut self) -> ParseResult<NodeID> {
        // Get the start span
        let start_span = self.token_to_span(&self.current);

        // Consume "Callable"
        self.skip();

        // Expect first left bracket
        if !self.check(TokenKind::LeftBracket) {
            return Err(self.error("Expected '[' after 'Callable'"));
        }

        self.skip();

        // Expect second left bracket for parameter list
        if !self.check(TokenKind::LeftBracket) {
            return Err(self.error("Expected '[' for parameter list in Callable type"));
        }

        self.skip();

        // Parse parameter types
        let mut param_ids = Vec::new();

        // Handle empty parameter list
        if !self.check(TokenKind::RightBracket) {
            // Parse first parameter type
            let param_id = self.parse_type_expression()?;
            param_ids.push(param_id);

            // Parse additional parameters separated by commas
            while self.check(TokenKind::Comma) {
                self.skip(); // Skip comma
                let param_id = self.parse_type_expression()?;
                param_ids.push(param_id);
            }
        }

        // Expect closing bracket for parameter list
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close parameter list in Callable type"));
        }

        self.skip();

        // Expect comma between parameter list and return type
        if !self.check(TokenKind::Comma) {
            return Err(self.error("Expected ',' after parameter list in Callable type"));
        }

        self.skip();

        // Parse return type
        let return_type_id = self.parse_type_expression()?;

        // Expect closing bracket
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close Callable type"));
        }

        self.skip();

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
            self.set_parent(*param_id, node_id);
        }

        self.set_parent(return_type_id, node_id);

        Ok(node_id)
    }

    /// Parse a generic type (with type parameters).
    ///
    /// Generic types allow parameterization of types with other types. Common
    /// examples include container types like `list[T]`, `dict[K, V]`, and
    /// `Optional[T]`. The base type is followed by type arguments in brackets.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// generic_type: type_identifier "[" type_arguments "]"
    /// type_arguments: type_expression ("," type_expression)*
    ///               | ε
    /// ```
    ///
    /// ## Examples
    ///
    /// Single type parameter:
    ///
    /// ```python
    /// items: list[str]
    /// optional: Optional[int]
    /// ```
    ///
    /// Multiple type parameters:
    ///
    /// ```python
    /// mapping: dict[str, int]
    /// pair: tuple[int, str]
    /// ```
    ///
    /// Nested generics:
    ///
    /// ```python
    /// matrix: list[list[float]]
    /// lookup: dict[str, list[int]]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing `[` after base type identifier
    /// - Invalid type expression in argument list
    /// - Missing commas between type arguments
    /// - Missing closing `]` bracket
    /// - Type argument parsing fails
    fn parse_generic_type(&mut self, base_id: NodeID) -> ParseResult<NodeID> {
        // Consume the left bracket
        self.skip();

        // Get the base type span
        let base_span = self.get_node_span(base_id)?;

        // Parse type arguments
        let mut arg_ids = Vec::new();

        // Handle empty brackets
        if !self.check(TokenKind::RightBracket) {
            // Parse first type argument
            let arg_id = self.parse_type_expression()?;
            arg_ids.push(arg_id);

            // Parse additional arguments separated by commas
            while self.check(TokenKind::Comma) {
                self.skip(); // Skip comma
                let arg_id = self.parse_type_expression()?;
                arg_ids.push(arg_id);
            }
        }

        // Expect closing bracket
        if !self.check(TokenKind::RightBracket) {
            return Err(self.error("Expected ']' to close generic type argument list"));
        }

        // Consume the right bracket
        self.skip();

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
        self.set_parent(base_id, node_id);
        for arg_id in &arg_ids {
            self.set_parent(*arg_id, node_id);
        }

        Ok(node_id)
    }

    /// Parse a literal type (e.g. `Literal["red", "green", "blue"]`).
    ///
    /// Literal types allow you to specify that a variable can only have specific
    /// literal values. This is useful for defining enumerations, constants, and
    /// restricting values to a specific set.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// literal_type: "Literal" "[" literal_values "]"
    /// literal_values: expression ("," expression)* [","]
    ///               | ε
    /// ```
    ///
    /// ## Examples
    ///
    /// String literals:
    ///
    /// ```python
    /// color: Literal["red", "green", "blue"]
    /// ```
    ///
    /// Integer literals:
    ///
    /// ```python
    /// status: Literal[200, 404, 500]
    /// ```
    ///
    /// Mixed types:
    ///
    /// ```python
    /// value: Literal["success", 1, True]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Missing `[` after `Literal` keyword
    /// - Invalid literal value expression
    /// - Missing commas between values
    /// - Missing closing `]` bracket
    fn parse_literal_type(&mut self) -> ParseResult<NodeID> {
        // Get the start span
        let start_span = self.token_to_span(&self.current);

        // Consume "Literal"
        self.skip();

        // Expect left bracket
        if !self.check(TokenKind::LeftBracket) {
            return Err(self.error("Expected '[' after 'Literal'"));
        }

        self.skip();

        // Parse literal values
        let mut value_ids = Vec::new();

        // Handle empty literal list
        if !self.check(TokenKind::RightBracket) {
            // Parse first literal value
            let value_id = self.parse_expression()?;
            value_ids.push(value_id);

            // Parse additional values separated by commas
            while self.check(TokenKind::Comma) {
                self.skip(); // Skip comma

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

        self.skip();

        // Get the end span
        let end_span = self.token_to_span(&self.current);
        let span = Span::new(start_span.start, end_span.end);

        // Create the literal type node
        let literal_type = LiteralType::new(value_ids.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::LiteralType(literal_type), span);

        // Set parent-child relationships
        for value_id in &value_ids {
            self.set_parent(*value_id, node_id);
        }

        Ok(node_id)
    }

    /// Parse a tuple type (e.g. `tuple[int, str, float]`).
    ///
    /// Tuple types represent fixed-length sequences with heterogeneous element
    /// types. Each element can have a different type, and the number and order
    /// of types is part of the type signature.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// tuple_type: "tuple" "[" type_list "]"
    /// type_list: type_expression ("," type_expression)* [","]
    ///          | ε
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple tuple:
    ///
    /// ```python
    /// point: tuple[int, int]
    /// ```
    ///
    /// Mixed types:
    ///
    /// ```python
    /// record: tuple[str, int, bool]
    /// ```
    ///
    /// Nested types:
    ///
    /// ```python
    /// complex: tuple[list[int], dict[str, str], int | None]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Invalid type expression in element list
    /// - Missing commas between element types
    /// - Missing closing `]` bracket
    /// - Type expression parsing fails
    fn parse_tuple_type(&mut self, base_id: NodeID) -> ParseResult<NodeID> {
        // Consume the left bracket
        self.skip();

        // Get the base type span
        let base_span = self.get_node_span(base_id)?;

        // Parse type arguments
        let mut element_type_ids = Vec::new();

        // Handle empty tuple
        if !self.check(TokenKind::RightBracket) {
            // Parse first element type
            let element_id = self.parse_type_expression()?;
            element_type_ids.push(element_id);

            // Parse additional element types separated by commas
            while self.check(TokenKind::Comma) {
                self.skip(); // Skip comma

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

        self.skip();

        // Get the end span
        let end_span = self.token_to_span(&self.current);
        let span = Span::new(base_span.start, end_span.end);

        // Create the tuple type node
        let tuple_type = TupleType::new(element_type_ids.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::TupleType(tuple_type), span);

        // Set parent-child relationships
        for element_id in &element_type_ids {
            self.set_parent(*element_id, node_id);
        }

        Ok(node_id)
    }

    /// Parse a type expression (any type annotation).
    ///
    /// This is the main entry point for parsing type annotations. It handles
    /// simple types, generic types with parameters, union types, callable
    /// function types, tuple types, and literal value types. The method
    /// dispatches to specialized parsers based on the type syntax.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// type_expression: simple_type
    ///                | generic_type
    ///                | union_type
    ///                | callable_type
    ///                | tuple_type
    ///                | literal_type
    /// simple_type: identifier
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple type:
    ///
    /// ```python
    /// x: int
    /// ```
    ///
    /// Generic type:
    ///
    /// ```python
    /// items: list[str]
    /// ```
    ///
    /// Union type:
    ///
    /// ```python
    /// value: int | str | None
    /// ```
    ///
    /// Callable type:
    ///
    /// ```python
    /// callback: Callable[[int, str], bool]
    /// ```
    ///
    /// Tuple type:
    ///
    /// ```python
    /// point: tuple[int, int]
    /// ```
    ///
    /// Literal type:
    ///
    /// ```python
    /// color: Literal["red", "green", "blue"]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The type identifier is invalid
    /// - Generic type arguments are malformed
    /// - Union type syntax is incorrect
    /// - Callable type structure is invalid
    /// - Tuple or literal type has syntax errors
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

    /// Parse a union type (e.g. int | str).
    ///
    /// Union types allow a value to be one of several types. They use the pipe
    /// operator (`|`) to separate alternative types. This is Python 3.10+ style
    /// syntax, replacing the older `Union[T1, T2]` notation.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// union_type: type_expression ("|" type_expression)+
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple union:
    ///
    /// ```python
    /// value: int | str
    /// ```
    ///
    /// Multiple types:
    ///
    /// ```python
    /// result: int | str | bool | None
    /// ```
    ///
    /// Complex types:
    ///
    /// ```python
    /// data: list[int] | dict[str, int] | None
    /// callback: Callable[[int], str] | Callable[[str], int]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Invalid type expression after `|` operator
    /// - Type expression parsing fails
    /// - Missing type after `|` operator
    fn parse_union_type(&mut self, first_type_id: NodeID) -> ParseResult<NodeID> {
        // Get the first type span
        let first_span = self.get_node_span(first_type_id)?;

        // Store the first type ID in the list of types
        let mut type_ids = vec![first_type_id];

        // Parse additional types in the union
        while self.check(TokenKind::Pipe) {
            // Consume the pipe
            self.skip();

            // Parse the next type in the union
            let next_type_id = self.parse_type_expression()?;
            type_ids.push(next_type_id);
        }

        // Get the end span from the most recently parsed type
        let last_type_id = *type_ids.last().expect("type_ids should not be empty");
        let last_type_span = self.get_node_span(last_type_id)?;
        let span = Span::new(first_span.start, last_type_span.end);

        // Create the union type node
        let union_type = UnionType::new(type_ids.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Type, AnyNode::UnionType(union_type), span);

        // Set parent-child relationships
        for type_id in &type_ids {
            self.set_parent(*type_id, node_id);
        }

        Ok(node_id)
    }
}
