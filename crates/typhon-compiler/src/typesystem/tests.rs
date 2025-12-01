//! Tests for the Typhon type system.
//!
//! This module contains tests for the type checker and type inference.

use std::rc::Rc;

use typhon_parser::ast::{
    BinaryOperator,
    Expression,
    Identifier,
    Literal,
    Module,
    Parameter,
    SourceInfo,
    Statement,
    TypeExpression,
    UnaryOperator,
};
use typhon_parser::common::Span;
use typhon_parser::token::{Token, TokenKind};

use crate::typesystem::checker::TypeChecker;
use crate::typesystem::error::TypeErrorKind;
use crate::typesystem::types::{PrimitiveTypeKind, Type};

/// Creates a token span for testing.
fn span(start: usize, end: usize) -> Span {
    Span::new(start, end)
}

/// Creates a source info for testing.
fn source_info(start: usize, end: usize) -> SourceInfo {
    SourceInfo::new(span(start, end))
}

/// Creates an identifier for testing.
fn ident(name: &str, start: usize, end: usize) -> Identifier {
    Identifier::new(name.to_string(), source_info(start, end))
}

/// Creates a literal expression for testing.
fn lit_expr(value: Literal, start: usize, end: usize) -> Expression {
    Expression::Literal { value, source_info: source_info(start, end) }
}

/// Creates an integer literal expression for testing.
fn int_lit(value: i64, start: usize, end: usize) -> Expression {
    lit_expr(Literal::Int(value), start, end)
}

/// Creates a float literal expression for testing.
fn float_lit(value: f64, start: usize, end: usize) -> Expression {
    lit_expr(Literal::Float(value), start, end)
}

/// Creates a string literal expression for testing.
fn str_lit(value: &str, start: usize, end: usize) -> Expression {
    lit_expr(Literal::String(value.to_string()), start, end)
}

/// Creates a boolean literal expression for testing.
fn bool_lit(value: bool, start: usize, end: usize) -> Expression {
    lit_expr(Literal::Bool(value), start, end)
}

/// Creates a none literal expression for testing.
fn none_lit(start: usize, end: usize) -> Expression {
    lit_expr(Literal::None, start, end)
}

/// Creates a variable reference expression for testing.
fn var_expr(name: &str, start: usize, end: usize) -> Expression {
    Expression::Variable { name: ident(name, start, end), source_info: source_info(start, end) }
}

/// Creates a binary operation expression for testing.
fn binary_expr(
    left: Expression,
    op: BinaryOperator,
    right: Expression,
    start: usize,
    end: usize,
) -> Expression {
    Expression::BinaryOp {
        left: Box::new(left),
        op,
        right: Box::new(right),
        source_info: source_info(start, end),
    }
}

/// Creates an attribute access expression for testing.
fn attr_expr(value: Expression, attr: &str, start: usize, end: usize) -> Expression {
    Expression::Attribute {
        value: Box::new(value),
        attr: ident(attr, start, end),
        source_info: source_info(start, end),
    }
}

/// Creates a function call expression for testing.
fn call_expr(func: Expression, args: Vec<Expression>, start: usize, end: usize) -> Expression {
    Expression::Call {
        func: Box::new(func),
        args,
        keywords: Default::default(),
        source_info: source_info(start, end),
    }
}

/// Creates a list expression for testing.
fn list_expr(elements: Vec<Expression>, start: usize, end: usize) -> Expression {
    Expression::List { elements, source_info: source_info(start, end) }
}

/// Creates a type expression for testing.
fn type_expr(name: &str, start: usize, end: usize) -> TypeExpression {
    TypeExpression::Named { name: ident(name, start, end), source_info: source_info(start, end) }
}

/// Creates a variable declaration statement for testing.
fn var_decl(
    name: &str,
    type_name: Option<&str>,
    value: Option<Expression>,
    mutable: bool,
    start: usize,
    end: usize,
) -> Statement {
    Statement::VariableDecl {
        name: ident(name, start, end),
        type_annotation: type_name.map(|t| type_expr(t, start, end)),
        value,
        mutable,
        source_info: source_info(start, end),
    }
}

