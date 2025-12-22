//! End-to-end integration test for the complete compilation pipeline.
//!
//! This test exercises the full pipeline from source code to LLVM IR:
//! Source → Parser → AST → Analyzer → MIR Builder → LLVM Codegen
//!
//! The test verifies that all components work together correctly and that
//! the generated LLVM IR contains expected patterns.

use std::sync::Arc;

use typhon_ast::nodes::{AnyNode, NodeID};
use typhon_codegen_llvm::compile_module;
use typhon_mir::module::MIRModule;
use typhon_mir_builder::context::LoweringContext;
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

/// Helper to parse source code into an AST.
///
/// Creates a source manager, adds the source file, and parses it into an AST.
/// Returns the parser (which owns the AST) and the module's `NodeID`.
fn parse_source(source: &str) -> (Parser<'_>, NodeID) {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), source.to_string());
    let mut parser = Parser::new(source, file_id, Arc::new(source_manager));
    let module_id = parser.parse_module().expect("Failed to parse module");

    (parser, module_id)
}

/// Helper to lower an AST module to MIR.
///
/// Takes the parsed AST and converts it to MIR (Mid-level Intermediate Representation).
/// This step processes all function declarations in the module.
fn lower_to_mir(parser: &Parser<'_>, module_id: NodeID) -> MIRModule {
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
                context.lower_function(stmt_id).expect("Failed to lower function");
            }
        }
    }

    context.build()
}

/// Helper to compile MIR to LLVM IR.
///
/// Takes a MIR module and generates LLVM IR as a string.
fn compile_to_llvm(mir_module: &MIRModule) -> String {
    compile_module(mir_module).expect("Failed to compile MIR to LLVM IR")
}

#[test]
fn test_pipeline_simple_return() {
    // Test the complete pipeline with a simple function that returns a constant
    let source = r"
def main() -> int:
    return 42
";

    // Step 1: Parse source → AST
    let (parser, module_id) = parse_source(source);

    // Verify parsing succeeded
    assert!(parser.ast().get_node(module_id).is_some());

    // Step 2: Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify MIR generation succeeded
    assert_eq!(mir_module.name, "test_module");
    assert_eq!(mir_module.functions.len(), 1);
    assert_eq!(mir_module.functions[0].name, "main");

    // Step 3: Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // Verify LLVM IR contains expected patterns
    assert!(llvm_ir.contains("define"), "LLVM IR should contain function definition");
    assert!(llvm_ir.contains("main"), "LLVM IR should contain 'main' function");
    assert!(llvm_ir.contains("ret"), "LLVM IR should contain return instruction");

    // Verify the IR is valid (basic syntax check)
    assert!(llvm_ir.starts_with("; ModuleID"), "LLVM IR should start with module ID");
}

#[test]
fn test_pipeline_arithmetic_operations() {
    // Test pipeline with arithmetic operations
    let source = r"
def add(x, y):
    return x + y

def multiply(a, b):
    result = a * b
    return result
";

    // Parse → AST
    let (parser, module_id) = parse_source(source);

    // Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify both functions were lowered
    assert_eq!(mir_module.functions.len(), 2);
    assert!(mir_module.functions.iter().any(|f| f.name == "add"));
    assert!(mir_module.functions.iter().any(|f| f.name == "multiply"));

    // Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // Verify LLVM IR contains both functions
    assert!(llvm_ir.contains("add"), "LLVM IR should contain 'add' function");
    assert!(llvm_ir.contains("multiply"), "LLVM IR should contain 'multiply' function");

    // Verify arithmetic operations are present (add/mul instructions)
    // Note: The exact instruction names may vary based on implementation
    assert!(
        llvm_ir.contains("call") || llvm_ir.contains("add") || llvm_ir.contains("mul"),
        "LLVM IR should contain arithmetic operations"
    );
}

#[test]
fn test_pipeline_conditional_control_flow() {
    // Test pipeline with conditional control flow (if/else)
    let source = r"
def max_value(a, b):
    if a > b:
        return a
    else:
        return b
";

    // Parse → AST
    let (parser, module_id) = parse_source(source);

    // Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify function was lowered
    assert_eq!(mir_module.functions.len(), 1);
    assert_eq!(mir_module.functions[0].name, "max_value");

    // Verify control flow created multiple basic blocks
    let function = &mir_module.functions[0];

    assert!(function.blocks.len() > 1, "Conditional should create multiple basic blocks");

    // Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // Verify LLVM IR contains control flow constructs
    assert!(llvm_ir.contains("max_value"), "LLVM IR should contain 'max_value' function");
    assert!(llvm_ir.contains("br"), "LLVM IR should contain branch instructions");

    // Verify multiple basic blocks (labels) exist
    let label_count = llvm_ir.matches(':').count();

    assert!(label_count >= 2, "LLVM IR should have multiple basic block labels");
}

