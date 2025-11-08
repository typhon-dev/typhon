// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/typesystem/checker.rs
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
//! Type checker for the Typhon programming language.
//!
//! This module implements the type checker for Typhon, which performs static type
//! analysis of AST nodes. It uses a visitor pattern to traverse the AST and
//! check type correctness of each node.

use std::rc::Rc;

use crate::common::{
    SourceInfo,
    Span,
};
use crate::frontend::ast::{
    BinaryOperator,
    DefaultVisitor,
    Expression,
    Literal,
    Module,
    Parameter,
    Statement,
    TypeExpression,
    UnaryOperator,
    Visitor,
};
use crate::typesystem::error::{
    TypeError,
    TypeErrorKind,
    TypeErrorReport,
};
use crate::typesystem::types::{
    FunctionType,
    GenericInstance,
    ListType,
    ParameterType,
    PrimitiveTypeKind,
    TupleType,
    Type,
    TypeEnv,
    UnionType,
    check_type_compatibility,
};

/// Result of a type checking operation.
pub type TypeCheckResult = Result<Rc<Type>, TypeError>;

/// Type checker for the Typhon programming language.
pub struct TypeChecker {
    /// Current type environment.
    env: TypeEnv,
    /// Type error report.
    error_report: TypeErrorReport,
    /// Current function return type for checking return statements.
    current_function_return_type: Option<Rc<Type>>,
    /// Current function name for error reporting.
    current_function_name: Option<String>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    /// Creates a new type checker.
    pub fn new() -> Self {
        let mut env = TypeEnv::new();
        env.create_builtins();

        Self {
            env,
            error_report: TypeErrorReport::new(),
            current_function_return_type: None,
            current_function_name: None,
        }
    }

    /// Creates a new type checker with a given type environment.
    pub fn with_env(env: TypeEnv) -> Self {
        Self {
            env,
            error_report: TypeErrorReport::new(),
            current_function_return_type: None,
            current_function_name: None,
        }
    }

    /// Enters a new scope.
    pub fn enter_scope(&mut self) {
        let parent_env = Box::new(std::mem::take(&mut self.env));
        self.env = TypeEnv::new();
        self.env.parent = Some(Rc::new(*parent_env));
    }

    /// Exits the current scope.
    pub fn exit_scope(&mut self) {
        match &self.env.parent {
            Some(parent) => {
                let parent_clone = (**parent).clone();
                self.env = parent_clone;
            }
            None => panic!("Cannot exit top-level scope"),
        }
    }

    /// Checks a module.
    pub fn check_module(&mut self, module: &Module) -> TypeCheckResult {
        // First pass: register all top-level declarations
        for stmt in &module.statements {
            match stmt {
                Statement::FunctionDef {
                    name,
                    parameters,
                    return_type,
                    ..
                } => {
                    let function_type = self.function_type_from_def(parameters, return_type)?;
                    self.env.add_variable(name.name.clone(), function_type);
                }
                Statement::ClassDef { name, bases: _, .. } => {
                    // Create a placeholder class type to handle recursive references
                    let class_type = Rc::new(Type::class(
                        name.name.clone(),
                        Some(name.source_info),
                    ));
                    self.env.add_type_def(name.name.clone(), class_type.clone());
                    self.env.add_variable(name.name.clone(), class_type);
                }
                _ => {}
            }
        }

        // Second pass: check all statements
        for stmt in &module.statements {
            self.visit_statement(stmt)?;
        }

        Ok(Rc::new(Type::None))
    }