/// Creates a function definition statement for testing.
fn func_def(
    name: &str,
    parameters: Vec<Parameter>,
    return_type: Option<&str>,
    body: Vec<Statement>,
    start: usize,
    end: usize,
) -> Statement {
    Statement::FunctionDef {
        name: ident(name, start, end),
        parameters,
        return_type: return_type.map(|t| type_expr(t, start, end)),
        body,
        source_info: source_info(start, end),
    }
}

/// Creates a parameter for testing.
fn param(
    name: &str,
    type_name: Option<&str>,
    default_value: Option<Expression>,
    start: usize,
    end: usize,
) -> Parameter {
    Parameter::new(
        ident(name, start, end),
        type_name.map(|t| type_expr(t, start, end)),
        default_value,
        source_info(start, end),
    )
}

/// Creates a return statement for testing.
fn return_stmt(value: Option<Expression>, start: usize, end: usize) -> Statement {
    Statement::Return { value, source_info: source_info(start, end) }
}

/// Creates an expression statement for testing.
fn expr_stmt(expr: Expression) -> Statement {
    Statement::Expression(expr)
}

/// Creates an assignment statement for testing.
fn assign_stmt(target: Expression, value: Expression, start: usize, end: usize) -> Statement {
    Statement::Assignment { target, value, source_info: source_info(start, end) }
}

/// Creates an if statement for testing.
fn if_stmt(
    condition: Expression,
    body: Vec<Statement>,
    else_body: Option<Vec<Statement>>,
    start: usize,
    end: usize,
) -> Statement {
    Statement::If { condition, body, else_body, source_info: source_info(start, end) }
}

/// Tests type checking of a simple variable declaration.
#[test]
fn test_var_decl() {
    let stmt = var_decl("x", Some("int"), Some(int_lit(42, 4, 6)), false, 0, 6);
    let module = Module::new("test".to_string(), vec![stmt], source_info(0, 6));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}

/// Tests type checking of a variable declaration with mismatched types.
#[test]
fn test_var_decl_type_mismatch() {
    let stmt = var_decl("x", Some("int"), Some(str_lit("hello", 4, 11)), false, 0, 11);
    let module = Module::new("test".to_string(), vec![stmt], source_info(0, 11));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_err());
    assert!(checker.has_errors());

    if let Err(error) = result {
        assert!(matches!(error.kind, TypeErrorKind::TypeMismatch { .. }));
    }
}

/// Tests type inference in a variable declaration without type annotation.
#[test]
fn test_var_decl_inference() {
    let decl_stmt = var_decl("x", None, Some(int_lit(42, 4, 6)), false, 0, 6);
    let ref_stmt = expr_stmt(var_expr("x", 7, 8));
    let module = Module::new("test".to_string(), vec![decl_stmt, ref_stmt], source_info(0, 8));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}

/// Tests type checking of a simple function definition.
#[test]
fn test_func_def() {
    let func_body = vec![return_stmt(Some(int_lit(42, 34, 36)), 30, 37)];
    let func = func_def(
        "add",
        vec![param("a", Some("int"), None, 10, 16), param("b", Some("int"), None, 18, 24)],
        Some("int"),
        func_body,
        0,
        37,
    );

    let module = Module::new("test".to_string(), vec![func], source_info(0, 37));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}

/// Tests type checking of a function with invalid return type.
#[test]
fn test_func_invalid_return() {
    let func_body = vec![return_stmt(Some(str_lit("hello", 34, 41)), 30, 42)];
    let func = func_def(
        "add",
        vec![param("a", Some("int"), None, 10, 16), param("b", Some("int"), None, 18, 24)],
        Some("int"),
        func_body,
        0,
        42,
    );

    let module = Module::new("test".to_string(), vec![func], source_info(0, 42));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_err());
    assert!(checker.has_errors());

    if let Err(error) = result {
        assert!(matches!(error.kind, TypeErrorKind::TypeMismatch { .. }));
    }
}