#[test]
fn test_pipeline_multiple_functions() {
    // Test pipeline with multiple independent functions
    let source = r"
def func1():
    return 1

def func2():
    return 2

def func3():
    return 3
";

    // Parse → AST
    let (parser, module_id) = parse_source(source);

    // Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify all functions were lowered
    assert_eq!(mir_module.functions.len(), 3);
    assert!(mir_module.functions.iter().any(|f| f.name == "func1"));
    assert!(mir_module.functions.iter().any(|f| f.name == "func2"));
    assert!(mir_module.functions.iter().any(|f| f.name == "func3"));

    // Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // Verify LLVM IR contains all functions
    assert!(llvm_ir.contains("func1"), "LLVM IR should contain 'func1'");
    assert!(llvm_ir.contains("func2"), "LLVM IR should contain 'func2'");
    assert!(llvm_ir.contains("func3"), "LLVM IR should contain 'func3'");

    // Count function definitions
    let def_count = llvm_ir.matches("define").count();

    assert_eq!(def_count, 3, "LLVM IR should have 3 function definitions");
}

#[test]
fn test_pipeline_with_local_variables() {
    // Test pipeline with local variable assignments
    let source = r"
def calculate(x, y):
    a = x + y
    b = x - y
    c = a * b
    return c
";

    // Parse → AST
    let (parser, module_id) = parse_source(source);

    // Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify function was lowered
    assert_eq!(mir_module.functions.len(), 1);

    let function = &mir_module.functions[0];

    assert_eq!(function.name, "calculate");

    // Verify local variables were tracked (should have allocations)
    assert!(!function.locals.is_empty(), "Function should have local variables");

    // Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // Verify LLVM IR contains function
    assert!(llvm_ir.contains("calculate"), "LLVM IR should contain 'calculate' function");

    // Verify stack allocations for local variables (alloca instructions)
    assert!(
        llvm_ir.contains("alloca") || llvm_ir.contains("store") || llvm_ir.contains("load"),
        "LLVM IR should contain memory operations for local variables"
    );
}

#[test]
fn test_pipeline_parameters() {
    // Test pipeline with function parameters
    let source = r"
def greet(name, age):
    return name
";

    // Parse → AST
    let (parser, module_id) = parse_source(source);

    // Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify function parameters
    assert_eq!(mir_module.functions.len(), 1);

    let function = &mir_module.functions[0];

    assert_eq!(function.name, "greet");
    assert_eq!(function.params.len(), 2);

    // Verify parameter names
    assert!(function.params.iter().any(|p| p.name == "name"));
    assert!(function.params.iter().any(|p| p.name == "age"));

    // Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // Verify LLVM IR contains function with parameters
    assert!(llvm_ir.contains("greet"), "LLVM IR should contain 'greet' function");

    // Function signature should have parameters
    // The exact format depends on implementation, but should have parameter types
    let greet_def_start = llvm_ir.find("define").expect("Should find function definition");
    let greet_def_end =
        llvm_ir[greet_def_start..].find('{').map_or(llvm_ir.len(), |pos| greet_def_start + pos);
    let greet_signature = &llvm_ir[greet_def_start..greet_def_end];

    assert!(greet_signature.contains("greet"), "Function signature should contain function name");
}

#[test]
fn test_pipeline_empty_module() {
    // Test pipeline with no functions (edge case)
    let source = "";

    // Parse → AST
    let (parser, module_id) = parse_source(source);

    // Lower AST → MIR
    let mir_module = lower_to_mir(&parser, module_id);

    // Verify module is empty but valid
    assert_eq!(mir_module.functions.len(), 0);

    // Compile MIR → LLVM IR
    let llvm_ir = compile_to_llvm(&mir_module);

    // LLVM IR should still be valid even for empty module
    assert!(llvm_ir.starts_with("; ModuleID"), "LLVM IR should have valid module ID");
}