    /// Creates a function type from a function definition.
    fn function_type_from_def(
        &mut self,
        parameters: &[Parameter],
        return_type: &Option<TypeExpression>,
    ) -> TypeCheckResult {
        let param_types = parameters
            .iter()
            .map(|param| {
                let ty = match &param.type_annotation {
                    Some(ty_expr) => self.resolve_type_expression(ty_expr)?,
                    None => Rc::new(Type::Any),
                };

                let has_default = param.default_value.is_some();

                Ok(ParameterType::new(
                    Some(param.name.name.clone()),
                    ty,
                    has_default,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let ret_type = match return_type {
            Some(ty_expr) => self.resolve_type_expression(ty_expr)?,
            None => Rc::new(Type::Any),
        };

        // Create a FunctionType and wrap it in Type::Function
        Ok(Rc::new(Type::from(FunctionType::new(
            param_types,
            ret_type,
            None,
        ))))
    }

    /// Resolves a type expression to a concrete type.
    fn resolve_type_expression(&mut self, type_expr: &TypeExpression) -> TypeCheckResult {
        match type_expr {
            TypeExpression::Named { name, source_info } => {
                match self.env.get_type_def(&name.name) {
                    Some(ty) => Ok(ty),
                    None => Err(TypeError::new(
                        TypeErrorKind::UndefinedType {
                            name: name.name.clone(),
                        },
                        Some(*source_info),
                    )),
                }
            }
            TypeExpression::Generic {
                base,
                args,
                source_info,
            } => {
                let base_type = self.resolve_type_expression(base)?;

                let type_args = args
                    .iter()
                    .map(|arg| self.resolve_type_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Rc::new(Type::GenericInstance(Rc::new(
                    GenericInstance::with_source_info(base_type, type_args, *source_info),
                ))))
            }
            TypeExpression::Union { types, source_info } => {
                let union_types = types
                    .iter()
                    .map(|ty| self.resolve_type_expression(ty))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Rc::new(Type::Union(UnionType::with_source_info(
                    union_types,
                    *source_info,
                ))))
            }
            TypeExpression::Optional { inner, source_info } => {
                let inner_type = self.resolve_type_expression(inner)?;

                // Optional[T] is equivalent to Union[T, None]
                let none_type = Rc::new(Type::None);

                Ok(Rc::new(Type::Union(UnionType::with_source_info(
                    vec![inner_type, none_type],
                    *source_info,
                ))))
            }
            TypeExpression::Callable {
                parameter_types,
                return_type,
                source_info,
            } => {
                let param_types = parameter_types
                    .iter()
                    .map(|ty| self.resolve_type_expression(ty))
                    .collect::<Result<Vec<_>, _>>()?;

                let ret_type = self.resolve_type_expression(return_type)?;

                // Convert to parameter types with no names
                let params = param_types
                    .into_iter()
                    .map(|ty| ParameterType::new(None, ty, false))
                    .collect();

                // Create function type with proper source info
                let function_type = FunctionType::new(params, ret_type, Some(*source_info));

                Ok(Rc::new(Type::Function(Rc::new(function_type))))
            }
        }
    }

    /// Checks type compatibility and reports an error if incompatible.
    fn check_type_compatibility(
        &mut self,
        expected: &Type,
        actual: &Type,
        source_info: Option<Span>,
    ) -> TypeCheckResult {
        let compatibility = check_type_compatibility(actual, expected);
        if compatibility.is_compatible() {
            Ok(Rc::new(actual.clone()))
        } else {
            Err(TypeError::new(
                TypeErrorKind::TypeMismatch {
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                },
                source_info.map(SourceInfo::new),
            ))
        }
    }

    /// Type checks a binary operation.
    fn check_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
        source_info: Option<Span>,
    ) -> TypeCheckResult {
        let left_type = self.visit_expression(left)?;
        let right_type = self.visit_expression(right)?;

        // Type checking for binary operations
        match op {
            // Arithmetic operations
            BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::FloorDiv
            | BinaryOperator::Mod
            | BinaryOperator::Pow => {
                self.check_numeric_op(op, &left_type, &right_type, source_info)
            }

            // Bitwise operations
            BinaryOperator::BitAnd
            | BinaryOperator::BitOr
            | BinaryOperator::BitXor
            | BinaryOperator::LShift
            | BinaryOperator::RShift => self.check_int_op(op, &left_type, &right_type, source_info),

            // Matrix multiplication
            BinaryOperator::MatMul => {
                // For now, treat matrix multiplication as requiring Any type
                // In the future, this would check for specific matrix types
                Ok(Rc::new(Type::Any))
            }

            // Comparison operations
            BinaryOperator::Eq | BinaryOperator::NotEq => {
                // Most types can be compared for equality
                Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Bool)))
            }

            BinaryOperator::Lt | BinaryOperator::LtE | BinaryOperator::Gt | BinaryOperator::GtE => {
                // Comparable types include numbers and strings
                let is_comparable = matches!(
                    left_type.as_ref(),
                    Type::Primitive(p)
                        if matches!(
                            p.kind,
                            PrimitiveTypeKind::Int
                                | PrimitiveTypeKind::Float
                                | PrimitiveTypeKind::Str
                        )
                ) && matches!(
                    right_type.as_ref(),
                    Type::Primitive(p)
                        if matches!(
                            p.kind,
                            PrimitiveTypeKind::Int
                                | PrimitiveTypeKind::Float
                                | PrimitiveTypeKind::Str
                        )
                );

                if is_comparable
                    || matches!(left_type.as_ref(), Type::Any)
                    || matches!(right_type.as_ref(), Type::Any)
                {
                    Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Bool)))
                } else {
                    Err(TypeError::new(
                        TypeErrorKind::InvalidBinaryOperandTypes {
                            operation: format!("{op:?}"),
                            left: left_type.to_string(),
                            right: right_type.to_string(),
                        },
                        source_info.map(SourceInfo::new),
                    ))
                }
            }

            // Logical operations
            BinaryOperator::And | BinaryOperator::Or => {
                // Logical operations return a boolean
                Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Bool)))
            }
        }
    }

    /// Type checks a numeric binary operation.
    fn check_numeric_op(
        &mut self,
        op: &BinaryOperator,
        left_type: &Type,
        right_type: &Type,
        source_info: Option<Span>,
    ) -> TypeCheckResult {
        let is_numeric_left = matches!(
            left_type,
            Type::Primitive(p)
                if matches!(
                    p.kind,
                    PrimitiveTypeKind::Int | PrimitiveTypeKind::Float
                )
        );

        let is_numeric_right = matches!(
            right_type,
            Type::Primitive(p)
                if matches!(
                    p.kind,
                    PrimitiveTypeKind::Int | PrimitiveTypeKind::Float
                )
        );

        let is_str_left = matches!(
            left_type,
            Type::Primitive(p) if p.kind == PrimitiveTypeKind::Str
        );

        let is_str_right = matches!(
            right_type,
            Type::Primitive(p) if p.kind == PrimitiveTypeKind::Str
        );

        // Special case for string concatenation
        if *op == BinaryOperator::Add && (is_str_left || is_str_right) {
            if is_str_left && is_str_right {
                return Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Str)));
            } else {
                return Err(TypeError::new(
                    TypeErrorKind::InvalidBinaryOperandTypes {
                        operation: format!("{op:?}"),
                        left: left_type.to_string(),
                        right: right_type.to_string(),
                    },
                    source_info.map(SourceInfo::new),
                ));
            }
        }

        if is_numeric_left && is_numeric_right {
            // Determine the result type based on operand types
            let is_float_left = matches!(
                left_type,
                Type::Primitive(p) if p.kind == PrimitiveTypeKind::Float
            );

            let is_float_right = matches!(
                right_type,
                Type::Primitive(p) if p.kind == PrimitiveTypeKind::Float
            );

            if is_float_left || is_float_right {
                Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Float)))
            } else {
                Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Int)))
            }
        } else if matches!(left_type, Type::Any) || matches!(right_type, Type::Any) {
            // If either operand is Any, the result is Any
            Ok(Rc::new(Type::Any))
        } else {
            Err(TypeError::new(
                TypeErrorKind::InvalidBinaryOperandTypes {
                    operation: format!("{op:?}"),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                },
                source_info.map(SourceInfo::new),
            ))
        }
    }

    /// Type checks an integer binary operation.
    fn check_int_op(
        &mut self,
        op: &BinaryOperator,
        left_type: &Type,
        right_type: &Type,
        source_info: Option<Span>,
    ) -> TypeCheckResult {
        let is_int_left = matches!(
            left_type,
            Type::Primitive(p) if p.kind == PrimitiveTypeKind::Int
        );

        let is_int_right = matches!(
            right_type,
            Type::Primitive(p) if p.kind == PrimitiveTypeKind::Int
        );

        if is_int_left && is_int_right {
            Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Int)))
        } else if matches!(left_type, Type::Any) || matches!(right_type, Type::Any) {
            // If either operand is Any, the result is Any
            Ok(Rc::new(Type::Any))
        } else {
            Err(TypeError::new(
                TypeErrorKind::InvalidBinaryOperandTypes {
                    operation: format!("{op:?}"),
                    left: left_type.to_string(),
                    right: right_type.to_string(),
                },
                source_info.map(SourceInfo::new),
            ))
        }
    }

    /// Type checks a unary operation.
    fn check_unary_op(
        &mut self,
        op: &UnaryOperator,
        operand: &Expression,
        source_info: Option<Span>,
    ) -> TypeCheckResult {
        let operand_type = self.visit_expression(operand)?;

        match op {
            // Arithmetic unary operations
            UnaryOperator::Pos | UnaryOperator::Neg => {
                let is_numeric = matches!(
                    operand_type.as_ref(),
                    Type::Primitive(p)
                        if matches!(
                            p.kind,
                            PrimitiveTypeKind::Int | PrimitiveTypeKind::Float
                        )
                );

                if is_numeric || matches!(operand_type.as_ref(), Type::Any) {
                    Ok(operand_type)
                } else {
                    Err(TypeError::new(
                        TypeErrorKind::InvalidUnaryOperandType {
                            operation: format!("{op:?}"),
                            ty: operand_type.to_string(),
                        },
                        source_info.map(SourceInfo::new),
                    ))
                }
            }

            // Logical not
            UnaryOperator::Not => {
                // Not can be applied to any type, but returns a boolean
                Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Bool)))
            }

            // Bitwise not
            UnaryOperator::Invert => {
                let is_int = matches!(
                    operand_type.as_ref(),
                    Type::Primitive(p) if p.kind == PrimitiveTypeKind::Int
                );

                if is_int || matches!(operand_type.as_ref(), Type::Any) {
                    Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Int)))
                } else {
                    Err(TypeError::new(
                        TypeErrorKind::InvalidUnaryOperandType {
                            operation: format!("{op:?}"),
                            ty: operand_type.to_string(),
                        },
                        source_info.map(SourceInfo::new),
                    ))
                }
            }
        }
    }

    /// Infers the type of a literal.
    fn infer_literal_type(&self, literal: &Literal) -> Rc<Type> {
        match literal {
            Literal::Int(_) => Rc::new(Type::primitive(PrimitiveTypeKind::Int)),
            Literal::Float(_) => Rc::new(Type::primitive(PrimitiveTypeKind::Float)),
            Literal::String(_) => Rc::new(Type::primitive(PrimitiveTypeKind::Str)),
            Literal::Bytes(_) => Rc::new(Type::primitive(PrimitiveTypeKind::Bytes)),
            Literal::Bool(_) => Rc::new(Type::primitive(PrimitiveTypeKind::Bool)),
            Literal::None => Rc::new(Type::None),
            Literal::Ellipsis => Rc::new(Type::Any),
        }
    }

    /// Gets the error report.
    pub fn error_report(&self) -> &TypeErrorReport {
        &self.error_report
    }

    /// Returns whether there are type errors.
    pub fn has_errors(&self) -> bool {
        self.error_report.has_errors()
    }
}

