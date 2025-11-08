// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/frontend/parser/mod.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! Parser implementation for the Typhon programming language.

pub mod error;

use self::error::{
    ParseError,
    ParseResult,
};
use crate::common::SourceInfo;
use crate::frontend::ast::{
    BinaryOperator,
    Expression,
    Identifier,
    Literal,
    Module,
    Parameter,
    Statement,
    TypeExpression,
    UnaryOperator,
};
use crate::frontend::lexer::Lexer;
use crate::frontend::lexer::token::{
    Token,
    TokenKind,
};

/// Parser for the Typhon programming language
pub struct Parser<'a> {
    /// Lexer that produces tokens
    lexer: Lexer<'a>,
    /// Current token
    current: Option<Token>,
    /// Previous token
    previous: Option<Token>,
    /// Source code
    source: &'a str,
    /// Flag indicating if there was an error
    had_error: bool,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source code.
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next();

        Self {
            lexer,
            current,
            previous: None,
            source,
            had_error: false,
        }
    }

    /// Parses the source code into a module.
    pub fn parse(&mut self) -> ParseResult<Module> {
        let source_info = SourceInfo::new((0..self.source.len()).into());
        let name = String::new(); // Module name will be set elsewhere

        let mut statements = Vec::new();

        while self.current.is_some() && self.current.unwrap().kind != TokenKind::Eof {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(Module::new(name, statements, source_info))
    }

    /// Advances the parser to the next token.
    fn advance(&mut self) -> Option<Token> {
        self.previous = self.current;
        self.current = self.lexer.next();
        self.previous
    }

    /// Returns the current token.
    fn current(&self) -> Option<Token> {
        self.current
    }

    /// Returns the previous token.
    fn previous(&self) -> Option<Token> {
        self.previous
    }

    /// Checks if the current token has the given kind.
    fn check(&self, kind: TokenKind) -> bool {
        self.current.is_some_and(|token| token.kind == kind)
    }

    /// Matches the current token against the given kinds.
    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for &kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Consumes the current token if it has the given kind, otherwise returns an error.
    fn consume(&mut self, kind: TokenKind, _message: &str) -> ParseResult<Token> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            let token = self.current.unwrap_or_else(|| self.previous.unwrap());
            let line = self.lexer.line();
            let column = self.lexer.column();

            Err(ParseError::unexpected_token(
                token,
                vec![kind],
                line,
                column,
            ))
        }
    }

    /// Synchronizes the parser after an error.
    fn synchronize(&mut self) {
        self.had_error = true;

        while let Some(token) = self.current {
            if self
                .previous
                .is_some_and(|t| t.kind == TokenKind::Semicolon)
            {
                return;
            }

            match token.kind {
                TokenKind::Class
                | TokenKind::Def
                | TokenKind::For
                | TokenKind::If
                | TokenKind::Return
                | TokenKind::While => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parses a statement.
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        if self.match_token(&[TokenKind::Def]) {
            self.parse_function_definition()
        } else if self.match_token(&[TokenKind::Class]) {
            self.parse_class_definition()
        } else if self.match_token(&[TokenKind::If]) {
            self.parse_if_statement()
        } else if self.match_token(&[TokenKind::While]) {
            self.parse_while_statement()
        } else if self.match_token(&[TokenKind::For]) {
            self.parse_for_statement()
        } else if self.match_token(&[TokenKind::Return]) {
            self.parse_return_statement()
        } else if self.match_token(&[TokenKind::Import]) {
            self.parse_import_statement()
        } else if self.match_token(&[TokenKind::From]) {
            self.parse_from_import_statement()
        } else if self.match_token(&[TokenKind::Pass]) {
            let token = self.previous.unwrap();
            let source_info = SourceInfo::new(token.span);
            Ok(Statement::Pass { source_info })
        } else if self.match_token(&[TokenKind::Break]) {
            let token = self.previous.unwrap();
            let source_info = SourceInfo::new(token.span);
            Ok(Statement::Break { source_info })
        } else if self.match_token(&[TokenKind::Continue]) {
            let token = self.previous.unwrap();
            let source_info = SourceInfo::new(token.span);
            Ok(Statement::Continue { source_info })
        } else {
            // Check if this is a variable declaration (identifier followed by colon or equals)
            if let Some(token) = self.current {
                if token.kind == TokenKind::Identifier {
                    let peek_result = self.lexer.peek();
                    if peek_result
                        .is_some_and(|t| t.kind == TokenKind::Colon || t.kind == TokenKind::Assign)
                    {
                        return self.parse_variable_declaration(false); // Default to immutable
                    }
                }
            }

            // Expression statement or assignment
            let expr = self.parse_expression()?;

            if self.match_token(&[TokenKind::Assign]) {
                // Assignment statement
                let value = self.parse_expression()?;
                let span = expr.source_info().span.start..value.source_info().span.end;
                let source_info = SourceInfo::new(span.into());

                Ok(Statement::Assignment {
                    target: expr,
                    value,
                    source_info,
                })
            } else {
                // Expression statement
                Ok(Statement::Expression(expr))
            }
        }
    }

    /// Parses a variable declaration.
    fn parse_variable_declaration(&mut self, _mutable: bool) -> ParseResult<Statement> {
        // Parse the identifier
        let name_token = self.consume(TokenKind::Identifier, "Expect variable name.")?;
        let name = Identifier::new(
            self.lexer.slice(name_token.span).to_string(),
            SourceInfo::new(name_token.span),
        );

        // Check for type annotation
        let mut type_annotation = None;
        if self.match_token(&[TokenKind::Colon]) {
            type_annotation = Some(self.parse_type_expression()?);
        }

        // Check for assignment
        let mut value = None;
        if self.match_token(&[TokenKind::Assign]) {
            value = Some(Box::new(self.parse_expression()?));
        }

        // Calculate the source span
        let end = if let Some(ref val) = value {
            val.source_info().span.end
        } else if let Some(ref ty) = type_annotation {
            ty.source_info().span.end
        } else {
            name.source_info.span.end
        };

        let source_info = SourceInfo::new((name.source_info.span.start..end).into());

        Ok(Statement::VariableDecl {
            name,
            type_annotation,
            value,
            mutable: true, // All variables are mutable by default in Python-style
            source_info,
        })
    }

    /// Parses a function definition.
    fn parse_function_definition(&mut self) -> ParseResult<Statement> {
        let name_token = self.consume(TokenKind::Identifier, "Expect function name.")?;
        let name = Identifier::new(
            self.lexer.slice(name_token.span).to_string(),
            SourceInfo::new(name_token.span),
        );

        self.consume(TokenKind::LeftParen, "Expect '(' after function name.")?;

        let mut parameters = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(ParseError::invalid_syntax(
                        "Cannot have more than 255 parameters.",
                        self.current.unwrap().span,
                    ));
                }

                let param_name_token =
                    self.consume(TokenKind::Identifier, "Expect parameter name.")?;
                let param_name = Identifier::new(
                    self.lexer.slice(param_name_token.span).to_string(),
                    SourceInfo::new(param_name_token.span),
                );

                let mut type_annotation = None;
                if self.match_token(&[TokenKind::Colon]) {
                    type_annotation = Some(self.parse_type_expression()?);
                }

                let mut default_value = None;
                if self.match_token(&[TokenKind::Assign]) {
                    default_value = Some(Box::new(self.parse_expression()?));
                }

                let param_end = if let Some(ref val) = default_value {
                    val.source_info().span.end
                } else if let Some(ref ty) = type_annotation {
                    ty.source_info().span.end
                } else {
                    param_name.source_info.span.end
                };

                let param_source_info =
                    SourceInfo::new((param_name.source_info.span.start..param_end).into());

                parameters.push(Parameter::new(
                    param_name,
                    type_annotation,
                    default_value,
                    param_source_info,
                ));

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;

        let mut return_type = None;
        if self.match_token(&[TokenKind::Arrow]) {
            return_type = Some(self.parse_type_expression()?);
        }

        self.consume(TokenKind::Colon, "Expect ':' after function header.")?;

        let body = self.parse_block()?;

        let body_end = if let Some(last) = body.last() {
            match last {
                Statement::Expression(expr) => expr.source_info().span.end,
                Statement::Assignment { source_info, .. }
                | Statement::FunctionDef { source_info, .. }
                | Statement::ClassDef { source_info, .. }
                | Statement::Return { source_info, .. }
                | Statement::Import { source_info, .. }
                | Statement::FromImport { source_info, .. }
                | Statement::If { source_info, .. }
                | Statement::While { source_info, .. }
                | Statement::For { source_info, .. }
                | Statement::Pass { source_info }
                | Statement::Break { source_info }
                | Statement::Continue { source_info }
                | Statement::VariableDecl { source_info, .. } => source_info.span.end,
            }
        } else {
            return_type
                .as_ref()
                .map_or_else(|| name.source_info.span.end, |ty| ty.source_info().span.end)
        };

        let source_info = SourceInfo::new((name.source_info.span.start..body_end).into());

        Ok(Statement::FunctionDef {
            name,
            parameters,
            return_type,
            body,
            source_info,
        })
    }

    /// Parses a class definition.
    fn parse_class_definition(&mut self) -> ParseResult<Statement> {
        let name_token = self.consume(TokenKind::Identifier, "Expect class name.")?;
        let name = Identifier::new(
            self.lexer.slice(name_token.span).to_string(),
            SourceInfo::new(name_token.span),
        );

        let mut bases = Vec::new();
        if self.match_token(&[TokenKind::LeftParen]) {
            if !self.check(TokenKind::RightParen) {
                loop {
                    let base = self.parse_expression()?;
                    bases.push(base);

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.consume(TokenKind::RightParen, "Expect ')' after base classes.")?;
        }

        self.consume(TokenKind::Colon, "Expect ':' after class header.")?;

        let body = self.parse_block()?;

        let body_end = if let Some(last) = body.last() {
            match last {
                Statement::Expression(expr) => expr.source_info().span.end,
                Statement::Assignment { source_info, .. }
                | Statement::FunctionDef { source_info, .. }
                | Statement::ClassDef { source_info, .. }
                | Statement::Return { source_info, .. }
                | Statement::Import { source_info, .. }
                | Statement::FromImport { source_info, .. }
                | Statement::If { source_info, .. }
                | Statement::While { source_info, .. }
                | Statement::For { source_info, .. }
                | Statement::Pass { source_info }
                | Statement::Break { source_info }
                | Statement::Continue { source_info }
                | Statement::VariableDecl { source_info, .. } => source_info.span.end,
            }
        } else {
            name.source_info.span.end
        };

        let source_info = SourceInfo::new((name.source_info.span.start..body_end).into());

        Ok(Statement::ClassDef {
            name,
            bases,
            body,
            source_info,
        })
    }

    /// Parses an if statement.
    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        let condition = self.parse_expression()?;

        self.consume(TokenKind::Colon, "Expect ':' after if condition.")?;

        let body = self.parse_block()?;

        let mut else_body = None;
        let mut source_info_end = if let Some(last) = body.last() {
            match last {
                Statement::Expression(expr) => expr.source_info().span.end,
                Statement::Assignment { source_info, .. }
                | Statement::FunctionDef { source_info, .. }
                | Statement::ClassDef { source_info, .. }
                | Statement::Return { source_info, .. }
                | Statement::Import { source_info, .. }
                | Statement::FromImport { source_info, .. }
                | Statement::If { source_info, .. }
                | Statement::While { source_info, .. }
                | Statement::For { source_info, .. }
                | Statement::Pass { source_info }
                | Statement::Break { source_info }
                | Statement::Continue { source_info }
                | Statement::VariableDecl { source_info, .. } => source_info.span.end,
            }
        } else {
            condition.source_info().span.end
        };

        if self.match_token(&[TokenKind::Else]) {
            self.consume(TokenKind::Colon, "Expect ':' after 'else'.")?;

            else_body = Some(self.parse_block()?);

            if let Some(last) = else_body.as_ref().unwrap().last() {
                source_info_end = match last {
                    Statement::Expression(expr) => expr.source_info().span.end,
                    Statement::Assignment { source_info, .. }
                    | Statement::FunctionDef { source_info, .. }
                    | Statement::ClassDef { source_info, .. }
                    | Statement::Return { source_info, .. }
                    | Statement::Import { source_info, .. }
                    | Statement::FromImport { source_info, .. }
                    | Statement::If { source_info, .. }
                    | Statement::While { source_info, .. }
                    | Statement::For { source_info, .. }
                    | Statement::Pass { source_info }
                    | Statement::Break { source_info }
                    | Statement::Continue { source_info }
                    | Statement::VariableDecl { source_info, .. } => source_info.span.end,
                };
            }
        }

        let source_info =
            SourceInfo::new((condition.source_info().span.start..source_info_end).into());

        Ok(Statement::If {
            condition,
            body,
            else_body,
            source_info,
        })
    }

    /// Parses a while statement.
    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        let condition = self.parse_expression()?;

        self.consume(TokenKind::Colon, "Expect ':' after while condition.")?;

        let body = self.parse_block()?;

        let body_end = if let Some(last) = body.last() {
            match last {
                Statement::Expression(expr) => expr.source_info().span.end,
                Statement::Assignment { source_info, .. }
                | Statement::FunctionDef { source_info, .. }
                | Statement::ClassDef { source_info, .. }
                | Statement::Return { source_info, .. }
                | Statement::Import { source_info, .. }
                | Statement::FromImport { source_info, .. }
                | Statement::If { source_info, .. }
                | Statement::While { source_info, .. }
                | Statement::For { source_info, .. }
                | Statement::Pass { source_info }
                | Statement::Break { source_info }
                | Statement::Continue { source_info }
                | Statement::VariableDecl { source_info, .. } => source_info.span.end,
            }
        } else {
            condition.source_info().span.end
        };

        let source_info = SourceInfo::new((condition.source_info().span.start..body_end).into());

        Ok(Statement::While {
            condition,
            body,
            source_info,
        })
    }

    /// Parses a for statement.
    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        let target = self.parse_expression()?;

        self.consume(TokenKind::In, "Expect 'in' after for target.")?;

        let iter = self.parse_expression()?;

        self.consume(TokenKind::Colon, "Expect ':' after for header.")?;

        let body = self.parse_block()?;

        let body_end = if let Some(last) = body.last() {
            match last {
                Statement::Expression(expr) => expr.source_info().span.end,
                Statement::Assignment { source_info, .. }
                | Statement::FunctionDef { source_info, .. }
                | Statement::ClassDef { source_info, .. }
                | Statement::Return { source_info, .. }
                | Statement::Import { source_info, .. }
                | Statement::FromImport { source_info, .. }
                | Statement::If { source_info, .. }
                | Statement::While { source_info, .. }
                | Statement::For { source_info, .. }
                | Statement::Pass { source_info }
                | Statement::Break { source_info }
                | Statement::Continue { source_info }
                | Statement::VariableDecl { source_info, .. } => source_info.span.end,
            }
        } else {
            iter.source_info().span.end
        };

        let source_info = SourceInfo::new((target.source_info().span.start..body_end).into());

        Ok(Statement::For {
            target,
            iter,
            body,
            source_info,
        })
    }

    /// Parses a return statement.
    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        let keyword = self.previous.unwrap();
        let mut value = None;

        if !self.check(TokenKind::Newline) && !self.check(TokenKind::Eof) {
            value = Some(Box::new(self.parse_expression()?));
        }

        let end = value
            .as_ref()
            .map_or(keyword.span.end, |expr| expr.source_info().span.end);

        let source_info = SourceInfo::new((keyword.span.start..end).into());

        Ok(Statement::Return { value, source_info })
    }

    /// Parses an import statement.
    fn parse_import_statement(&mut self) -> ParseResult<Statement> {
        let keyword = self.previous.unwrap();
        let mut names = Vec::new();

        loop {
            let name_token = self.consume(TokenKind::Identifier, "Expect module name.")?;
            let name = Identifier::new(
                self.lexer.slice(name_token.span).to_string(),
                SourceInfo::new(name_token.span),
            );

            let mut as_name = None;
            if self.match_token(&[TokenKind::As]) {
                let as_name_token =
                    self.consume(TokenKind::Identifier, "Expect identifier after 'as'.")?;
                as_name = Some(Identifier::new(
                    self.lexer.slice(as_name_token.span).to_string(),
                    SourceInfo::new(as_name_token.span),
                ));
            }

            names.push((name, as_name));

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        let end = names.last().map_or(keyword.span.end, |(name, as_name)| {
            as_name
                .as_ref()
                .map_or(name.source_info.span.end, |n| n.source_info.span.end)
        });

        let source_info = SourceInfo::new((keyword.span.start..end).into());

        Ok(Statement::Import { names, source_info })
    }

    /// Parses a from-import statement.
    fn parse_from_import_statement(&mut self) -> ParseResult<Statement> {
        let keyword = self.previous.unwrap();

        let mut level = 0;
        while self.match_token(&[TokenKind::Dot]) {
            level += 1;
        }

        let module_token = self.consume(TokenKind::Identifier, "Expect module name.")?;
        let module = Identifier::new(
            self.lexer.slice(module_token.span).to_string(),
            SourceInfo::new(module_token.span),
        );

        self.consume(TokenKind::Import, "Expect 'import' after module name.")?;

        let mut names = Vec::new();

        if self.match_token(&[TokenKind::Star]) {
            let star_token = self.previous.unwrap();
            let name = Identifier::new("*".to_string(), SourceInfo::new(star_token.span));

            names.push((name, None));
        } else {
            loop {
                let name_token =
                    self.consume(TokenKind::Identifier, "Expect identifier after 'import'.")?;
                let name = Identifier::new(
                    self.lexer.slice(name_token.span).to_string(),
                    SourceInfo::new(name_token.span),
                );

                let mut as_name = None;
                if self.match_token(&[TokenKind::As]) {
                    let as_name_token =
                        self.consume(TokenKind::Identifier, "Expect identifier after 'as'.")?;
                    as_name = Some(Identifier::new(
                        self.lexer.slice(as_name_token.span).to_string(),
                        SourceInfo::new(as_name_token.span),
                    ));
                }

                names.push((name, as_name));

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        let end = names
            .last()
            .map_or(module.source_info.span.end, |(name, as_name)| {
                as_name
                    .as_ref()
                    .map_or(name.source_info.span.end, |n| n.source_info.span.end)
            });

        let source_info = SourceInfo::new((keyword.span.start..end).into());

        Ok(Statement::FromImport {
            module,
            names,
            level,
            source_info,
        })
    }

    /// Parses a block of statements.
    fn parse_block(&mut self) -> ParseResult<Vec<Statement>> {
        let mut statements = Vec::new();

        // Expect an INDENT token
        self.consume(TokenKind::Indent, "Expect indent after ':'.")?;

        // Parse statements until we see a DEDENT token
        while self.current.is_some() && self.current.unwrap().kind != TokenKind::Dedent {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        // Expect a DEDENT token
        self.consume(TokenKind::Dedent, "Expect dedent at end of block.")?;

        Ok(statements)
    }

    /// Parses an expression.
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment()
    }

    /// Parses an assignment expression.
    fn parse_assignment(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_or()?;

        if self.match_token(&[TokenKind::Assign]) {
            let equals = self.previous.unwrap();
            let _value = self.parse_assignment()?;

            // Only variable and attribute expressions can be assigned to.
            if let Expression::Variable { .. } | Expression::Attribute { .. } = expr {
                return Ok(expr);
            }

            return Err(ParseError::invalid_syntax(
                "Invalid assignment target.",
                equals.span,
            ));
        }

        Ok(expr)
    }

    /// Parses an or expression.
    fn parse_or(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_and()?;

        while self.match_token(&[TokenKind::Or]) {
            let operator = BinaryOperator::Or;
            let right = self.parse_and()?;
            let span = expr.source_info().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
                source_info,
            };
        }

        Ok(expr)
    }

    /// Parses an and expression.
    fn parse_and(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&[TokenKind::And]) {
            let operator = BinaryOperator::And;
            let right = self.parse_equality()?;
            let span = expr.source_info().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
                source_info,
            };
        }

        Ok(expr)
    }

    /// Parses an equality expression.
    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.match_token(&[TokenKind::Equal, TokenKind::NotEqual]) {
            let operator = match self.previous.unwrap().kind {
                TokenKind::Equal => BinaryOperator::Eq,
                TokenKind::NotEqual => BinaryOperator::NotEq,
                _ => unreachable!(),
            };

            let right = self.parse_comparison()?;
            let span = expr.source_info().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
                source_info,
            };
        }

        Ok(expr)
    }

    /// Parses a comparison expression.
    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_term()?;

        while self.match_token(&[
            TokenKind::LessThan,
            TokenKind::GreaterThan,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
        ]) {
            let operator = match self.previous.unwrap().kind {
                TokenKind::LessThan => BinaryOperator::Lt,
                TokenKind::GreaterThan => BinaryOperator::Gt,
                TokenKind::LessEqual => BinaryOperator::LtE,
                TokenKind::GreaterEqual => BinaryOperator::GtE,
                _ => unreachable!(),
            };

            let right = self.parse_term()?;
            let span = expr.source_info().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
                source_info,
            };
        }

        Ok(expr)
    }

    /// Parses a term expression.
    fn parse_term(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = match self.previous.unwrap().kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                _ => unreachable!(),
            };

            let right = self.parse_factor()?;
            let span = expr.source_info().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
                source_info,
            };
        }

        Ok(expr)
    }

    /// Parses a factor expression.
    fn parse_factor(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_token(&[
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::DoubleSlash,
        ]) {
            let operator = match self.previous.unwrap().kind {
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                TokenKind::Percent => BinaryOperator::Mod,
                TokenKind::DoubleSlash => BinaryOperator::FloorDiv,
                _ => unreachable!(),
            };

            let right = self.parse_unary()?;
            let span = expr.source_info().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
                source_info,
            };
        }

        Ok(expr)
    }

    /// Parses a unary expression.
    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if self.match_token(&[
            TokenKind::Not,
            TokenKind::Minus,
            TokenKind::Plus,
            TokenKind::Tilde,
        ]) {
            let operator = match self.previous.unwrap().kind {
                TokenKind::Not => UnaryOperator::Invert,
                TokenKind::Minus => UnaryOperator::Neg,
                TokenKind::Plus => UnaryOperator::Pos,
                TokenKind::Tilde => UnaryOperator::Not,
                _ => unreachable!(),
            };

            let right = self.parse_unary()?;
            let span = self.previous.unwrap().span.start..right.source_info().span.end;
            let source_info = SourceInfo::new(span.into());

            return Ok(Expression::UnaryOp {
                op: operator,
                operand: Box::new(right),
                source_info,
            });
        }

        self.parse_primary()
    }

    /// Parses a primary expression.
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        let token = self.current.ok_or_else(|| {
            ParseError::unexpected_eof(vec![
                TokenKind::Identifier,
                TokenKind::IntLiteral,
                TokenKind::FloatLiteral,
                TokenKind::StringLiteral,
            ])
        })?;

        match token.kind {
            TokenKind::Identifier => {
                self.advance();
                let name = Identifier::new(
                    self.lexer.slice(token.span).to_string(),
                    SourceInfo::new(token.span),
                );
                let source_info = SourceInfo::new(token.span);

                Ok(Expression::Variable { name, source_info })
            }
            TokenKind::IntLiteral => {
                self.advance();
                let text = self.lexer.slice(token.span);
                let value = text.replace('_', "").parse().map_err(|_| {
                    ParseError::invalid_syntax("Invalid integer literal.", token.span)
                })?;

                let source_info = SourceInfo::new(token.span);

                Ok(Expression::Literal {
                    value: Literal::Int(value),
                    source_info,
                })
            }
            TokenKind::FloatLiteral => {
                self.advance();
                let text = self.lexer.slice(token.span);
                let value = text.replace('_', "").parse().map_err(|_| {
                    ParseError::invalid_syntax("Invalid float literal.", token.span)
                })?;

                let source_info = SourceInfo::new(token.span);

                Ok(Expression::Literal {
                    value: Literal::Float(value),
                    source_info,
                })
            }
            TokenKind::StringLiteral | TokenKind::StringLiteral2 => {
                self.advance();
                let text = self.lexer.slice(token.span);
                let value = text[1..text.len() - 1].to_string(); // Remove quotes

                let source_info = SourceInfo::new(token.span);

                Ok(Expression::Literal {
                    value: Literal::String(value),
                    source_info,
                })
            }
            _ => Err(ParseError::unexpected_token(
                token,
                vec![
                    TokenKind::Identifier,
                    TokenKind::IntLiteral,
                    TokenKind::FloatLiteral,
                    TokenKind::StringLiteral,
                ],
                self.lexer.line(),
                self.lexer.column(),
            )),
        }
    }

    /// Parses a type expression.
    fn parse_type_expression(&mut self) -> ParseResult<TypeExpression> {
        // Start with a simple named type
        let token = self.consume(TokenKind::Identifier, "Expect type name.")?;
        let name = Identifier::new(
            self.lexer.slice(token.span).to_string(),
            SourceInfo::new(token.span),
        );
        let source_info = SourceInfo::new(token.span);

        let mut type_expr = TypeExpression::Named { name, source_info };

        // Check for generic type parameters
        if self.match_token(&[TokenKind::LeftBracket]) {
            let mut args = Vec::new();

            if !self.check(TokenKind::RightBracket) {
                loop {
                    let arg = self.parse_type_expression()?;
                    args.push(arg);

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.consume(
                TokenKind::RightBracket,
                "Expect ']' after generic type arguments.",
            )?;

            let base_source_info = *type_expr.source_info();
            let base = Box::new(type_expr);
            let end = self.previous.unwrap().span.end;
            let span = base_source_info.span.start..end;
            let source_info = SourceInfo::new(span.into());

            type_expr = TypeExpression::Generic {
                base,
                args,
                source_info,
            };
        }

        Ok(type_expr)
    }
}

impl Expression {
    /// Returns the source information of the expression.
    fn source_info(&self) -> &SourceInfo {
        match self {
            Expression::BinaryOp { source_info, .. } => source_info,
            Expression::UnaryOp { source_info, .. } => source_info,
            Expression::Literal { source_info, .. } => source_info,
            Expression::Variable { source_info, .. } => source_info,
            Expression::Attribute { source_info, .. } => source_info,
            Expression::Subscript { source_info, .. } => source_info,
            Expression::Call { source_info, .. } => source_info,
            Expression::Lambda { source_info, .. } => source_info,
            Expression::List { source_info, .. } => source_info,
            Expression::Tuple { source_info, .. } => source_info,
            Expression::Dict { source_info, .. } => source_info,
        }
    }
}

impl TypeExpression {
    /// Returns the source information of the type expression.
    fn source_info(&self) -> &SourceInfo {
        match self {
            TypeExpression::Named { source_info, .. } => source_info,
            TypeExpression::Generic { source_info, .. } => source_info,
            TypeExpression::Union { source_info, .. } => source_info,
            TypeExpression::Optional { source_info, .. } => source_info,
            TypeExpression::Callable { source_info, .. } => source_info,
        }
    }
}
