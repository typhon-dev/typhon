// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/tests.rs
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use inkwell::OptimizationLevel;

    use crate::backend::codegen::CodeGenerator;
    use crate::backend::llvm::LLVMContext;
    use crate::frontend::ast::*;
    use crate::frontend::lexer::token::{
        SourceInfo,
        SourceSpan,
    };

    // Helper function to create a source span
    fn create_source_span() -> SourceSpan {
        SourceSpan {
            start: (0, 0),
            end: (0, 0),
            source_id: 0,
        }
    }

    // Helper function to create source info
    fn create_source_info() -> SourceInfo {
        SourceInfo {
            span: create_source_span(),
        }
    }

    // Helper function to create an identifier
    fn create_identifier(name: &str) -> Identifier {
        Identifier {
            name: name.to_string(),
            source_info: create_source_info(),
        }
    }

    // Test basic literal code generation
    #[test]
    fn test_literal_codegen() {
        // Create a context
        let context = Rc::new(RefCell::new(LLVMContext::new("test")));

        // Create a code generator
        let mut codegen = CodeGenerator::new(context.clone());

        // Create a simple function that returns a literal
        let function = Function {
            name: create_identifier("test_literal"),
            params: Vec::new(),
            return_type: None,
            body: vec![Statement::Return {
                value: Some(Expression::Literal {
                    value: Literal::Int(42),
                    source_info: create_source_info(),
                }),
                source_info: create_source_info(),
            }],
            source_info: create_source_info(),
        };

        // Compile the function
        let result = codegen.visit_statement(&Statement::FunctionDecl(function));
        assert!(
            result.is_ok(),
            "Failed to compile function: {:?}",
            result.err()
        );

        // Verify the module
        let context = context.borrow();
        let verification_result = context.module().verify();
        assert!(
            verification_result.is_ok(),
            "Module verification failed: {:?}",
            verification_result.err()
        );

        // Print the generated IR for debugging
        let ir = context.module().print_to_string().to_string();
        println!("Generated IR:\n{}", ir);

        // Check that the IR contains our function
        assert!(
            ir.contains("define i64 @test_literal()"),
            "Generated IR does not contain our function"
        );

        // Check that the IR contains the return value
        assert!(
            ir.contains("ret i64 42"),
            "Generated IR does not contain the expected return value"
        );
    }

    // Test binary operations code generation
    #[test]
    fn test_binary_op_codegen() {
        // Create a context
        let context = Rc::new(RefCell::new(LLVMContext::new("test_binary_op")));

        // Create a code generator
        let mut codegen = CodeGenerator::new(context.clone());

        // Create a simple function that performs a binary operation
        let function = Function {
            name: create_identifier("test_add"),
            params: Vec::new(),
            return_type: None,
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(Expression::Literal {
                        value: Literal::Int(40),
                        source_info: create_source_info(),
                    }),
                    right: Box::new(Expression::Literal {
                        value: Literal::Int(2),
                        source_info: create_source_info(),
                    }),
                    source_info: create_source_info(),
                }),
                source_info: create_source_info(),
            }],
            source_info: create_source_info(),
        };

        // Compile the function
        let result = codegen.visit_statement(&Statement::FunctionDecl(function));
        assert!(
            result.is_ok(),
            "Failed to compile function: {:?}",
            result.err()
        );

        // Verify the module
        let context = context.borrow();
        let verification_result = context.module().verify();
        assert!(
            verification_result.is_ok(),
            "Module verification failed: {:?}",
            verification_result.err()
        );

        // Print the generated IR for debugging
        let ir = context.module().print_to_string().to_string();
        println!("Generated IR:\n{}", ir);

        // Check that the IR contains our function
        assert!(
            ir.contains("define i64 @test_add()"),
            "Generated IR does not contain our function"
        );

        // Check that the IR contains the add operation
        assert!(
            ir.contains("add i64"),
            "Generated IR does not contain the add operation"
        );
    }

    // Test variable declaration and assignment code generation
    #[test]
    fn test_variable_codegen() {
        // Create a context
        let context = Rc::new(RefCell::new(LLVMContext::new("test_variable")));

        // Create a code generator
        let mut codegen = CodeGenerator::new(context.clone());

        // Create a simple function that declares a variable and returns it
        let function = Function {
            name: create_identifier("test_variable"),
            params: Vec::new(),
            return_type: None,
            body: vec![
                Statement::VariableDecl {
                    name: create_identifier("x"),
                    type_annotation: None,
                    value: Some(Expression::Literal {
                        value: Literal::Int(42),
                        source_info: create_source_info(),
                    }),
                    mutable: true,
                    source_info: create_source_info(),
                },
                Statement::Return {
                    value: Some(Expression::Variable {
                        name: create_identifier("x"),
                        source_info: create_source_info(),
                    }),
                    source_info: create_source_info(),
                },
            ],
            source_info: create_source_info(),
        };

        // Compile the function
        let result = codegen.visit_statement(&Statement::FunctionDecl(function));
        assert!(
            result.is_ok(),
            "Failed to compile function: {:?}",
            result.err()
        );

        // Verify the module
        let context = context.borrow();
        let verification_result = context.module().verify();
        assert!(
            verification_result.is_ok(),
            "Module verification failed: {:?}",
            verification_result.err()
        );

        // Print the generated IR for debugging
        let ir = context.module().print_to_string().to_string();
        println!("Generated IR:\n{}", ir);

        // Check that the IR contains our function
        assert!(
            ir.contains("define i64 @test_variable()"),
            "Generated IR does not contain our function"
        );

        // Check that the IR contains the variable allocation
        assert!(
            ir.contains("alloca i64, i64 1, align 8, !\"x\""),
            "Generated IR does not contain the variable allocation"
        );

        // Check that the IR contains a load of the variable
        assert!(
            ir.contains("load i64, ptr %"),
            "Generated IR does not contain a load of the variable"
        );
    }

    // Test function with parameters code generation
    #[test]
    fn test_function_params_codegen() {
        // Create a context
        let context = Rc::new(RefCell::new(LLVMContext::new("test_function_params")));

        // Create a code generator
        let mut codegen = CodeGenerator::new(context.clone());

        // Create a simple function that takes parameters and returns their sum
        let function = Function {
            name: create_identifier("test_params"),
            params: vec![
                Parameter {
                    name: create_identifier("a"),
                    type_annotation: None,
                    default_value: None,
                    source_info: create_source_info(),
                },
                Parameter {
                    name: create_identifier("b"),
                    type_annotation: None,
                    default_value: None,
                    source_info: create_source_info(),
                },
            ],
            return_type: None,
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(Expression::Variable {
                        name: create_identifier("a"),
                        source_info: create_source_info(),
                    }),
                    right: Box::new(Expression::Variable {
                        name: create_identifier("b"),
                        source_info: create_source_info(),
                    }),
                    source_info: create_source_info(),
                }),
                source_info: create_source_info(),
            }],
            source_info: create_source_info(),
        };

        // Compile the function
        let result = codegen.visit_statement(&Statement::FunctionDecl(function));
        assert!(
            result.is_ok(),
            "Failed to compile function: {:?}",
            result.err()
        );

        // Verify the module
        let context = context.borrow();
        let verification_result = context.module().verify();
        assert!(
            verification_result.is_ok(),
            "Module verification failed: {:?}",
            verification_result.err()
        );

        // Print the generated IR for debugging
        let ir = context.module().print_to_string().to_string();
        println!("Generated IR:\n{}", ir);

        // Check that the IR contains our function with parameters
        assert!(
            ir.contains("define i64 @test_params(i64 %0, i64 %1)"),
            "Generated IR does not contain our function with parameters"
        );

        // Check that the IR contains the parameter allocations
        assert!(
            ir.contains("alloca i64, i64 1, align 8, !\"a\""),
            "Generated IR does not contain the allocation for parameter a"
        );
        assert!(
            ir.contains("alloca i64, i64 1, align 8, !\"b\""),
            "Generated IR does not contain the allocation for parameter b"
        );

        // Check that the IR contains the add operation
        assert!(
            ir.contains("add i64"),
            "Generated IR does not contain the add operation"
        );
    }
}