impl<'a> Visitor<TypeCheckResult> for TypeChecker {
    fn visit_module(&mut self, module: &Module) -> TypeCheckResult {
        self.check_module(module)
    }

    fn visit_statement(&mut self, stmt: &Statement) -> TypeCheckResult {
        match stmt {
            Statement::Expression(expr) => self.visit_expression(expr),

            Statement::Assignment {
                target,
                value,
                source_info,
            } => {
                let target_type = self.visit_expression(target)?;
                let value_type = self.visit_expression(value)?;

                // Check if the target is assignable
                match target {
                    Expression::Variable { .. }
                    | Expression::Attribute { .. }
                    | Expression::Subscript { .. } => {
                        // These are valid assignment targets
                    }
                    _ => {
                        return Err(TypeError::new(
                            TypeErrorKind::InvalidAssignmentTarget {
                                ty: target_type.to_string(),
                            },
                            Some(*source_info),
                        ));
                    }
                }

                // Check type compatibility
                self.check_type_compatibility(&target_type, &value_type, Some(source_info.span))?;

                Ok(Rc::new(Type::None))
            }

            Statement::FunctionDef {
                name,
                parameters,
                return_type,
                body,
                source_info,
            } => {
                // Create function type
                let function_type = self.function_type_from_def(parameters, return_type)?;

                // First, get the return type from the function type before entering a new scope
                let return_type_clone = match function_type.as_ref() {
                    Type::Function(f) => f.return_type.clone(),
                    _ => Rc::new(Type::Any), // Fallback
                };

                // Store current function information before modifying
                let prev_return_type = self.current_function_return_type.clone();
                let prev_function_name = self.current_function_name.clone();

                // Set function context before entering scope
                self.current_function_return_type = Some(return_type_clone);
                self.current_function_name = Some(name.name.clone());

                // Enter a new scope for the function body
                self.enter_scope();

                // Add parameters to the scope - avoid mutable self borrowing issues
                for param in parameters.iter() {
                    let param_type = if let Some(ty_expr) = &param.type_annotation {
                        self.resolve_type_expression(ty_expr)?
                    } else {
                        Rc::new(Type::Any)
                    };

                    self.env.add_variable(param.name.name.clone(), param_type);
                }

                // Check the function body
                let mut return_seen = false;
                for stmt in body {
                    if let Statement::Return { .. } = stmt {
                        return_seen = true;
                    }

                    self.visit_statement(stmt)?;
                }

                // Check if a return statement is required but missing
                if !return_seen {
                    // Extract return type information to use in the error message
                    let (is_optional_return, return_type_str) = match function_type.as_ref() {
                        Type::Function(f) => {
                            let rt = &f.return_type;
                            let is_optional = matches!(rt.as_ref(), Type::None | Type::Any);
                            (is_optional, rt.to_string())
                        }
                        _ => (true, "<unknown>".to_string()),
                    };

                    if !is_optional_return {
                        return Err(TypeError::new(
                            TypeErrorKind::MissingReturn {
                                function: name.name.clone(),
                                expected: return_type_str,
                            },
                            Some(*source_info),
                        ));
                    }
                }

                // Restore function context
                self.current_function_return_type = prev_return_type;
                self.current_function_name = prev_function_name;

                // Exit function scope
                self.exit_scope();

                Ok(Rc::new(Type::None))
            }

            Statement::ClassDef {
                name: _,
                bases,
                body,
                source_info,
            } => {
                // Resolve base class types
                let base_types = bases
                    .iter()
                    .map(|base| {
                        let base_type = self.visit_expression(base)?;
                        // Validate that base type is a class or can be used as a base
                        match base_type.as_ref() {
                            Type::Class(_) => Ok(base_type),
                            Type::Any => Ok(base_type), // Allow Any as a base for flexibility
                            _ => Err(TypeError::new(
                                TypeErrorKind::Generic {
                                    message: format!(
                                        "Base class must be a class type, got {base_type}"
                                    ),
                                },
                                Some(*source_info),
                            )),
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Verify all base types and collect methods/attributes that should be inherited
                let mut inherited_methods = vec![];
                let mut inherited_fields = vec![];

                for base_type in base_types {
                    if let Type::Class(class_type) = base_type.as_ref() {
                        // Collect methods and fields for inheritance
                        for (method_name, method_type) in &class_type.methods {
                            inherited_methods.push((method_name.clone(), method_type.clone()));
                        }

                        for (field_name, field_type) in &class_type.fields {
                            inherited_fields.push((field_name.clone(), field_type.clone()));
                        }
                    }
                    // For Any type bases, we don't do inheritance since we don't know the structure
                }

                // Enter a new scope for the class body
                self.enter_scope();

                // Check the class body
                for stmt in body {
                    self.visit_statement(stmt)?;
                }

                // Exit class scope
                self.exit_scope();

                Ok(Rc::new(Type::None))
            }

            Statement::Return { value, source_info } => {
                // Check if return is in a function
                if self.current_function_return_type.is_none() {
                    return Err(TypeError::new(
                        TypeErrorKind::Generic {
                            message: "Return statement outside of function".to_string(),
                        },
                        Some(*source_info),
                    ));
                }

                // Get a clone of the return type to avoid borrow issues
                let return_type = self.current_function_return_type.clone().unwrap();

                // Now check the expression type against the return type
                match value {
                    Some(expr) => {
                        let expr_type = self.visit_expression(expr)?;
                        // Pass a reference to return_type that isn't tied to self
                        self.check_type_compatibility(
                            &return_type,
                            &expr_type,
                            Some(source_info.span),
                        )?;
                    }
                    None => {
                        // No value is equivalent to returning None
                        let none_type = Type::None;
                        self.check_type_compatibility(
                            &return_type,
                            &none_type,
                            Some(source_info.span),
                        )?;
                    }
                }

                Ok(Rc::new(Type::None))
            }

            Statement::Import {
                names,
                source_info: _,
            } => {
                // For now, assume imported modules and names are valid and have Any type
                for (name, as_name) in names {
                    let var_name = if let Some(alias) = as_name {
                        alias.name.clone()
                    } else {
                        name.name.clone()
                    };

                    self.env.add_variable(var_name, Rc::new(Type::Any));
                }

                Ok(Rc::new(Type::None))
            }

            Statement::FromImport {
                module: _,
                names,
                level: _,
                source_info: _,
            } => {
                // For now, assume imported names are valid and have Any type
                for (name, as_name) in names {
                    let var_name = if let Some(alias) = as_name {
                        alias.name.clone()
                    } else {
                        name.name.clone()
                    };

                    self.env.add_variable(var_name, Rc::new(Type::Any));
                }

                Ok(Rc::new(Type::None))
            }

            Statement::If {
                condition,
                body,
                else_body,
                source_info: _,
            } => {
                // Check condition
                let _cond_type = self.visit_expression(condition)?;

                // Enter a new scope for the if body
                self.enter_scope();

                // Check the if body
                for stmt in body {
                    self.visit_statement(stmt)?;
                }

                // Exit if body scope
                self.exit_scope();

                // Check else body if present
                if let Some(else_stmts) = else_body {
                    self.enter_scope();

                    for stmt in else_stmts {
                        self.visit_statement(stmt)?;
                    }

                    self.exit_scope();
                }

                Ok(Rc::new(Type::None))
            }

            Statement::While {
                condition,
                body,
                source_info: _,
            } => {
                // Check condition
                let _cond_type = self.visit_expression(condition)?;

                // Enter a new scope for the while body
                self.enter_scope();

                // Check the while body
                for stmt in body {
                    self.visit_statement(stmt)?;
                }

                // Exit while body scope
                self.exit_scope();

                Ok(Rc::new(Type::None))
            }

            Statement::For {
                target,
                iter,
                body,
                source_info,
            } => {
                // Check iterator expression
                let iter_type = self.visit_expression(iter)?;

                // Infer element type from iterator
                let elem_type = match iter_type.as_ref() {
                    Type::List(list_type) => list_type.element_type.clone(),
                    Type::Tuple(tuple_type) => {
                        if tuple_type.element_types.is_empty() {
                            Rc::new(Type::Any)
                        } else {
                            // Use the first element type as a simplification
                            tuple_type.element_types[0].clone()
                        }
                    }
                    // For other types, assume Any element type
                    _ => Rc::new(Type::Any),
                };

                // Enter a new scope for the for loop body
                self.enter_scope();

                // Add target variable to scope with inferred type
                match target {
                    Expression::Variable { name, .. } => {
                        self.env.add_variable(name.name.clone(), elem_type);
                    }
                    // For other target expressions, check they're assignable
                    _ => {
                        let target_type = self.visit_expression(target)?;

                        // Check if target is an assignable expression
                        match target {
                            Expression::Variable { .. }
                            | Expression::Attribute { .. }
                            | Expression::Subscript { .. } => {
                                // These are valid assignment targets, assign elem_type
                                // For now we don't actually enforce the assignment
                                // but we could modify to perform type checking here
                            }
                            _ => {
                                return Err(TypeError::new(
                                    TypeErrorKind::InvalidAssignmentTarget {
                                        ty: target_type.to_string(),
                                    },
                                    Some(*source_info),
                                ));
                            }
                        }
                    }
                }

                // Check the for loop body
                for stmt in body {
                    self.visit_statement(stmt)?;
                }

                // Exit for loop body scope
                self.exit_scope();

                Ok(Rc::new(Type::None))
            }

            Statement::Pass { .. } | Statement::Break { .. } | Statement::Continue { .. } => {
                // These statements don't have any type checking concerns
                Ok(Rc::new(Type::None))
            }

            Statement::VariableDecl {
                name,
                type_annotation,
                value,
                mutable: _,
                source_info,
            } => {
                // Resolve variable type
                let var_type = if let Some(ty_expr) = type_annotation {
                    self.resolve_type_expression(ty_expr)?
                } else if let Some(val) = value {
                    // Infer type from value if no annotation
                    self.visit_expression(val)?
                } else {
                    // Default to Any if no type annotation and no value
                    Rc::new(Type::Any)
                };

                // Check value against the type if provided
                if let Some(val) = value {
                    let val_type = self.visit_expression(val)?;
                    self.check_type_compatibility(&var_type, &val_type, Some(source_info.span))?;
                }

                // Add variable to environment
                self.env.add_variable(name.name.clone(), var_type);

                Ok(Rc::new(Type::None))
            }
        }
    }

    fn visit_expression(&mut self, expr: &Expression) -> TypeCheckResult {
        match expr {
            Expression::BinaryOp {
                left,
                op,
                right,
                source_info,
            } => self.check_binary_op(left, op, right, Some(source_info.span)),

            Expression::UnaryOp {
                op,
                operand,
                source_info,
            } => self.check_unary_op(op, operand, Some(source_info.span)),

            Expression::Literal {
                value,
                source_info: _,
            } => Ok(self.infer_literal_type(value)),

            Expression::Variable { name, source_info } => match self.env.get_variable(&name.name) {
                Some(ty) => Ok(ty),
                None => Err(TypeError::new(
                    TypeErrorKind::UndefinedVariable {
                        name: name.name.clone(),
                    },
                    Some(*source_info),
                )),
            },

            Expression::Attribute {
                value,
                attr,
                source_info,
            } => {
                let value_type = self.visit_expression(value)?;

                // Handle attribute access based on value type
                match value_type.as_ref() {
                    Type::Class(class_type) => {
                        // Check if attribute exists in class fields or methods
                        if let Some(field_type) = class_type.fields.get(&attr.name) {
                            Ok(field_type.clone())
                        } else if let Some(method_type) = class_type.methods.get(&attr.name) {
                            Ok(Rc::new(Type::Function(method_type.clone())))
                        } else {
                            // Check base classes
                            for base in &class_type.bases {
                                if let Type::Class(base_class) = base.as_ref() {
                                    if base_class.fields.contains_key(&attr.name)
                                        || base_class.methods.contains_key(&attr.name)
                                    {
                                        return Ok(Rc::new(Type::Any));
                                    }
                                }
                            }

                            // For module attributes or Any type, assume attribute exists
                            if matches!(value_type.as_ref(), Type::Any) {
                                Ok(Rc::new(Type::Any))
                            } else {
                                Err(TypeError::new(
                                    TypeErrorKind::UndefinedAttribute {
                                        base: value_type.to_string(),
                                        name: attr.name.clone(),
                                    },
                                    Some(*source_info),
                                ))
                            }
                        }
                    }
                    // For Any type, assume attribute exists with Any type
                    Type::Any => Ok(Rc::new(Type::Any)),
                    // For other types, check if the attribute is valid
                    _ => Err(TypeError::new(
                        TypeErrorKind::UndefinedAttribute {
                            base: value_type.to_string(),
                            name: attr.name.clone(),
                        },
                        Some(*source_info),
                    )),
                }
            }

            Expression::Subscript {
                value,
                index,
                source_info,
            } => {
                let value_type = self.visit_expression(value)?;
                let index_type = self.visit_expression(index)?;

                // Check if value type supports indexing
                match value_type.as_ref() {
                    Type::List(list_type) => {
                        // Check if index is an integer
                        if matches!(index_type.as_ref(), Type::Primitive(p) if p.kind == PrimitiveTypeKind::Int)
                            || matches!(index_type.as_ref(), Type::Any)
                        {
                            Ok(list_type.element_type.clone())
                        } else {
                            Err(TypeError::new(
                                TypeErrorKind::InvalidOperationType {
                                    operation: "indexing".to_string(),
                                    ty: index_type.to_string(),
                                },
                                Some(*source_info),
                            ))
                        }
                    }
                    Type::Tuple(tuple_type) => {
                        // Check if index is an integer
                        if matches!(index_type.as_ref(), Type::Primitive(p) if p.kind == PrimitiveTypeKind::Int)
                            || matches!(index_type.as_ref(), Type::Any)
                        {
                            // For constant integer literals, we could check the bounds
                            // For now, just return the first element type or Any
                            if tuple_type.element_types.is_empty() {
                                Ok(Rc::new(Type::Any))
                            } else {
                                Ok(tuple_type.element_types[0].clone())
                            }
                        } else {
                            Err(TypeError::new(
                                TypeErrorKind::InvalidOperationType {
                                    operation: "indexing".to_string(),
                                    ty: index_type.to_string(),
                                },
                                Some(*source_info),
                            ))
                        }
                    }
                    // String indexing returns a string
                    Type::Primitive(p) if p.kind == PrimitiveTypeKind::Str => {
                        if matches!(index_type.as_ref(), Type::Primitive(p) if p.kind == PrimitiveTypeKind::Int)
                            || matches!(index_type.as_ref(), Type::Any)
                        {
                            Ok(Rc::new(Type::primitive(PrimitiveTypeKind::Str)))
                        } else {
                            Err(TypeError::new(
                                TypeErrorKind::InvalidOperationType {
                                    operation: "indexing".to_string(),
                                    ty: index_type.to_string(),
                                },
                                Some(*source_info),
                            ))
                        }
                    }
                    // For Any type, assume indexing returns Any
                    Type::Any => Ok(Rc::new(Type::Any)),
                    // For other types, indexing is invalid
                    _ => Err(TypeError::new(
                        TypeErrorKind::InvalidOperationType {
                            operation: "indexing".to_string(),
                            ty: value_type.to_string(),
                        },
                        Some(*source_info),
                    )),
                }
            }

            Expression::Call {
                func,
                args,
                keywords: _,
                source_info,
            } => {
                let func_type = self.visit_expression(func)?;

                // Evaluate argument types
                let arg_types = args
                    .iter()
                    .map(|arg| self.visit_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                // Check if function is callable
                match func_type.as_ref() {
                    Type::Function(func_type) => {
                        // Check number of arguments
                        let required_params =
                            func_type.parameters.iter().filter(|p| !p.optional).count();

                        if arg_types.len() < required_params
                            || arg_types.len() > func_type.parameters.len()
                        {
                            return Err(TypeError::new(
                                TypeErrorKind::IncorrectArgumentCount {
                                    expected: func_type.parameters.len(),
                                    actual: arg_types.len(),
                                },
                                Some(*source_info),
                            ));
                        }

                        // Check argument types
                        for (arg_type, param) in arg_types.iter().zip(func_type.parameters.iter()) {
                            self.check_type_compatibility(
                                &param.ty,
                                arg_type,
                                Some(source_info.span),
                            )?;
                        }

                        // Return function's return type
                        Ok(func_type.return_type.clone())
                    }
                    // For Any type, assume callable with Any return type
                    Type::Any => Ok(Rc::new(Type::Any)),
                    // For other types, check if it has a __call__ method
                    _ => Err(TypeError::new(
                        TypeErrorKind::NotCallable {
                            ty: func_type.to_string(),
                        },
                        Some(*source_info),
                    )),
                }
            }

            Expression::Lambda {
                parameters,
                body,
                source_info,
            } => {
                // Enter a new scope for the lambda body
                self.enter_scope();

                // Add parameters to the scope
                for param in parameters {
                    let param_type = if let Some(ty_expr) = &param.type_annotation {
                        self.resolve_type_expression(ty_expr)?
                    } else {
                        Rc::new(Type::Any)
                    };

                    self.env.add_variable(param.name.name.clone(), param_type);
                }

                // Infer return type from body
                let return_type = self.visit_expression(body)?;

                // Create parameter types for function type
                let param_types = parameters
                    .iter()
                    .map(|param| {
                        let ty = if let Some(ty_expr) = &param.type_annotation {
                            self.resolve_type_expression(ty_expr)?
                        } else {
                            Rc::new(Type::Any)
                        };

                        let has_default = param.default_value.is_some();

                        Ok(ParameterType::new(
                            Some(param.name.name.clone()),
                            ty,
                            has_default,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Exit lambda scope
                self.exit_scope();

                // Create function type
                let function_type = Rc::new(FunctionType::new(
                    param_types,
                    return_type,
                    Some(*source_info),
                ));

                Ok(Rc::new(Type::Function(function_type)))
            }

            Expression::List {
                elements,
                source_info,
            } => {
                if elements.is_empty() {
                    // Empty list has Any element type
                    return Ok(Rc::new(Type::List(ListType::new(Rc::new(Type::Any)))));
                }

                // Infer element type from the first element
                let first_type = self.visit_expression(&elements[0])?;

                // Check that all elements have compatible types
                for element in &elements[1..] {
                    let elem_type = self.visit_expression(element)?;
                    self.check_type_compatibility(&first_type, &elem_type, Some(source_info.span))?;
                }

                Ok(Rc::new(Type::List(ListType::with_source_info(
                    first_type,
                    *source_info,
                ))))
            }

            Expression::Tuple {
                elements,
                source_info,
            } => {
                let element_types = elements
                    .iter()
                    .map(|elem| self.visit_expression(elem))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Rc::new(Type::Tuple(TupleType::with_source_info(
                    element_types,
                    *source_info,
                ))))
            }

            Expression::Dict { pairs, source_info } => {
                if pairs.is_empty() {
                    // Empty dict has Any key and value types
                    return Ok(Rc::new(Type::GenericInstance(Rc::new(
                        GenericInstance::with_source_info(
                            Rc::new(Type::class("dict".to_string(), None)),
                            vec![Rc::new(Type::Any), Rc::new(Type::Any)],
                            *source_info,
                        ),
                    ))));
                }

                // Infer key and value types from the first pair
                let (first_key, first_val) = &pairs[0];
                let key_type = self.visit_expression(first_key)?;
                let val_type = self.visit_expression(first_val)?;

                // Check that all keys and values have compatible types
                for (key, val) in &pairs[1..] {
                    let k_type = self.visit_expression(key)?;
                    let v_type = self.visit_expression(val)?;

                    self.check_type_compatibility(&key_type, &k_type, Some(source_info.span))?;
                    self.check_type_compatibility(&val_type, &v_type, Some(source_info.span))?;
                }

                // Create dict type with inferred key and value types
                Ok(Rc::new(Type::GenericInstance(Rc::new(
                    GenericInstance::with_source_info(
                        Rc::new(Type::class("dict".to_string(), None)),
                        vec![key_type, val_type],
                        *source_info,
                    ),
                ))))
            }
        }
    }

    fn visit_type_expression(&mut self, type_expr: &TypeExpression) -> TypeCheckResult {
        self.resolve_type_expression(type_expr)
    }

    fn visit_literal(&mut self, lit: &Literal) -> TypeCheckResult {
        Ok(self.infer_literal_type(lit))
    }
}

impl DefaultVisitor<TypeCheckResult> for TypeChecker {
    fn default_value(&self) -> TypeCheckResult {
        Ok(Rc::new(Type::None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be implemented in the tests.rs file
}
