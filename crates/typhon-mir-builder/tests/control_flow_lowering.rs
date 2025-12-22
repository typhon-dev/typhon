//! Integration tests for the lowering pipeline
//!
//! These tests verify that the complete lowering process works correctly
//! by taking Python source code through parsing, analysis, and lowering to MIR.

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

/// Helper function to lower an AST to MIR
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
fn test_simple_function() {
    let source = r"
def add(a, b):
    return a + b
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower simple function: {:?}", result.err());

    let module = result.unwrap();

    assert_eq!(module.functions.len(), 1, "Expected 1 function in module");

    let func = &module.functions[0];

    assert_eq!(func.name, "add", "Function name mismatch");
    assert_eq!(func.params.len(), 2, "Expected 2 parameters");
}

#[test]
fn test_function_with_if_statement() {
    let source = r"
def max_value(a, b):
    if a > b:
        return a
    else:
        return b
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower function with if statement: {:?}", result.err());

    let module = result.unwrap();

    assert_eq!(module.functions.len(), 1);

    let func = &module.functions[0];

    assert_eq!(func.name, "max_value");
    assert_eq!(func.params.len(), 2);

    // Should have multiple blocks for if/else
    assert!(func.blocks.len() > 1, "Expected multiple blocks for if/else");
}

#[test]
fn test_function_with_while_loop() {
    let source = r"
def count_down(n):
    while n > 0:
        n = n - 1
    return n
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower function with while loop: {:?}", result.err());

    let module = result.unwrap();
    assert_eq!(module.functions.len(), 1);

    let func = &module.functions[0];
    assert_eq!(func.name, "count_down");

    // Should have multiple blocks for loop
    assert!(func.blocks.len() > 1, "Expected multiple blocks for while loop");
}

#[test]
fn test_recursive_fibonacci() {
    let source = r"
def fibonacci(n):
    if n <= 1:
        return n
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower recursive fibonacci function: {:?}", result.err());

    let module = result.unwrap();

    assert_eq!(module.functions.len(), 1);

    let func = &module.functions[0];

    assert_eq!(func.name, "fibonacci");
    assert_eq!(func.params.len(), 1);

    // Should have multiple blocks for if/else and recursive calls
    assert!(func.blocks.len() > 1, "Expected multiple blocks for fibonacci");
}

#[test]
fn test_function_with_multiple_statements() {
    let source = r"
def calculate(x, y):
    a = x + y
    b = x - y
    c = a * b
    return c
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(
        result.is_ok(),
        "Failed to lower function with multiple statements: {:?}",
        result.err()
    );

    let module = result.unwrap();

    assert_eq!(module.functions.len(), 1);

    let func = &module.functions[0];

    assert_eq!(func.name, "calculate");
    assert_eq!(func.params.len(), 2);
}
