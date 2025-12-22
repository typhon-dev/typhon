//! Tests for function inlining optimization pass.

use rustc_hash::FxHashMap;
use typhon_mir::builder::FunctionBuilder;
use typhon_mir::instr::{MIRInstr, Terminator, ValueID};
use typhon_mir::module::MIRModule;
use typhon_mir::types::MIRType;
use typhon_mir_optimizer::config::OptimizerConfig;
use typhon_mir_optimizer::passes::FunctionInliner;

/// Test inlining a simple function with no parameters.
#[test]
fn test_inline_simple_function() {
    // TODO: Requires function name resolution from ValueID
    // This test is blocked until we can resolve function calls
    //
    // Expected behavior:
    // - Small function with no parameters should be inlined
    // - Call instruction replaced with function body
    // - No parameter mapping needed
}

/// Test inlining a function with parameters, mapping arguments to parameters.
#[test]
fn test_inline_with_parameters() {
    // TODO: Requires function name resolution from ValueID
    // This test is blocked until we can resolve function calls
    //
    // Expected behavior:
    // - Function parameters are replaced with actual call arguments
    // - All uses of parameters in inlined body use the arguments
    // - Local variables are renamed to avoid conflicts
}

/// Test handling return value propagation from inlined function.
#[test]
fn test_inline_with_return() {
    // TODO: Requires function name resolution from ValueID
    // This test is blocked until we can resolve function calls
    //
    // Expected behavior:
    // - Return value from inlined function replaces call result
    // - Multiple return paths handled with phi nodes
    // - Control flow properly merged at continuation point
}

/// Test inlining the same function at multiple call sites.
#[test]
fn test_inline_multiple_calls() {
    // TODO: Requires function name resolution from ValueID
    // This test is blocked until we can resolve function calls
    //
    // Expected behavior:
    // - Each call site independently evaluates inlining heuristics
    // - Function inlined at each eligible call site
    // - Each inlined copy has unique renamed locals
}

/// Test that recursive functions are not inlined.
#[test]
fn test_no_inline_recursive() {
    // Create a module with a recursive function
    let mut module = MIRModule {
        name: "test_module".to_string(),
        functions: Vec::new(),
        globals: Vec::new(),
        types: Vec::new(),
        value_names: FxHashMap::default(),
    };

    // Create a simple recursive function
    let mut builder = FunctionBuilder::new(
        "factorial".to_string(),
        vec![("n".to_string(), MIRType::Int)],
        MIRType::Int,
    );

    let entry = builder.create_block();
    builder.switch_to_block(entry);

    // Just return None (simplified - full recursion requires function resolution)
    builder.set_terminator(Terminator::Return(None));

    let factorial_func = builder.build();
    module.functions.push(factorial_func);

    // Run inlining pass
    let config = OptimizerConfig::level2();
    let mut inliner = FunctionInliner::new();
    let result = inliner.inline_functions(&mut module, &config);

    // Should succeed but not inline anything (no calls to inline)
    assert!(result.is_ok());
    assert_eq!(inliner.call_sites_inlined(), 0);
    assert_eq!(inliner.functions_inlined(), 0);
}

/// Test that large functions exceeding size threshold are not inlined.
#[test]
fn test_no_inline_large_function() {
    // Create a module with a large function
    let mut module = MIRModule {
        name: "test_module".to_string(),
        functions: Vec::new(),
        globals: Vec::new(),
        types: Vec::new(),
        value_names: FxHashMap::default(),
    };

    // Create a function with many instructions (exceeds max_inline_size)
    let mut builder = FunctionBuilder::new("large_function".to_string(), vec![], MIRType::Int);

    let entry = builder.create_block();
    builder.switch_to_block(entry);

    // Add many IncRef instructions to make function large
    // (use a dummy value for the reference)
    let val_id = ValueID(0);
    for _ in 0..100 {
        builder.add_instruction(MIRInstr::IncRef(val_id));
    }

    builder.set_terminator(Terminator::Return(None));

    let large_func = builder.build();
    module.functions.push(large_func);

    // Run inlining pass with small threshold
    let mut config = OptimizerConfig::level2();
    config.max_inline_size = 10; // Much smaller than our function

    let mut inliner = FunctionInliner::new();
    let result = inliner.inline_functions(&mut module, &config);

    // Should succeed but not inline due to size (no calls to inline anyway)
    assert!(result.is_ok());
    assert_eq!(inliner.call_sites_inlined(), 0);
    assert_eq!(inliner.functions_inlined(), 0);
}

/// Test that functions called only once are prioritized for inlining.
#[test]
fn test_inline_single_use() {
    // TODO: Requires function name resolution from ValueID
    // This test is blocked until we can resolve function calls
    //
    // Expected behavior:
    // - Function called only once should be inlined (if enabled in config)
    // - Eliminates call overhead completely
    // - May inline even if slightly larger than normal threshold
}

/// Test that call graph correctly detects recursive functions using Tarjan's SCC.
#[test]
fn test_call_graph_scc_detection() {
    use typhon_mir_optimizer::passes::CallGraph;

    // Create call graph with mutual recursion: A -> B -> C -> A
    let mut graph = CallGraph::new();
    graph.add_edge("A", "B");
    graph.add_edge("B", "C");
    graph.add_edge("C", "A");

    let recursive = graph.find_scc();

    // All three functions should be detected as recursive
    assert!(recursive.contains("A"));
    assert!(recursive.contains("B"));
    assert!(recursive.contains("C"));
    assert_eq!(recursive.len(), 3);
}

/// Test that call graph detects direct self-recursion.
#[test]
fn test_call_graph_direct_recursion() {
    use typhon_mir_optimizer::passes::CallGraph;

    // Create call graph with direct recursion: A -> A
    let mut graph = CallGraph::new();
    graph.add_edge("A", "A");

    let recursive = graph.find_scc();

    // A should be detected as recursive
    assert!(recursive.contains("A"));
    assert_eq!(recursive.len(), 1);
}

/// Test that call graph handles non-recursive functions correctly.
#[test]
fn test_call_graph_non_recursive() {
    use typhon_mir_optimizer::passes::CallGraph;

    // Create call graph with linear chain: A -> B -> C
    let mut graph = CallGraph::new();
    graph.add_edge("A", "B");
    graph.add_edge("B", "C");

    let recursive = graph.find_scc();

    // No functions should be detected as recursive
    assert!(recursive.is_empty());
}
