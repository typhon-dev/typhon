//! Integration tests for class lowering
//!
//! These tests verify that class declarations are correctly lowered to MIR.

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

/// Helper function to lower a module containing classes
fn lower_module(parser: &Parser<'_>, module_id: NodeID) -> LoweringResult<MIRModule> {
    let mut context = LoweringContext::new(parser.ast(), "test_module".to_string());

    // Get the module node
    if let Some(module_node) = parser.ast().get_node(module_id)
        && let AnyNode::Module(module) = &module_node.data
    {
        // Lower all class declarations in the module
        for &stmt_id in &module.statements {
            if let Some(stmt_node) = parser.ast().get_node(stmt_id) {
                match &stmt_node.data {
                    AnyNode::ClassDecl(_) => {
                        let _ = context.lower_class(stmt_id)?;
                    }
                    AnyNode::FunctionDecl(_) => {
                        context.lower_function(stmt_id)?;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(context.build())
}

#[test]
fn test_simple_class_with_fields() {
    let source = r"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower simple class: {:?}", result.err());

    let module = result.unwrap();

    // Should have 1 type definition
    assert_eq!(module.types.len(), 1, "Expected 1 class type");

    let class = &module.types[0];
    assert_eq!(class.name, "Point", "Class name mismatch");

    // Should have extracted 2 fields (x and y)
    assert_eq!(class.fields.len(), 2, "Expected 2 fields");
    assert_eq!(class.fields[0].name, "x", "First field should be 'x'");
    assert_eq!(class.fields[1].name, "y", "Second field should be 'y'");

    // Should have 1 method (__init__)
    assert_eq!(class.methods.len(), 1, "Expected 1 method");
    assert_eq!(class.methods[0].name, "__init__", "Method should be __init__");

    // Should have initializer set
    assert!(class.initializer.is_some(), "Initializer should be set");
}

#[test]
fn test_class_with_methods() {
    let source = r"
class Calculator:
    def __init__(self, initial):
        self.value = initial

    def add(self, x):
        self.value = self.value + x
        return self.value

    def multiply(self, x):
        self.value = self.value * x
        return self.value
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower class with methods: {:?}", result.err());

    let module = result.unwrap();

    assert_eq!(module.types.len(), 1);

    let class = &module.types[0];
    assert_eq!(class.name, "Calculator");

    // Should have 3 methods (__init__, add, multiply)
    assert_eq!(class.methods.len(), 3, "Expected 3 methods");

    let method_names: Vec<&str> = class.methods.iter().map(|m| m.name.as_str()).collect();
    assert!(method_names.contains(&"__init__"), "Should have __init__ method");
    assert!(method_names.contains(&"add"), "Should have add method");
    assert!(method_names.contains(&"multiply"), "Should have multiply method");

    // Methods should be mangled with class name
    for method in &class.methods {
        assert!(
            method.function_name.starts_with("Calculator__"),
            "Method {} should be mangled with class name",
            method.name
        );
    }
}

#[test]
fn test_class_with_private_methods() {
    let source = r"
class SecretKeeper:
    def __init__(self, secret):
        self.secret = secret

    def _get_secret(self):
        return self.secret

    def reveal(self):
        return self._get_secret()
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower class with private methods: {:?}", result.err());

    let module = result.unwrap();

    assert_eq!(module.types.len(), 1);

    let class = &module.types[0];

    // Find the private method
    let private_method = class.methods.iter().find(|m| m.name == "_get_secret");
    assert!(private_method.is_some(), "Should have _get_secret method");

    // Verify it's marked as private
    assert!(private_method.unwrap().is_private, "_get_secret should be marked as private");

    // Public method should not be private
    let public_method = class.methods.iter().find(|m| m.name == "reveal");
    assert!(public_method.is_some(), "Should have reveal method");
    assert!(!public_method.unwrap().is_private, "reveal should not be marked as private");
}

#[test]
fn test_empty_class() {
    let source = r"
class Empty:
    pass
";

    let (parser, module_id) = parse_source(source);
    let result = lower_module(&parser, module_id);

    assert!(result.is_ok(), "Failed to lower empty class: {:?}", result.err());

    let module = result.unwrap();

    assert_eq!(module.types.len(), 1);

    let class = &module.types[0];
    assert_eq!(class.name, "Empty");

    // Empty class should have no fields and no methods
    assert_eq!(class.fields.len(), 0, "Empty class should have no fields");
    assert_eq!(class.methods.len(), 0, "Empty class should have no methods");
}
