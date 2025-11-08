// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/frontend/ast/visitor.rs
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
use super::*;

/// Visitor trait for AST nodes
pub trait Visitor<T> {
    /// Visits a module
    fn visit_module(&mut self, module: &Module) -> T;

    /// Visits a statement
    fn visit_statement(&mut self, stmt: &Statement) -> T;

    /// Visits an expression
    fn visit_expression(&mut self, expr: &Expression) -> T;

    /// Visits a type expression
    fn visit_type_expression(&mut self, type_expr: &TypeExpression) -> T;

    /// Visits a literal
    fn visit_literal(&mut self, lit: &Literal) -> T;
}

/// Default implementation of the Visitor trait
pub trait DefaultVisitor<T>: Visitor<T> {
    /// Default value for a visitor
    fn default_value(&self) -> T;

    /// Visits a module with default implementation
    fn default_visit_module(&mut self, module: &Module) -> T {
        for stmt in &module.statements {
            self.visit_statement(stmt);
        }
        self.default_value()
    }

    /// Visits a statement with default implementation
    fn default_visit_statement(&mut self, stmt: &Statement) -> T {
        match stmt {
            Statement::Expression(expr) => {
                self.visit_expression(expr);
            }
            Statement::Assignment { target, value, .. } => {
                self.visit_expression(target);
                self.visit_expression(value);
            }
            Statement::FunctionDef {
                name: _,
                parameters,
                return_type,
                body,
                ..
            } => {
                for param in parameters {
                    if let Some(type_annotation) = &param.type_annotation {
                        self.visit_type_expression(type_annotation);
                    }
                    if let Some(default_value) = &param.default_value {
                        self.visit_expression(default_value);
                    }
                }
                if let Some(ret_type) = return_type {
                    self.visit_type_expression(ret_type);
                }
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::ClassDef { bases, body, .. } => {
                for base in bases {
                    self.visit_expression(base);
                }
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::Return { value, .. } => {
                if let Some(val) = value {
                    self.visit_expression(val);
                }
            }
            Statement::Import { .. } | Statement::FromImport { .. } => {
                // No expressions to visit
            }
            Statement::If {
                condition,
                body,
                else_body,
                ..
            } => {
                self.visit_expression(condition);
                for stmt in body {
                    self.visit_statement(stmt);
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.visit_statement(stmt);
                    }
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                self.visit_expression(condition);
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::For {
                target, iter, body, ..
            } => {
                self.visit_expression(target);
                self.visit_expression(iter);
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::Pass { .. } | Statement::Break { .. } | Statement::Continue { .. } => {
                // No expressions to visit
            }
            Statement::VariableDecl {
                type_annotation,
                value,
                ..
            } => {
                if let Some(type_annot) = type_annotation {
                    self.visit_type_expression(type_annot);
                }
                if let Some(val) = value {
                    self.visit_expression(val);
                }
            }
        }
        self.default_value()
    }

    /// Visits an expression with default implementation
    fn default_visit_expression(&mut self, expr: &Expression) -> T {
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.visit_expression(operand);
            }
            Expression::Literal { value, .. } => {
                self.visit_literal(value);
            }
            Expression::Variable { .. } => {
                // No expressions to visit
            }
            Expression::Attribute { value, .. } => {
                self.visit_expression(value);
            }
            Expression::Subscript { value, index, .. } => {
                self.visit_expression(value);
                self.visit_expression(index);
            }
            Expression::Call {
                func,
                args,
                keywords,
                ..
            } => {
                self.visit_expression(func);
                for arg in args {
                    self.visit_expression(arg);
                }
                for value in keywords.values() {
                    self.visit_expression(value);
                }
            }
            Expression::Lambda {
                parameters, body, ..
            } => {
                for param in parameters {
                    if let Some(type_annotation) = &param.type_annotation {
                        self.visit_type_expression(type_annotation);
                    }
                    if let Some(default_value) = &param.default_value {
                        self.visit_expression(default_value);
                    }
                }
                self.visit_expression(body);
            }
            Expression::List { elements, .. } | Expression::Tuple { elements, .. } => {
                for element in elements {
                    self.visit_expression(element);
                }
            }
            Expression::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.visit_expression(key);
                    self.visit_expression(value);
                }
            }
        }
        self.default_value()
    }

    /// Visits a type expression with default implementation
    fn default_visit_type_expression(&mut self, type_expr: &TypeExpression) -> T {
        match type_expr {
            TypeExpression::Named { .. } => {
                // No type expressions to visit
            }
            TypeExpression::Generic { base, args, .. } => {
                self.visit_type_expression(base);
                for arg in args {
                    self.visit_type_expression(arg);
                }
            }
            TypeExpression::Union { types, .. } => {
                for ty in types {
                    self.visit_type_expression(ty);
                }
            }
            TypeExpression::Optional { inner, .. } => {
                self.visit_type_expression(inner);
            }
            TypeExpression::Callable {
                parameter_types,
                return_type,
                ..
            } => {
                for param_type in parameter_types {
                    self.visit_type_expression(param_type);
                }
                self.visit_type_expression(return_type);
            }
        }
        self.default_value()
    }

    /// Visits a literal with default implementation
    fn default_visit_literal(&mut self, _lit: &Literal) -> T {
        self.default_value()
    }
}
