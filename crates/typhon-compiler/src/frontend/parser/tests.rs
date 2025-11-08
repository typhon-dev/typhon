// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/frontend/parser/tests.rs
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

#[cfg(test)]
mod tests {
    use crate::frontend::ast::{
        Expression,
        Identifier,
        Literal,
        Module,
        Parameter,
        SourceInfo,
        Statement,
        TypeExpression,
    };
    use crate::frontend::parser::Parser;

    fn parse_expression(source: &str) -> Expression {
        let mut parser = Parser::new(source);
        parser.parse_expression().unwrap()
    }

    fn parse_statement(source: &str) -> Statement {
        let mut parser = Parser::new(source);
        parser.parse_statement().unwrap()
    }

    fn parse_module(source: &str) -> Module {
        let mut parser = Parser::new(source);
        parser.parse().unwrap()
    }

    #[test]
    fn test_variable_expression() {
        let expr = parse_expression("foo");
        match expr {
            Expression::Variable { name, .. } => {
                assert_eq!(name.name, "foo");
            }
            _ => panic!("Expected variable expression"),
        }
    }

    #[test]
    fn test_literal_expressions() {
        // Integer literal
        let expr = parse_expression("42");
        match expr {
            Expression::Literal {
                value: Literal::Int(value),
                ..
            } => {
                assert_eq!(value, 42);
            }
            _ => panic!("Expected integer literal"),
        }

        // Float literal
        let expr = parse_expression("3.14");
        match expr {
            Expression::Literal {
                value: Literal::Float(value),
                ..
            } => {
                assert!((value - 3.14).abs() < f64::EPSILON);
            }
            _ => panic!("Expected float literal"),
        }

        // String literal
        let expr = parse_expression("\"hello\"");
        match expr {
            Expression::Literal {
                value: Literal::String(value),
                ..
            } => {
                assert_eq!(value, "hello");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_binary_expressions() {
        // Addition
        let expr = parse_expression("a + b");
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                match *left {
                    Expression::Variable { name, .. } => assert_eq!(name.name, "a"),
                    _ => panic!("Expected variable expression for left operand"),
                }
                match *right {
                    Expression::Variable { name, .. } => assert_eq!(name.name, "b"),
                    _ => panic!("Expected variable expression for right operand"),
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_function_definition() {
        let stmt = parse_statement("def test(a: int, b = 5) -> str:\n    return a + b\n");

        match stmt {
            Statement::FunctionDef {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                assert_eq!(name.name, "test");

                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].name.name, "a");
                assert_eq!(parameters[1].name.name, "b");

                assert!(return_type.is_some());

                assert_eq!(body.len(), 1);
                match &body[0] {
                    Statement::Return { .. } => {}
                    _ => panic!("Expected return statement in function body"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_variable_declaration() {
        // Let declaration with type
        let stmt = parse_statement("let x: int = 42");
        match stmt {
            Statement::VariableDecl {
                name,
                type_annotation,
                value,
                mutable,
                ..
            } => {
                assert_eq!(name.name, "x");
                assert!(type_annotation.is_some());
                assert!(value.is_some());
                assert_eq!(mutable, true);
            }
            _ => panic!("Expected variable declaration"),
        }

        // Mut declaration without type
        let stmt = parse_statement("mut y = 3.14");
        match stmt {
            Statement::VariableDecl {
                name,
                type_annotation,
                value,
                mutable,
                ..
            } => {
                assert_eq!(name.name, "y");
                assert!(type_annotation.is_none());
                assert!(value.is_some());
                assert_eq!(mutable, false);
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_if_statement() {
        let stmt = parse_statement("if x > 0:\n    y = 1\nelse:\n    y = -1\n");
        match stmt {
            Statement::If {
                condition,
                body,
                else_body,
                ..
            } => {
                match condition {
                    Expression::BinaryOp { .. } => {}
                    _ => panic!("Expected binary operation in condition"),
                }

                assert_eq!(body.len(), 1);
                assert!(else_body.is_some());
                assert_eq!(else_body.unwrap().len(), 1);
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_while_statement() {
        let stmt = parse_statement("while i < 10:\n    i += 1\n");
        match stmt {
            Statement::While {
                condition, body, ..
            } => {
                match condition {
                    Expression::BinaryOp { .. } => {}
                    _ => panic!("Expected binary operation in condition"),
                }

                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected while statement"),
        }
    }

    #[test]
    fn test_for_statement() {
        let stmt = parse_statement("for i in range(10):\n    print(i)\n");
        match stmt {
            Statement::For {
                target, iter, body, ..
            } => {
                match target {
                    Expression::Variable { name, .. } => assert_eq!(name.name, "i"),
                    _ => panic!("Expected variable expression for target"),
                }

                match iter {
                    Expression::Call { .. } => {}
                    _ => panic!("Expected call expression for iterator"),
                }

                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected for statement"),
        }
    }

    #[test]
    fn test_import_statements() {
        // Simple import
        let stmt = parse_statement("import math");
        match stmt {
            Statement::Import { names, .. } => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].0.name, "math");
            }
            _ => panic!("Expected import statement"),
        }

        // Import with alias
        let stmt = parse_statement("import math as m");
        match stmt {
            Statement::Import { names, .. } => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].0.name, "math");
                assert!(names[0].1.is_some());
                assert_eq!(names[0].1.as_ref().unwrap().name, "m");
            }
            _ => panic!("Expected import statement"),
        }

        // From import
        let stmt = parse_statement("from math import sin, cos");
        match stmt {
            Statement::FromImport {
                module,
                names,
                level,
                ..
            } => {
                assert_eq!(module.name, "math");
                assert_eq!(names.len(), 2);
                assert_eq!(names[0].0.name, "sin");
                assert_eq!(names[1].0.name, "cos");
                assert_eq!(level, 0);
            }
            _ => panic!("Expected from-import statement"),
        }
    }

    #[test]
    fn test_simple_module() {
        let source = r#"
def greet(name: str) -> str:
    return "Hello, " + name

let message = greet("World")
print(message)
"#;

        let module = parse_module(source);
        assert_eq!(module.statements.len(), 3);

        match &module.statements[0] {
            Statement::FunctionDef { name, .. } => assert_eq!(name.name, "greet"),
            _ => panic!("Expected function definition"),
        }

        match &module.statements[1] {
            Statement::VariableDecl { name, .. } => assert_eq!(name.name, "message"),
            _ => panic!("Expected variable declaration"),
        }

        match &module.statements[2] {
            Statement::Expression(Expression::Call { .. }) => {}
            _ => panic!("Expected call expression"),
        }
    }
}
