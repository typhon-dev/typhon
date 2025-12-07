//! Identifier parsing for the Typhon programming language.
//!
//! This module handles parsing different types of identifiers,
//! including variable identifiers, parameter identifiers, and type identifiers.

use typhon_ast::nodes::{AnyNode, BasicIdent, NodeID, NodeKind, ParameterIdent, VariableExpr};

use super::{Context, ContextType, Parser};
use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;

impl Parser<'_> {
    /// Parse an identifier (dispatches based on context).
    ///
    /// This method parses a standard identifier token and creates the appropriate
    /// AST node based on the context. Identifiers are fundamental building blocks
    /// used for variable names, function names, class names, and more. The naming
    /// convention (private, constant, etc.) is determined by the identifier's
    /// name string, not by the token type.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// identifier: (letter | `_`) (letter | digit | `_`)*
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple identifier:
    ///
    /// ```python
    /// x
    /// ```
    ///
    /// Snake case identifier:
    ///
    /// ```python
    /// my_variable
    /// ```
    ///
    /// Camel case identifier:
    ///
    /// ```python
    /// myClassName
    /// ```
    ///
    /// Constant style (uppercase):
    ///
    /// ```python
    /// MAX_SIZE
    /// ```
    ///
    /// Private identifier (by convention):
    ///
    /// ```python
    /// _private_var
    /// ```
    ///
    /// Underscore only:
    ///
    /// ```python
    /// _
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The current token is not a valid identifier
    /// - The current token is not an underscore token
    pub fn parse_identifier(&mut self) -> ParseResult<NodeID> {
        // Accept both Identifier and Underscore tokens as identifiers
        if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
            let message = format!("Expected identifier, found {}", self.current.kind);

            return Err(self.error(&message));
        }

        // Get the identifier name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the identifier token
        self.skip();

        // Create the basic identifier node
        let identifier = BasicIdent::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::BasicIdent(identifier), span);

        Ok(node_id)
    }

    /// Parse a module name identifier (for import statements).
    ///
    /// Module identifiers are used in import statements to specify which
    /// module or package to import. They follow standard Python identifier
    /// rules but are used specifically in the context of module imports.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// module_identifier: identifier
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple module name:
    ///
    /// ```python
    /// import math
    /// ```
    ///
    /// Package submodule:
    ///
    /// ```python
    /// import os.path
    /// ```
    ///
    /// Import with alias:
    ///
    /// ```python
    /// import numpy as np
    /// ```
    ///
    /// From import:
    ///
    /// ```python
    /// from collections import defaultdict
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The current token is not an identifier
    /// - Expected module name is missing
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
        self.skip();

        // Create the identifier node
        let identifier = BasicIdent::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::BasicIdent(identifier), span);

        Ok(node_id)
    }

    /// Parse a parameter identifier (in function declarations).
    ///
    /// Parameter identifiers are used in function and method declarations to
    /// name the parameters. They can include type annotations and default
    /// values. This method handles the complete parameter syntax including
    /// optional type hints and defaults.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// parameter_identifier: identifier [`:` type_expression] [`=` expression]
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple parameter:
    ///
    /// ```python
    /// def func(x):
    ///     pass
    /// ```
    ///
    /// Parameter with type annotation:
    ///
    /// ```python
    /// def func(x: int):
    ///     pass
    /// ```
    ///
    /// Parameter with default value:
    ///
    /// ```python
    /// def func(x=10):
    ///     pass
    /// ```
    ///
    /// Parameter with type and default:
    ///
    /// ```python
    /// def func(x: int = 10):
    ///     pass
    /// ```
    ///
    /// Underscore parameter (ignored):
    ///
    /// ```python
    /// lambda v, _: v * 2
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The current token is not an identifier or underscore
    /// - The type annotation has syntax errors
    /// - The default value expression is invalid
    pub fn parse_parameter_identifier(&mut self) -> ParseResult<NodeID> {
        // Parse the parameter name
        // Accept both Identifier and Underscore tokens (e.g. `lambda v, _: ...`)
        if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
            return Err(self.error("Expected parameter name"));
        }

        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the parameter name
        self.skip();

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
            self.skip(); // Skip the colon

            // Push type annotation context
            self.context_stack.push(Context::new(
                ContextType::TypeAnnotation,
                None,
                self.context_stack.current_indent_level(),
            ));

            // Parse the type expression
            let type_annotation = self.parse_type_expression()?;
            parameter.type_annotation = Some(type_annotation);

            // Pop type annotation context
            drop(self.context_stack.pop());
        }

        // Check for default value
        if self.check(TokenKind::Assign) {
            self.skip(); // Skip the equals sign

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

    /// Parse a soft keyword as an identifier.
    ///
    /// Soft keywords like `type`, `match`, and `case` can be used as identifiers
    /// in expression contexts, but act as keywords in specific statement contexts.
    /// This allows backward compatibility with code that uses these names as
    /// variable names while still supporting the new syntax features.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// soft_keyword_identifier: `type` | `match` | `case`
    /// ```
    ///
    /// ## Examples
    ///
    /// Using 'type' as variable:
    ///
    /// ```python
    /// type = str  # 'type' used as identifier
    /// ```
    ///
    /// Using 'match' as variable:
    ///
    /// ```python
    /// match = re.search(pattern, text)
    /// ```
    ///
    /// Using 'case' as variable:
    ///
    /// ```python
    /// case = "upper"
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The current token is not `type`, `match`, or `case`
    pub fn parse_soft_keyword_as_identifier(&mut self) -> ParseResult<NodeID> {
        // Verify we have a soft keyword token
        if !self.matches(&[TokenKind::Type, TokenKind::Match, TokenKind::Case]) {
            return Err(self.error("Expected soft keyword"));
        }

        // Get the keyword name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the soft keyword token
        self.skip();

        // Create the variable identifier node (soft keywords act as regular names in expressions)
        let variable = VariableExpr::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Expression, AnyNode::VariableExpr(variable), span);

        Ok(node_id)
    }

    /// Parse a type identifier (name used in type annotations).
    ///
    /// Type identifiers are used in type annotations to reference types.
    /// They can be built-in types, user-defined classes, or type aliases.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// type_identifier: identifier
    /// ```
    ///
    /// ## Examples
    ///
    /// Built-in type:
    ///
    /// ```python
    /// x: int = 5
    /// ```
    ///
    /// User-defined class:
    ///
    /// ```python
    /// user: User = User()
    /// ```
    ///
    /// Generic type:
    ///
    /// ```python
    /// items: list[str] = []
    /// ```
    ///
    /// Optional type:
    ///
    /// ```python
    /// value: Optional[int] = None
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The current token is not an identifier
    /// - Expected type name is missing
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
        self.skip();

        // Create the identifier node
        let identifier = BasicIdent::new(name, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::BasicIdent(identifier), span);

        Ok(node_id)
    }

    /// Parse a variable identifier (standard name in expressions).
    ///
    /// Variable identifiers are the most common identifier type, used to
    /// reference variables in expressions. They can be simple names or
    /// underscores for ignored values.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// variable_identifier: identifier | `_`
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple variable:
    ///
    /// ```python
    /// x = 10
    /// result = x + 5
    /// ```
    ///
    /// Multiple assignment:
    ///
    /// ```python
    /// a, b, c = 1, 2, 3
    /// ```
    ///
    /// Ignoring values with underscore:
    ///
    /// ```python
    /// _, value = get_tuple()
    /// ```
    ///
    /// In expression:
    ///
    /// ```python
    /// total = sum(numbers)
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The current token is not an identifier
    /// - The current token is not an underscore
    pub fn parse_variable_identifier(&mut self) -> ParseResult<NodeID> {
        // Accept both Identifier and Underscore tokens
        if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
            return Err(self.error("Expected identifier"));
        }

        // Get the identifier name and span
        let token = self.current.clone();
        let name = token.lexeme.to_string();
        let span = self.token_to_span(&token);

        // Consume the identifier token
        self.skip();

        // Create the variable node
        let variable = VariableExpr { name, id: NodeID::placeholder(), parent: None, span };

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Expression, AnyNode::VariableExpr(variable), span);

        Ok(node_id)
    }
}
