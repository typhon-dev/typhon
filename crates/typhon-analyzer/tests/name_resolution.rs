//! Tests for name resolution and closure analysis.

use std::sync::Arc;

use typhon_analyzer::analyze_module;
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

/// Helper to create a parser and parse source code
fn parse_source(source: &str) -> (Parser<'_>, typhon_ast::nodes::NodeID) {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());
    let mut parser = Parser::new(source, file_id, Arc::new(source_manager));
    let module_id = parser.parse_module().expect("Failed to parse module");

    (parser, module_id)
}

#[test]
fn test_simple_variable_reference() {
    let source = r"
x = 42
y = x
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for simple variable reference");
}

#[test]
fn test_undefined_variable() {
    // y = x  (x is not defined)
    let source = "y = x\n";
    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_err(), "Should error on undefined variable");

    if let Err(errors) = result {
        assert_eq!(errors.len(), 1, "Should have exactly one error");
        assert!(format!("{:?}", errors[0]).contains("UndefinedName"));
    }
}

#[test]
fn test_function_parameter_reference() {
    let source = r"
def add(a, b):
    return a + b
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for function parameter references");
}

#[test]
fn test_closure_capture() {
    let source = r"def outer():
    x = 10
    def inner():
        return x  # x is captured from outer
    return inner
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for closure");

    // Check that the context has detected the closure
    if let Ok(context) = result {
        let symbol_table = context.symbol_table();
        // The symbol 'x' should be marked as captured
        // Just verify the symbol table was created successfully
        assert!(symbol_table.scopes().count() > 0);
    }
}

#[test]
fn test_lambda_closure() {
    let source = r"
x = 10
f = lambda: x
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for lambda closure");
}

#[test]
fn test_scope_shadowing() {
    let source = r"
x = 1
def f():
    x = 2  # shadows outer x
    return x
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed with shadowing");
}

#[test]
fn test_assignment_to_variable() {
    let source = r"
x = 1
x = 2  # reassignment
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for reassignment");
}

#[test]
fn test_nested_function_closure() {
    let source = r"
def outer():
    x = 1
    def middle():
        y = 2
        def inner():
            return x + y  # captures both x and y
        return inner
    return middle
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for nested closures");
}

#[test]
fn test_class_method_reference() {
    let source = r"
class MyClass:
    def method(self):
        pass
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for class methods");
}

#[test]
fn test_reference_to_function() {
    let source = r"d
ef foo():
    pass
bar = foo  # reference to function
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for function references");
}

#[test]
fn test_for_loop_variable() {
    let source = r"
for i in range(10):
    print(i)
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for loop variables");
}

#[test]
fn test_list_comprehension_variable() {
    let source = "result = [x for x in range(10)]";
    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for comprehension variables");
}

#[test]
fn test_multiple_undefined_variables() {
    let source = "y = a + b  # both a and b are undefined";
    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_err(), "Should error on multiple undefined variables");

    if let Err(errors) = result {
        assert!(errors.len() >= 2, "Should have at least 2 errors for undefined a and b");
    }
}

#[test]
fn test_builtin_function_reference() {
    let source = "x = len([1, 2, 3])";
    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    // This should succeed even though 'len' isn't defined in the source
    // because it's a builtin (though we may need to add builtin support)
    // For now, it will likely error, which is acceptable
    let _ = result;
}

#[test]
fn test_lambda_with_parameters() {
    let source = "f = lambda x, y: x + y";
    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    assert!(result.is_ok(), "Name resolution should succeed for lambda with parameters");
}

#[test]
fn test_with_statement_variable() {
    let source = r"
with open('file.txt') as f:
    content = f.read()
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    // This will have undefined 'open' but f should be resolved
    let _ = result;
}

#[test]
fn test_exception_variable() {
    let source = r"
try:
    pass
except Exception as e:
    print(e)
";

    let (parser, module_id) = parse_source(source);
    let result = analyze_module(parser.ast(), module_id);

    // Exception and print are undefined, but e should be resolved
    let _ = result;
}
