//! Integration tests for closure lowering
//!
//! These tests verify that nested functions with captured variables are correctly lowered to MIR.

use std::sync::Arc;

use typhon_ast::nodes::{AnyNode, NodeID};
use typhon_mir::module::MIRModule;
use typhon_mir_builder::context::LoweringContext;
use typhon_mir_builder::error::LoweringResult;
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

/// Helper to create a parser and parse source code
fn parse_source(source: &str) -> (Parser<'_>, NodeID) {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());
    let mut parser = Parser::new(source, file_id, Arc::new(source_manager));
    let module_id = parser.parse_module().expect("Failed to parse module");

    (parser, module_id)
}

/// Helper function to lower a module containing functions with closures
fn lower_module(parser: &Parser<'_>, module_id: NodeID) -> LoweringResult<MIRModule> {
    let mut context = LoweringContext::new(parser.ast(), "test_module".to_string());

    // Get the module node
    if let Some(module_node) = parser.ast().get_node(module_id)
        && let AnyNode::Module(module) = &module_node.data
    {
        // Lower all function declarations in the module
        for &stmt_id in &module.statements {
            if let Some(stmt_node) = parser.ast().get_node(stmt_id)
                && let AnyNode::FunctionDecl(_) = &stmt_node.data
            {
                context.lower_function(stmt_id)?;
            }
        }
    }

    Ok(context.build())
}

#[test]
fn test_simple_closure_capture() {
    let source = r"
def outer(x):
    def inner():
        return x
    return inner
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower simple closure: {:?}", result.err());

    let module = result.unwrap();

    // Should have 2 functions: outer and outer__inner
    assert!(module.functions.len() >= 2, "Expected at least 2 functions (outer and inner)");

    // Find the inner function (should be mangled as outer__inner)
    let inner_func = module.functions.iter().find(|f| f.name.contains("inner"));
    assert!(inner_func.is_some(), "Should have an inner function");

    let inner = inner_func.unwrap();

    // Inner function should have captures
    assert!(!inner.captures.is_empty(), "Inner function should have captures");
    assert_eq!(inner.captures.len(), 1, "Inner function should capture 1 variable");
    assert_eq!(inner.captures[0].name, "x", "Should capture variable 'x'");
}

#[test]
fn test_closure_with_multiple_captures() {
    let source = r"
def make_adder(x, y):
    def add(z):
        return x + y + z
    return add
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower closure with multiple captures: {:?}", result.err());

    let module = result.unwrap();

    // Find the inner function
    let inner_func = module.functions.iter().find(|f| f.name.contains("add"));
    assert!(inner_func.is_some(), "Should have an add function");

    let inner = inner_func.unwrap();

    // Should capture both x and y
    assert_eq!(inner.captures.len(), 2, "Should capture 2 variables");

    let capture_names: Vec<&str> = inner.captures.iter().map(|c| c.name.as_str()).collect();
    assert!(capture_names.contains(&"x"), "Should capture variable 'x'");
    assert!(capture_names.contains(&"y"), "Should capture variable 'y'");
}

#[test]
fn test_nested_closures() {
    let source = r"
def outer(x):
    def middle(y):
        def inner(z):
            return x + y + z
        return inner
    return middle
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower nested closures: {:?}", result.err());

    let module = result.unwrap();

    // Should have 3 functions: outer, outer__middle, outer__middle__inner
    assert!(module.functions.len() >= 3, "Expected at least 3 functions");

    // Find the innermost function
    let inner_func = module.functions.iter().find(|f| f.name.contains("inner"));
    assert!(inner_func.is_some(), "Should have an inner function");

    let inner = inner_func.unwrap();

    // Inner should capture x and y (not z, which is a parameter)
    assert_eq!(inner.captures.len(), 2, "Innermost function should capture 2 variables");

    let capture_names: Vec<&str> = inner.captures.iter().map(|c| c.name.as_str()).collect();
    assert!(capture_names.contains(&"x"), "Should capture variable 'x'");
    assert!(capture_names.contains(&"y"), "Should capture variable 'y'");
}

#[test]
fn test_closure_no_captures() {
    let source = r"
def outer():
    def inner(x):
        return x + 1
    return inner
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower closure with no captures: {:?}", result.err());

    let module = result.unwrap();

    // Find the inner function
    let inner_func = module.functions.iter().find(|f| f.name.contains("inner"));
    assert!(inner_func.is_some(), "Should have an inner function");

    let inner = inner_func.unwrap();

    // Should have no captures since it only uses its own parameter
    assert_eq!(inner.captures.len(), 0, "Inner function should have no captures");
}
