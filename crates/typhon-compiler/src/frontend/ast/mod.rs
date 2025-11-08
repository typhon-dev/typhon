// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/frontend/ast/mod.rs
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
//! Abstract Syntax Tree (AST) definitions for the Typhon programming language.

use std::hash::Hash;

use rustc_hash::FxHashMap;

pub mod visitor;

pub use visitor::{
    DefaultVisitor,
    Visitor,
};

use crate::common::SourceInfo;

/// Identifier in the AST
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    /// Name of the identifier
    pub name: String,
    /// Source information
    pub source_info: SourceInfo,
}

impl Identifier {
    /// Creates a new identifier.
    pub fn new(name: String, source_info: SourceInfo) -> Self {
        Self { name, source_info }
    }
}

/// Module in the AST
#[derive(Debug, Clone)]
pub struct Module {
    /// Name of the module
    pub name: String,
    /// Statements in the module
    pub statements: Vec<Statement>,
    /// Source information
    pub source_info: SourceInfo,
}

impl Module {
    /// Creates a new module.
    pub fn new(name: String, statements: Vec<Statement>, source_info: SourceInfo) -> Self {
        Self {
            name,
            statements,
            source_info,
        }
    }
}

/// Statement in the AST
#[derive(Debug, Clone)]
pub enum Statement {
    /// Expression statement
    Expression(Expression),
    /// Assignment statement
    Assignment {
        /// Target of the assignment
        target: Expression,
        /// Value being assigned
        value: Expression,
        /// Source information
        source_info: SourceInfo,
    },
    /// Function definition
    FunctionDef {
        /// Name of the function
        name: Identifier,
        /// Parameters of the function
        parameters: Vec<Parameter>,
        /// Return type annotation (optional)
        return_type: Option<TypeExpression>,
        /// Body of the function
        body: Vec<Statement>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Class definition
    ClassDef {
        /// Name of the class
        name: Identifier,
        /// Base classes
        bases: Vec<Expression>,
        /// Body of the class
        body: Vec<Statement>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Return statement
    Return {
        /// Value being returned (optional)
        value: Option<Box<Expression>>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Import statement
    Import {
        /// Names being imported
        names: Vec<(Identifier, Option<Identifier>)>, // (name, asname)
        /// Source information
        source_info: SourceInfo,
    },
    /// From-import statement
    FromImport {
        /// Module being imported from
        module: Identifier,
        /// Names being imported
        names: Vec<(Identifier, Option<Identifier>)>, // (name, asname)
        /// Level of relative import
        level: usize,
        /// Source information
        source_info: SourceInfo,
    },
    /// If statement
    If {
        /// Condition
        condition: Expression,
        /// Body of the if statement
        body: Vec<Statement>,
        /// Else body (optional)
        else_body: Option<Vec<Statement>>,
        /// Source information
        source_info: SourceInfo,
    },
    /// While statement
    While {
        /// Condition
        condition: Expression,
        /// Body of the while statement
        body: Vec<Statement>,
        /// Source information
        source_info: SourceInfo,
    },
    /// For statement
    For {
        /// Target of the for loop
        target: Expression,
        /// Iterator expression
        iter: Expression,
        /// Body of the for loop
        body: Vec<Statement>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Pass statement
    Pass {
        /// Source information
        source_info: SourceInfo,
    },
    /// Break statement
    Break {
        /// Source information
        source_info: SourceInfo,
    },
    /// Continue statement
    Continue {
        /// Source information
        source_info: SourceInfo,
    },
    /// Variable declaration statement (Typhon-specific)
    VariableDecl {
        /// Name of the variable
        name: Identifier,
        /// Type annotation (optional)
        type_annotation: Option<TypeExpression>,
        /// Initial value (optional)
        value: Option<Box<Expression>>,
        /// Mutability
        mutable: bool,
        /// Source information
        source_info: SourceInfo,
    },
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Name of the parameter
    pub name: Identifier,
    /// Type annotation (optional)
    pub type_annotation: Option<TypeExpression>,
    /// Default value (optional)
    pub default_value: Option<Box<Expression>>,
    /// Source information
    pub source_info: SourceInfo,
}

impl Parameter {
    /// Creates a new parameter.
    pub fn new(
        name: Identifier,
        type_annotation: Option<TypeExpression>,
        default_value: Option<Box<Expression>>,
        source_info: SourceInfo,
    ) -> Self {
        Self {
            name,
            type_annotation,
            default_value,
            source_info,
        }
    }
}

/// Expression in the AST
#[derive(Debug, Clone)]
pub enum Expression {
    /// Binary operation
    BinaryOp {
        /// Left operand
        left: Box<Expression>,
        /// Operator
        op: BinaryOperator,
        /// Right operand
        right: Box<Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Unary operation
    UnaryOp {
        /// Operator
        op: UnaryOperator,
        /// Operand
        operand: Box<Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Literal
    Literal {
        /// Value of the literal
        value: Literal,
        /// Source information
        source_info: SourceInfo,
    },
    /// Variable reference
    Variable {
        /// Name of the variable
        name: Identifier,
        /// Source information
        source_info: SourceInfo,
    },
    /// Attribute access
    Attribute {
        /// Value being accessed
        value: Box<Expression>,
        /// Attribute name
        attr: Identifier,
        /// Source information
        source_info: SourceInfo,
    },
    /// Subscript
    Subscript {
        /// Value being subscripted
        value: Box<Expression>,
        /// Index expression
        index: Box<Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Call
    Call {
        /// Function being called
        func: Box<Expression>,
        /// Arguments
        args: Vec<Expression>,
        /// Keyword arguments
        keywords: FxHashMap<String, Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Lambda
    Lambda {
        /// Parameters
        parameters: Vec<Parameter>,
        /// Body of the lambda
        body: Box<Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// List
    List {
        /// Elements of the list
        elements: Vec<Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Tuple
    Tuple {
        /// Elements of the tuple
        elements: Vec<Expression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Dictionary
    Dict {
        /// Keys and values
        pairs: Vec<(Expression, Expression)>,
        /// Source information
        source_info: SourceInfo,
    },
}

/// Type expression in the AST
#[derive(Debug, Clone)]
pub enum TypeExpression {
    /// Named type
    Named {
        /// Name of the type
        name: Identifier,
        /// Source information
        source_info: SourceInfo,
    },
    /// Generic type
    Generic {
        /// Base type
        base: Box<TypeExpression>,
        /// Type arguments
        args: Vec<TypeExpression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Union type
    Union {
        /// Types in the union
        types: Vec<TypeExpression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Optional type
    Optional {
        /// Inner type
        inner: Box<TypeExpression>,
        /// Source information
        source_info: SourceInfo,
    },
    /// Callable type
    Callable {
        /// Parameter types
        parameter_types: Vec<TypeExpression>,
        /// Return type
        return_type: Box<TypeExpression>,
        /// Source information
        source_info: SourceInfo,
    },
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Floor division
    FloorDiv,
    /// Modulo
    Mod,
    /// Power
    Pow,
    /// Left shift
    LShift,
    /// Right shift
    RShift,
    /// Bitwise AND
    BitAnd,
    /// Bitwise OR
    BitOr,
    /// Bitwise XOR
    BitXor,
    /// Matrix multiplication
    MatMul,
    /// Equality
    Eq,
    /// Inequality
    NotEq,
    /// Less than
    Lt,
    /// Less than or equal
    LtE,
    /// Greater than
    Gt,
    /// Greater than or equal
    GtE,
    /// Logical AND
    And,
    /// Logical OR
    Or,
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    /// Positive
    Pos,
    /// Negative
    Neg,
    /// Bitwise NOT
    Not,
    /// Logical NOT
    Invert,
}

/// Literal value
#[derive(Debug, Clone)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Bytes literal
    Bytes(Vec<u8>),
    /// Boolean literal
    Bool(bool),
    /// None literal
    None,
    /// Ellipsis literal
    Ellipsis,
}
