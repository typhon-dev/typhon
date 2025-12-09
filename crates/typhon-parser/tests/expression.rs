//! Tests for expression parsing.

use std::sync::Arc;

use typhon_ast::nodes::{AnyNode, NodeKind};
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

fn create_parser(source: &'_ str) -> Parser<'_> {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());

    Parser::new(source, file_id, Arc::new(source_manager))
}

// ============================================================================
// Literal Expression Tests
// ============================================================================

#[test]
fn test_integer_literal() {
    let mut parser = create_parser("42");
    let expr_id = parser.parse_expression().expect("Failed to parse integer");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LiteralExpr(_)));
}

#[test]
fn test_float_literal() {
    let mut parser = create_parser("3.14");
    let expr_id = parser.parse_expression().expect("Failed to parse float");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LiteralExpr(_)));
}

#[test]
fn test_string_literal() {
    let mut parser = create_parser("\"hello world\"");
    let expr_id = parser.parse_expression().expect("Failed to parse string");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LiteralExpr(_)));
}

#[test]
fn test_true_literal() {
    let mut parser = create_parser("True");
    let expr_id = parser.parse_expression().expect("Failed to parse True");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LiteralExpr(_)));
}

#[test]
fn test_false_literal() {
    let mut parser = create_parser("False");
    let expr_id = parser.parse_expression().expect("Failed to parse False");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LiteralExpr(_)));
}

#[test]
fn test_none_literal() {
    let mut parser = create_parser("None");
    let expr_id = parser.parse_expression().expect("Failed to parse None");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LiteralExpr(_)));
}

// ============================================================================
// Binary Operation Tests
// ============================================================================

#[test]
fn test_addition() {
    let mut parser = create_parser("1 + 2");
    let expr_id = parser.parse_expression().expect("Failed to parse addition");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_subtraction() {
    let mut parser = create_parser("5 - 3");
    let expr_id = parser.parse_expression().expect("Failed to parse subtraction");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_multiplication() {
    let mut parser = create_parser("4 * 6");
    let expr_id = parser.parse_expression().expect("Failed to parse multiplication");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_division() {
    let mut parser = create_parser("10 / 2");
    let expr_id = parser.parse_expression().expect("Failed to parse division");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_floor_division() {
    let mut parser = create_parser("10 // 3");
    let expr_id = parser.parse_expression().expect("Failed to parse floor division");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_modulo() {
    let mut parser = create_parser("10 % 3");
    let expr_id = parser.parse_expression().expect("Failed to parse modulo");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_power() {
    let mut parser = create_parser("2 ** 8");
    let expr_id = parser.parse_expression().expect("Failed to parse power");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

// ============================================================================
// Comparison Operation Tests
// ============================================================================

#[test]
fn test_equality() {
    let mut parser = create_parser("a == b");
    let expr_id = parser.parse_expression().expect("Failed to parse equality");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_inequality() {
    let mut parser = create_parser("a != b");
    let expr_id = parser.parse_expression().expect("Failed to parse inequality");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_less_than() {
    let mut parser = create_parser("a < b");
    let expr_id = parser.parse_expression().expect("Failed to parse less than");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_greater_than() {
    let mut parser = create_parser("a > b");
    let expr_id = parser.parse_expression().expect("Failed to parse greater than");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_less_equal() {
    let mut parser = create_parser("a <= b");
    let expr_id = parser.parse_expression().expect("Failed to parse less equal");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

#[test]
fn test_greater_equal() {
    let mut parser = create_parser("a >= b");
    let expr_id = parser.parse_expression().expect("Failed to parse greater equal");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::BinaryOpExpr(_)));
}

// ============================================================================
// Unary Operation Tests
// ============================================================================

#[test]
fn test_unary_minus() {
    let mut parser = create_parser("-42");
    let expr_id = parser.parse_expression().expect("Failed to parse unary minus");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::UnaryOpExpr(_)));
}

#[test]
fn test_unary_plus() {
    let mut parser = create_parser("+42");
    let expr_id = parser.parse_expression().expect("Failed to parse unary plus");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::UnaryOpExpr(_)));
}

#[test]
fn test_logical_not() {
    let mut parser = create_parser("not True");
    let expr_id = parser.parse_expression().expect("Failed to parse logical not");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::UnaryOpExpr(_)));
}

#[test]
fn test_bitwise_not() {
    let mut parser = create_parser("~42");
    let expr_id = parser.parse_expression().expect("Failed to parse bitwise not");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::UnaryOpExpr(_)));
}

// ============================================================================
// Container Expression Tests
// ============================================================================

#[test]
fn test_list_literal() {
    let mut parser = create_parser("[1, 2, 3]");
    let expr_id = parser.parse_expression().expect("Failed to parse list");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::ListExpr(_)));
}

#[test]
fn test_tuple_literal() {
    let mut parser = create_parser("(1, 2, 3)");
    let expr_id = parser.parse_expression().expect("Failed to parse tuple");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::TupleExpr(_)));
}

#[test]
fn test_set_literal() {
    let mut parser = create_parser("{1, 2, 3}");
    let expr_id = parser.parse_expression().expect("Failed to parse set");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::SetExpr(_)));
}

#[test]
fn test_dict_literal() {
    let mut parser = create_parser("{\"a\": 1, \"b\": 2}");
    let expr_id = parser.parse_expression().expect("Failed to parse dict");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::DictExpr(_)));
}

// ============================================================================
// F-String Tests  (tests format specifiers and conversions)
// ============================================================================

