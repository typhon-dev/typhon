//! Declaration parsing for the Typhon programming language.
//!
//! This module handles parsing declarations of all types,
//! including function definitions, class definitions, and type definitions.
//! It also handles parsing decorators for function and class declarations.

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
use super::context::{Context, ContextFlags, ContextType, FunctionModifiers};
use crate::diagnostics::ParseResult;
use crate::lexer::TokenKind;

impl Parser<'_> {
    /// Parse base classes for a class declaration.
    ///
    /// This parses a list of base classes separated by commas within parentheses.
    /// Base classes can be simple identifiers or complex expressions like attribute
    /// access or subscripted generics.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// base_classes: expression (`,` expression)* [`,`]
    /// ```
    ///
    /// ## Examples
    ///
    /// Single base class:
    ///
    /// ```python
    /// class Student(Person):
    ///     pass
    /// ```
    ///
    /// Multiple base classes:
    ///
    /// ```python
    /// class Employee(Person, Worker):
    ///     pass
    /// ```
    ///
    /// Generic base class:
    ///
    /// ```python
    /// class MyList(list[int]):
    ///     pass
    /// ```
    ///
    /// Module-qualified base class:
    ///
    /// ```python
    /// class MyException(exceptions.BaseException):
    ///     pass
    /// ```
    ///
    /// Trailing comma allowed:
    ///
    /// ```python
    /// class Multi(Base1, Base2, Base3,):
    ///     pass
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - A base class expression is invalid
    /// - Commas are missing between base classes
    /// - The closing parenthesis `)` is missing
    fn parse_base_classes(&mut self) -> ParseResult<Vec<NodeID>> {
        let mut bases = Vec::new();

        // Empty base class list
        if self.check(TokenKind::RightParen) {
            self.skip(); // Consume ')'
            return Ok(bases);
        }

        // Loop until we hit the closing parenthesis
        loop {
            // Parse base class expression (typically an identifier, but could be more complex)
            let base = self.parse_expression()?;
            bases.push(base);

            // Check if we have more base classes
            if self.check(TokenKind::Comma) {
                self.skip(); // Consume ','

                // If we see a right paren after a comma, it's a trailing comma
                if self.check(TokenKind::RightParen) {
                    self.skip(); // Consume ')'
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

    /// Parse a class declaration.
    ///
    /// Classes define new types with methods and attributes. They support
    /// single and multiple inheritance through base classes, and can be
    /// decorated with decorator expressions.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// class_def: [decorators] `class` identifier [`(` [base_classes] `)`] `:` suite
    /// base_classes: expression (`,` expression)* [`,`]
    /// decorators: (`@` expression NEWLINE)+
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple class definition:
    ///
    /// ```python
    /// class Point:
    ///     x: int
    ///     y: int
    /// ```
    ///
    /// Class with single inheritance:
    ///
    /// ```python
    /// class Student(Person):
    ///     student_id: str
    /// ```
    ///
    /// Class with multiple inheritance:
    ///
    /// ```python
    /// class Employee(Person, Worker):
    ///     employee_id: int
    /// ```
    ///
    /// Class with methods:
    ///
    /// ```python
    /// class Circle:
    ///     def __init__(self, radius: float):
    ///         self.radius = radius
    ///
    ///     def area(self) -> float:
    ///         return 3.14159 * self.radius ** 2
    /// ```
    ///
    /// Decorated class:
    ///
    /// ```python
    /// @dataclass
    /// @frozen
    /// class Config:
    ///     host: str
    ///     port: int
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `class` keyword is missing or misplaced
    /// - The class name (identifier) is missing
    /// - The base class list has syntax errors
    /// - The colon `:` before the body is missing
    /// - The class body is malformed
    pub(super) fn parse_class_declaration(&mut self) -> ParseResult<NodeID> {
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
        self.skip(); // Consume the identifier

        // Parse optional base classes
        let bases = if self.check(TokenKind::LeftParen) {
            self.skip(); // Consume '('
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

            self.get_node_span(*last_stmt)?.end
        };

        // Create the span for the class declaration
        let span = Span::new(start_pos, end_pos);

        // Get any pending decorators
        let decorators = std::mem::take(&mut self.context_stack.current_mut().decorator_stack);

        // Create the ClassDef node
        let mut class_def =
            ClassDecl::new(name, bases.clone(), body.clone(), NodeID::placeholder(), span);

        // Apply decorators if any
        if !decorators.is_empty() {
            class_def = class_def.with_decorators(decorators.clone());
        }

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::ClassDecl(class_def), span);

        // Set parent-child relationships
        for base in &bases {
            self.set_parent(*base, node_id);
        }

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        // Set parent relationship for decorators
        for decorator in &decorators {
            self.set_parent(*decorator, node_id);
        }

        // Pop the class context
        drop(self.context_stack.pop());

        Ok(node_id)
    }

    /// Parse a declaration.
    ///
    /// This is the main entry point for parsing declarations. It handles
    /// decorators and dispatches to the appropriate declaration parser
    /// (function, class, or type) based on the current token.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// declaration: [decorators] (function_def | class_def | type_def)
    /// decorators: (`@` expression NEWLINE)+
    /// ```
    ///
    /// ## Examples
    ///
    /// Function declaration:
    ///
    /// ```python
    /// def greet(name: str) -> str:
    ///     return f"Hello, {name}!"
    /// ```
    ///
    /// Decorated function:
    ///
    /// ```python
    /// @staticmethod
    /// @cache
    /// def compute(x: int) -> int:
    ///     return x * 2
    /// ```
    ///
    /// Class declaration:
    ///
    /// ```python
    /// class MyClass(BaseClass):
    ///     pass
    /// ```
    ///
    /// Type alias declaration:
    ///
    /// ```python
    /// type Vector = list[float]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - Decorators are present but not followed by a function or class
    /// - Type declarations have decorators (not allowed in Python)
    /// - No valid declaration keyword (`def`, `class`, `type`) is found
    /// - The specific declaration has syntax errors
    pub fn parse_declaration(&mut self) -> ParseResult<NodeID> {
        // Parse decorators if present
        while self.check(TokenKind::At) {
            let decorator = self.parse_decorator()?;
            self.context_stack.current_mut().decorator_stack.push(decorator);

            // Set the has_decorator flag
            self.context_stack.current_mut().flags.fn_modifiers.has_decorator = true;
        }

        // Check the current token to determine the type of declaration
        let result = match self.current_token().kind {
            TokenKind::Def => self.parse_function_declaration(),
            TokenKind::Class => self.parse_class_declaration(),
            TokenKind::Identifier
                if self.current_token().lexeme == "type"
                    && self.peek_token().kind == TokenKind::Identifier =>
            {
                // Type declarations can't have decorators in Python
                if !self.context_stack.current().decorator_stack.is_empty() {
                    return Err(self.error("Type declarations cannot have decorators"));
                }

                self.parse_type_declaration()
            }
            _ => {
                // If there are decorators but no valid declaration, report an error
                if self.context_stack.current().decorator_stack.is_empty() {
                    Err(self.error("Expected declaration (function, class, or type)"))
                } else {
                    Err(self.error("Expected function or class declaration after decorators"))
                }
            }
        };

        // Clear any pending decorators after parsing a declaration
        // (they're either consumed or an error was reported)
        self.context_stack.current_mut().decorator_stack.clear();

        result
    }

    /// Parse a decorator expression.
    ///
    /// Decorators modify the behavior of functions and classes. They can be
    /// simple identifiers, attribute access, or function calls that return
    /// decorator functions. Multiple decorators can be stacked and are applied
    /// in bottom-to-top order.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// decorator: `@` expression NEWLINE
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple decorator:
    ///
    /// ```python
    /// @staticmethod
    /// def my_method():
    ///     pass
    /// ```
    ///
    /// Decorator with arguments:
    ///
    /// ```python
    /// @decorator(arg1, arg2)
    /// def my_function():
    ///     pass
    /// ```
    ///
    /// Attribute decorator:
    ///
    /// ```python
    /// @module.decorator
    /// def my_function():
    ///     pass
    /// ```
    ///
    /// Decorator with keyword arguments:
    ///
    /// ```python
    /// @app.route('/users', methods=['GET', 'POST'])
    /// def users():
    ///     pass
    /// ```
    ///
    /// Multiple stacked decorators (applied bottom-to-top):
    ///
    /// ```python
    /// @decorator1
    /// @decorator2
    /// @decorator3
    /// def my_function():
    ///     pass
    /// ```
    ///
    /// Class decorator:
    ///
    /// ```python
    /// @dataclass
    /// @frozen
    /// class Config:
    ///     host: str
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `@` symbol is missing
    /// - The decorator expression is invalid
    /// - The newline after the decorator is missing
    fn parse_decorator(&mut self) -> ParseResult<NodeID> {
        // Consume the '@' token
        self.expect(TokenKind::At)?;

        // Parse the decorator expression
        let decorator = self.parse_expression()?;

        // Expect a newline after the decorator
        self.expect_statement_end()?;

        Ok(decorator)
    }

    /// Parse a function declaration.
    ///
    /// Functions can be synchronous or asynchronous, have parameters with
    /// type annotations and defaults, return type annotations, decorators,
    /// and a body of statements.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// function_def: [decorators] [`async`] `def` identifier `(` [parameters] `)` [`->` expression] `:` suite
    /// parameters: parameter (`,` parameter)* [`,`]
    ///           | `*` [identifier] [`,` parameter]* [`,` `**` identifier]
    ///           | `**` identifier
    /// parameter: identifier [`:` expression] [`=` expression]
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple function:
    ///
    /// ```python
    /// def add(a: int, b: int) -> int:
    ///     return a + b
    /// ```
    ///
    /// Function with default parameters:
    ///
    /// ```python
    /// def greet(name: str, prefix: str = "Hello") -> str:
    ///     return f"{prefix}, {name}!"
    /// ```
    ///
    /// Async function:
    ///
    /// ```python
    /// async def fetch_data(url: str) -> dict:
    ///     response = await client.get(url)
    ///     return await response.json()
    /// ```
    ///
    /// Function with *args and **kwargs:
    ///
    /// ```python
    /// def process(*args: int, **kwargs: str) -> None:
    ///     print(args, kwargs)
    /// ```
    ///
    /// Decorated function:
    ///
    /// ```python
    /// @property
    /// @cache
    /// def expensive_property(self) -> int:
    ///     return self.compute()
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `def` keyword is missing
    /// - The function name (identifier) is missing
    /// - The parameter list has syntax errors
    /// - The return type annotation has syntax errors
    /// - The colon `:` before the body is missing
    /// - The function body is malformed
    pub(super) fn parse_function_declaration(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Check for async keyword
        let is_async = if self.check(TokenKind::Async) {
            self.skip(); // Consume 'async'
            true
        } else {
            false
        };

        // Expect the 'def' keyword
        self.expect(TokenKind::Def)?;

        // Create a context for the function declaration
        let function_modifiers = FunctionModifiers { is_async, ..Default::default() };

        let context_flags = ContextFlags { fn_modifiers: function_modifiers, ..Default::default() };

        self.context_stack.push(
            Context::new(ContextType::Function, None, self.context_stack.current_indent_level())
                .with_flags(context_flags),
        );

        // Parse the function name (identifier)
        let name = if self.check(TokenKind::Identifier) {
            self.current_token().lexeme.to_string()
        } else {
            return Err(self.error("Expected function name"));
        };
        self.skip(); // Consume the identifier

        // Parse the parameter list
        self.expect(TokenKind::LeftParen)?;
        let parameters = self.parse_parameter_list()?;
        self.expect(TokenKind::RightParen)?;

        // Parse optional return type annotation
        let return_type = if self.check(TokenKind::Arrow) {
            self.skip(); // Consume '->'
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
                self.get_node_span(rt)?.end
            } else {
                self.current_token().span.end
            }
        } else {
            // Use the end of the last statement in the body
            let last_stmt = body.last().unwrap();

            self.get_node_span(*last_stmt)?.end
        };

        // Create the span for the function declaration
        let span = Span::new(start_pos, end_pos);

        // Get any pending decorators
        let decorators = std::mem::take(&mut self.context_stack.current_mut().decorator_stack);

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

            // Apply decorators if any
            if !decorators.is_empty() {
                async_function_def = async_function_def.with_decorators(decorators.clone());
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

            // Apply decorators if any
            if !decorators.is_empty() {
                function_def = function_def.with_decorators(decorators.clone());
            }

            // Allocate the node in the AST
            self.ast.alloc_node(NodeKind::Declaration, AnyNode::FunctionDecl(function_def), span)
        };

        // Set parent-child relationships
        for param in &parameters {
            self.set_parent(*param, node_id);
        }

        if let Some(rt) = return_type {
            self.set_parent(rt, node_id);
        }

        for stmt in &body {
            self.set_parent(*stmt, node_id);
        }

        // Set parent relationship for decorators
        for decorator in &decorators {
            self.set_parent(*decorator, node_id);
        }

        // Pop the function context
        drop(self.context_stack.pop());

        Ok(node_id)
    }

    /// Parse a parameter list for a function declaration.
    ///
    /// Function parameters can include positional, keyword-only (after `*`),
    /// variable positional (`*args`), and variable keyword (`**kwargs`) parameters.
    /// Each parameter can have type annotations and default values.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// parameter_list: [parameter (`,` parameter)*] [`,`]
    /// parameter: identifier [`:` type] [`=` default]
    ///          | `*` identifier [`:` type]
    ///          | `**` identifier [`:` type]
    ///          | `*` `,` parameter (`,` parameter)*
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple parameters:
    ///
    /// ```python
    /// def func(a, b, c):
    ///     pass
    /// ```
    ///
    /// Parameters with type annotations:
    ///
    /// ```python
    /// def func(x: int, y: str, z: float):
    ///     pass
    /// ```
    ///
    /// Parameters with defaults:
    ///
    /// ```python
    /// def func(a, b=10, c="default"):
    ///     pass
    /// ```
    ///
    /// Keyword-only parameters (after `*`):
    ///
    /// ```python
    /// def func(a, *, b, c=10):
    ///     pass
    /// ```
    ///
    /// Variable parameters:
    ///
    /// ```python
    /// def func(*args, **kwargs):
    ///     pass
    /// ```
    ///
    /// Complete example:
    ///
    /// ```python
    /// def func(a: int, b: str = "default", *args: int, c: bool, **kwargs: str):
    ///     pass
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - A parameter name is missing or invalid
    /// - Type annotations have syntax errors
    /// - Default value expressions are invalid
    /// - The parameter list structure is malformed
    fn parse_parameter_list(&mut self) -> ParseResult<Vec<NodeID>> {
        let mut parameters = Vec::new();

        // Skip any leading newlines (implicit line continuation in parentheses)
        self.skip_newlines();

        // Empty parameter list
        if self.check(TokenKind::RightParen) {
            return Ok(parameters);
        }

        // Loop until we hit the closing parenthesis
        loop {
            // Skip newlines (implicit line continuation in parentheses)
            self.skip_newlines();

            // Check for *args (variable positional arguments)
            if self.check(TokenKind::Star) && self.peek_token().kind == TokenKind::Identifier {
                let param_id = self.parse_var_positional_parameter()?;
                parameters.push(param_id);

                // Check for continuation
                if self.check(TokenKind::Comma) {
                    self.skip(); // Consume ','
                    if self.check(TokenKind::RightParen) {
                        break;
                    }
                } else {
                    break;
                }

                continue;
            }

            // Check for **kwargs (variable keyword arguments)
            if self.check(TokenKind::DoubleStar) && self.peek_token().kind == TokenKind::Identifier
            {
                let param_id = self.parse_var_keyword_parameter()?;
                parameters.push(param_id);

                // Skip newlines after parsing the parameter (implicit line continuation in parentheses)
                self.skip_newlines();

                // Check for continuation (trailing comma)
                if self.expect(TokenKind::Comma).is_ok() {
                    if self.check(TokenKind::RightParen) {
                        break;
                    }
                } else {
                    break;
                }

                continue;
            }

            // Check for bare '*' (keyword-only parameter marker)
            if self.check(TokenKind::Star) {
                self.skip(); // Consume '*'

                // After '*', if there's a comma, continue to next parameter
                if self.check(TokenKind::Comma) {
                    self.skip(); // Consume ','

                    // If we see a right paren after comma, it's a trailing comma
                    if self.check(TokenKind::RightParen) {
                        break;
                    }

                    // Continue to parse the next parameter (which must be keyword-only)
                    continue;
                } else if self.check(TokenKind::RightParen) {
                    // End of parameter list
                    break;
                }

                // The next token should be an identifier for a keyword-only parameter
                // Continue to parse it normally
            }

            // Get the start position of the parameter
            let start_pos = self.current_token().span.start;

            // Parse parameter name
            let name = if self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
                self.current_token().lexeme.to_string()
            } else {
                return Err(self.error("Expected parameter name"));
            };

            self.skip(); // Consume the parameter name

            // Parse optional type annotation
            let type_annotation = if self.check(TokenKind::Colon) {
                self.skip(); // Consume ':'

                // Push type annotation context
                self.context_stack.push(Context::new(
                    ContextType::TypeAnnotation,
                    None,
                    self.context_stack.current_indent_level(),
                ));

                let type_expr = self.parse_expression()?;

                // Pop type annotation context
                drop(self.context_stack.pop());

                Some(type_expr)
            } else {
                None
            };

            // Parse optional default value
            let default_value = if self.check(TokenKind::Assign) {
                self.skip(); // Consume '='

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
                (_, Some(def)) => self.get_node_span(def)?.end,
                (Some(typ), None) => self.get_node_span(typ)?.end,
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
                self.set_parent(typ, param_id);
            }

            if let Some(def) = default_value {
                self.set_parent(def, param_id);
            }

            // Add the parameter to the list
            parameters.push(param_id);

            // Check if we have more parameters
            if self.check(TokenKind::Comma) {
                self.skip(); // Consume ','

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

    /// Parse a type declaration (type alias).
    ///
    /// Type declarations create aliases for existing types, making code
    /// more readable and maintainable. They are similar to `typedef` in C
    /// or type aliases in TypeScript.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// type_def: `type` identifier `=` expression
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple type alias:
    ///
    /// ```python
    /// type Vector = list[float]
    /// ```
    ///
    /// Complex type alias:
    ///
    /// ```python
    /// type Callback = Callable[[int, str], bool]
    /// ```
    ///
    /// Union type alias:
    ///
    /// ```python
    /// type Result = Success | Error
    /// ```
    ///
    /// Generic type alias:
    ///
    /// ```python
    /// type OptionalList[T] = list[T] | None
    /// ```
    ///
    /// Nested type alias:
    ///
    /// ```python
    /// type Matrix = list[list[float]]
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `type` keyword is missing
    /// - The type name (identifier) is missing
    /// - The `=` assignment operator is missing
    /// - The type expression is invalid
    /// - The statement terminator is missing
    pub(super) fn parse_type_declaration(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'type' keyword (which is an identifier)
        self.skip();

        // Parse the type name (identifier)
        let name = if self.check(TokenKind::Identifier) {
            self.current_token().lexeme.to_string()
        } else {
            return Err(self.error("Expected type name"));
        };

        self.skip(); // Consume the identifier

        // Expect the '=' token
        self.expect(TokenKind::Assign)?;

        // Push type annotation context
        self.context_stack.push(Context::new(
            ContextType::TypeAnnotation,
            None,
            self.context_stack.current_indent_level(),
        ));

        // Parse the original type expression
        let original_type = self.parse_expression()?;

        // Pop type annotation context
        drop(self.context_stack.pop());

        // Get the end position
        let end_pos = self.get_node_span(original_type)?.end;

        // Create the span for the type declaration
        let span = Span::new(start_pos, end_pos);

        // Create the TypeDef node
        let type_def = TypeDecl::new(name, original_type, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Declaration, AnyNode::TypeDecl(type_def), span);

        // Set parent-child relationship
        self.set_parent(original_type, node_id);

        // Expect a newline or semicolon after the statement
        self.expect_statement_end()?;

        Ok(node_id)
    }

    /// Parse a `**kwargs` parameter (variable keyword arguments).
    ///
    /// Variable keyword parameters capture all remaining keyword arguments
    /// that weren't matched by other parameters. They are collected into
    /// a dictionary within the function.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// var_keyword_param: `**` identifier [`:` type]
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple **kwargs parameter:
    ///
    /// ```python
    /// def func(**kwargs):
    ///     pass
    /// ```
    ///
    /// **kwargs with type annotation:
    ///
    /// ```python
    /// def func(**kwargs: str):
    ///     pass
    /// ```
    ///
    /// Combined with other parameters:
    ///
    /// ```python
    /// def func(a: int, b: str = "default", *args, **kwargs):
    ///     pass
    /// ```
    ///
    /// **kwargs in async function:
    ///
    /// ```python
    /// async def fetch(**options: Any):
    ///     pass
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `**` token is missing or malformed
    /// - The parameter name (identifier) is missing
    /// - The type annotation has syntax errors
    fn parse_var_keyword_parameter(&mut self) -> ParseResult<NodeID> {
        // Consume '**'
        let double_star = self.consume(TokenKind::DoubleStar)?;

        // Consume the identifier
        let ident = self.consume(TokenKind::Identifier)?;

        // **kwargs can have type annotations but not defaults
        let type_annotation = if self.check(TokenKind::Colon) {
            self.skip(); // Consume ':'

            // Push type annotation context
            self.context_stack.push(Context::new(
                ContextType::TypeAnnotation,
                None,
                self.context_stack.current_indent_level(),
            ));

            let type_expr = self.parse_expression()?;

            // Skip newlines after type annotation (implicit line continuation in parentheses)
            self.skip_newlines();

            // Pop type annotation context
            drop(self.context_stack.pop());

            Some(type_expr)
        } else {
            None
        };

        let end_pos = match type_annotation {
            Some(typ) => self.get_node_span(typ)?.end,
            None => self.current_token().span.start,
        };
        let span = Span::new(double_star.span().start, end_pos);

        let mut param =
            ParameterIdent::new(ident.lexeme().to_string(), NodeID::placeholder(), span);
        if let Some(typ) = type_annotation {
            param = param.with_type(typ);
        }

        let param_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::ParameterIdent(param), span);
        if let Some(typ) = type_annotation {
            self.set_parent(typ, param_id);
        }

        Ok(param_id)
    }

    /// Parse a `*args` parameter (variable positional arguments).
    ///
    /// Variable positional parameters capture all remaining positional
    /// arguments that weren't matched by other parameters. They are
    /// collected into a tuple within the function.
    ///
    /// ## Grammar
    ///
    /// ```ebnf
    /// var_positional_param: `*` identifier [`:` type]
    /// ```
    ///
    /// ## Examples
    ///
    /// Simple *args parameter:
    ///
    /// ```python
    /// def func(*args):
    ///     pass
    /// ```
    ///
    /// *args with type annotation:
    ///
    /// ```python
    /// def func(*args: int):
    ///     pass
    /// ```
    ///
    /// Combined with other parameters:
    ///
    /// ```python
    /// def func(a: int, *args: str, b: bool = True):
    ///     pass
    /// ```
    ///
    /// *args before keyword-only parameters:
    ///
    /// ```python
    /// def func(*args, kwonly: str):
    ///     pass
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`ParseError`] if:
    ///
    /// - The `*` token is missing or malformed
    /// - The parameter name (identifier) is missing
    /// - The type annotation has syntax errors
    fn parse_var_positional_parameter(&mut self) -> ParseResult<NodeID> {
        // Consume '*'
        let star = self.consume(TokenKind::Star)?;

        // Consume the identifier
        let ident = self.consume(TokenKind::Identifier)?;

        // *args can optionally have type annotations in Python 3
        let type_annotation = if self.check(TokenKind::Colon) {
            self.skip(); // Consume ':'

            // Push type annotation context
            self.context_stack.push(Context::new(
                ContextType::TypeAnnotation,
                None,
                self.context_stack.current_indent_level(),
            ));

            let type_expr = self.parse_expression()?;

            // Skip newlines after type annotation (implicit line continuation in parentheses)
            // Do this BEFORE restoring the flag so skip_newlines can use the flag if needed
            self.skip_newlines();

            // Pop type annotation context
            drop(self.context_stack.pop());

            Some(type_expr)
        } else {
            None
        };

        let end_pos = match type_annotation {
            Some(typ) => self.get_node_span(typ)?.end,
            None => self.current_token().span.start,
        };
        let span = Span::new(star.span().start, end_pos);

        let mut param =
            ParameterIdent::new(ident.lexeme().to_string(), NodeID::placeholder(), span);
        if let Some(typ) = type_annotation {
            param = param.with_type(typ);
        }

        let param_id =
            self.ast.alloc_node(NodeKind::Identifier, AnyNode::ParameterIdent(param), span);

        if let Some(typ) = type_annotation {
            self.set_parent(typ, param_id);
        }

        Ok(param_id)
    }
}