/// Tests type checking of a function call.
#[test]
fn test_func_call() {
    let func = func_def(
        "add",
        vec![param("a", Some("int"), None, 10, 16), param("b", Some("int"), None, 18, 24)],
        Some("int"),
        vec![return_stmt(Some(int_lit(42, 34, 36)), 30, 37)],
        0,
        37,
    );

    let call = expr_stmt(call_expr(
        var_expr("add", 38, 41),
        vec![int_lit(1, 42, 43), int_lit(2, 45, 46)],
        38,
        47,
    ));

    let module = Module::new("test".to_string(), vec![func, call], source_info(0, 47));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}

/// Tests type checking of a function call with incorrect argument types.
#[test]
fn test_func_call_invalid_args() {
    let func = func_def(
        "add",
        vec![param("a", Some("int"), None, 10, 16), param("b", Some("int"), None, 18, 24)],
        Some("int"),
        vec![return_stmt(Some(int_lit(42, 34, 36)), 30, 37)],
        0,
        37,
    );

    let call = expr_stmt(call_expr(
        var_expr("add", 38, 41),
        vec![int_lit(1, 42, 43), str_lit("hello", 45, 52)],
        38,
        53,
    ));

    let module = Module::new("test".to_string(), vec![func, call], source_info(0, 53));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_err());
    assert!(checker.has_errors());
}

/// Tests type checking of binary operations.
#[test]
fn test_binary_ops() {
    let decl1 = var_decl("a", Some("int"), Some(int_lit(1, 8, 9)), false, 0, 9);
    let decl2 = var_decl("b", Some("int"), Some(int_lit(2, 18, 19)), false, 10, 19);

    let add = expr_stmt(binary_expr(
        var_expr("a", 20, 21),
        BinaryOperator::Add,
        var_expr("b", 24, 25),
        20,
        25,
    ));

    let module = Module::new("test".to_string(), vec![decl1, decl2, add], source_info(0, 25));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}

/// Tests type checking of binary operations with incompatible types.
#[test]
fn test_binary_ops_invalid() {
    let decl1 = var_decl("a", Some("int"), Some(int_lit(1, 8, 9)), false, 0, 9);
    let decl2 = var_decl("b", Some("str"), Some(str_lit("hello", 18, 25)), false, 10, 25);

    let add = expr_stmt(binary_expr(
        var_expr("a", 26, 27),
        BinaryOperator::Add,
        var_expr("b", 30, 31),
        26,
        31,
    ));

    let module = Module::new("test".to_string(), vec![decl1, decl2, add], source_info(0, 31));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_err());
    assert!(checker.has_errors());
}

/// Tests type checking of a list.
#[test]
fn test_list() {
    let list = var_decl(
        "nums",
        None,
        Some(list_expr(vec![int_lit(1, 12, 13), int_lit(2, 15, 16), int_lit(3, 18, 19)], 11, 20)),
        false,
        0,
        20,
    );

    let module = Module::new("test".to_string(), vec![list], source_info(0, 20));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}

/// Tests type checking of a list with mixed types (should fail).
#[test]
fn test_list_mixed_types() {
    let list = var_decl(
        "mixed",
        None,
        Some(list_expr(vec![int_lit(1, 13, 14), str_lit("hello", 16, 23)], 12, 24)),
        false,
        0,
        24,
    );

    let module = Module::new("test".to_string(), vec![list], source_info(0, 24));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_err());
    assert!(checker.has_errors());
}

/// Tests inference of list element type when using subscript.
#[test]
fn test_list_subscript() {
    let list_decl = var_decl(
        "nums",
        None,
        Some(list_expr(vec![int_lit(1, 12, 13), int_lit(2, 15, 16), int_lit(3, 18, 19)], 11, 20)),
        false,
        0,
        20,
    );

    let subscript = Expression::Subscript {
        value: Box::new(var_expr("nums", 21, 25)),
        index: Box::new(int_lit(0, 26, 27)),
        source_info: source_info(21, 28),
    };

    let use_elem = var_decl("first", Some("int"), Some(subscript), false, 29, 48);

    let module = Module::new("test".to_string(), vec![list_decl, use_elem], source_info(0, 48));

    let mut checker = TypeChecker::new();
    let result = checker.check_module(&module);

    assert!(result.is_ok());
    assert!(!checker.has_errors());
}