#[test]
fn test_fstring_simple() {
    let mut parser = create_parser("f\"hello {name}\"");
    let expr_id = parser.parse_expression().expect("Failed to parse simple f-string");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::FmtStringExpr(_)));
}

#[test]
fn test_fstring_with_format_spec() {
    let mut parser = create_parser("f\"{num:0.2f}\"");
    let expr_id = parser.parse_expression().expect("Failed to parse f-string with format spec");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::FmtStringExpr(_)));
}

#[test]
fn test_fstring_with_conversion() {
    let mut parser = create_parser("f\"{value!r}\"");
    let expr_id = parser.parse_expression().expect("Failed to parse f-string with conversion");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::FmtStringExpr(_)));
}

#[test]
fn test_fstring_multiple_expressions() {
    let mut parser = create_parser("f\"{a} and {b}\"");
    let expr_id =
        parser.parse_expression().expect("Failed to parse f-string with multiple expressions");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::FmtStringExpr(_)));
}

// ============================================================================
// Subscript and Attribute Access Tests
// ============================================================================

#[test]
fn test_subscript() {
    let mut parser = create_parser("arr[0]");
    let expr_id = parser.parse_expression().expect("Failed to parse subscript");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::SubscriptionExpr(_)));
}

#[test]
fn test_attribute_access() {
    let mut parser = create_parser("obj.attr");
    let expr_id = parser.parse_expression().expect("Failed to parse attribute access");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::AttributeExpr(_)));
}

#[test]
fn test_attribute_access_with_match_keyword() {
    let mut parser = create_parser("obj.match");
    let expr_id = parser.parse_expression().expect("Failed to parse attribute access with 'match'");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::AttributeExpr(_)));
}

// ============================================================================
// Function Call Tests
// ============================================================================

#[test]
fn test_function_call_no_args() {
    let mut parser = create_parser("func()");
    let expr_id = parser.parse_expression().expect("Failed to parse function call");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::CallExpr(_)));
}

#[test]
fn test_function_call_with_args() {
    let mut parser = create_parser("func(a, b, c)");
    let expr_id = parser.parse_expression().expect("Failed to parse function call with args");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::CallExpr(_)));
}

#[test]
fn test_function_call_with_kwargs() {
    let mut parser = create_parser("func(a=1, b=2)");
    let expr_id = parser.parse_expression().expect("Failed to parse function call with kwargs");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::CallExpr(_)));
}

// ============================================================================
// Ternary Expression Tests
// ============================================================================

#[test]
fn test_ternary_expression() {
    let mut parser = create_parser("x if condition else y");
    let expr_id = parser.parse_expression().expect("Failed to parse ternary");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::TernaryExpr(_)));
}

// ============================================================================
// Comprehension Tests
// ============================================================================

#[test]
fn test_list_comprehension() {
    let mut parser = create_parser("[x for x in range(10)]");
    let expr_id = parser.parse_expression().expect("Failed to parse list comprehension");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::ListComprehensionExpr(_)));
}

#[test]
fn test_list_comprehension_with_filter() {
    let mut parser = create_parser("[x for x in range(10) if x % 2 == 0]");
    let expr_id =
        parser.parse_expression().expect("Failed to parse list comprehension with filter");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::ListComprehensionExpr(_)));
}

#[test]
fn test_set_comprehension() {
    let mut parser = create_parser("{x for x in range(10)}");
    let expr_id = parser.parse_expression().expect("Failed to parse set comprehension");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::SetComprehensionExpr(_)));
}

#[test]
fn test_dict_comprehension() {
    let mut parser = create_parser("{k: v for k, v in items}");
    let expr_id = parser.parse_expression().expect("Failed to parse dict comprehension");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::DictComprehensionExpr(_)));
}

#[test]
fn test_generator_expression() {
    let mut parser = create_parser("(x for x in range(10))");
    let expr_id = parser.parse_expression().expect("Failed to parse generator expression");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::GeneratorExpr(_)));
}

// ============================================================================
// Lambda Expression Tests
// ============================================================================

#[test]
fn test_lambda_no_args() {
    let mut parser = create_parser("lambda: 42");
    let expr_id = parser.parse_expression().expect("Failed to parse lambda with no args");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LambdaExpr(_)));
}

#[test]
fn test_lambda_with_args() {
    let mut parser = create_parser("lambda x, y: x + y");
    let expr_id = parser.parse_expression().expect("Failed to parse lambda with args");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::LambdaExpr(_)));
}

// ============================================================================
// Starred Expression Tests
// ============================================================================

#[test]
fn test_starred_expression() {
    let mut parser = create_parser("*args");
    let expr_id = parser.parse_expression().expect("Failed to parse starred expression");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::StarredExpr(_)));
}

// ============================================================================
// Await and Yield Expression Tests
// ============================================================================

#[test]
fn test_await_expression() {
    let mut parser = create_parser("await coroutine()");
    let expr_id = parser.parse_expression().expect("Failed to parse await expression");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::AwaitExpr(_)));
}

#[test]
fn test_yield_expression() {
    let mut parser = create_parser("yield value");
    let expr_id = parser.parse_expression().expect("Failed to parse yield expression");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::YieldExpr(_)));
}

#[test]
fn test_yield_from_expression() {
    let mut parser = create_parser("yield from iterable");
    let expr_id = parser.parse_expression().expect("Failed to parse yield from expression");
    let node = parser.ast().get_node(expr_id).expect("Node not found");

    assert_eq!(node.kind, NodeKind::Expression);
    assert!(matches!(node.data, AnyNode::YieldFromExpr(_)));
}
