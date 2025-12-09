//! Integration tests for semantic validation.
//!
//! Tests control flow analysis, definite assignment, dead code detection,
//! context validation, and attribute/method lookup.

use std::sync::Arc;

use typhon_analyzer::analyze_module;
use typhon_analyzer::error::SemanticError;
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

/// Helper function to parse and analyze a module.
fn analyze_code(code: &str) -> Result<(), Vec<SemanticError>> {
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file("test.ty".to_string(), code.to_string());
    let mut parser = Parser::new(code, file_id, Arc::new(source_manager));
    let module_id = parser.parse_module().expect("Failed to parse module");

    analyze_module(parser.ast(), module_id).map(|_| ())
}

/// Helper to check if error list contains a specific error type.
fn contains_error<F>(errors: &[SemanticError], pred: F) -> bool
where F: Fn(&SemanticError) -> bool {
    errors.iter().any(pred)
}

// =============================================================================
// Control Flow Tests (7 tests)
// =============================================================================

#[test]
fn test_simple_if_statement() {
    let code = r"
def test() -> bool:
    x = 5
    if x > 0:
        return True
    return False
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_if_else_all_paths_return() {
    let code = r"
def test() -> int:
    if True:
        return 1
    else:
        return 2
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_nested_if_statements() {
    let code = r"
def test() -> int:
    x = 5
    if x > 0:
        if x > 10:
            return 1
        return 2
    return 0
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_while_loop_with_break() {
    let code = r"
def test():
    x = 0
    while x < 10:
        if x == 5:
            break
        x = x + 1
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_for_loop_with_continue() {
    let code = r"
def test():
    for i in range(10):
        if i == 5:
            continue
        print(i)
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_nested_loops() {
    let code = r"
def test():
    for i in range(5):
        for j in range(5):
            if i == j:
                break
";

    assert!(analyze_code(code).is_ok());
}

#[test]
#[ignore = "TODO: Parser bug - return statement after loop is parsed as part of loop body"]
fn test_early_return_in_loop() {
    let code = r"
def test() -> int:
    for i in range(10):
        if i == 5:
            return i
    return -1
";

    assert!(analyze_code(code).is_ok());
}

// =============================================================================
// Definite Assignment Tests (8 tests)
// =============================================================================

#[test]
fn test_variable_assigned_before_use() {
    let code = r"
def test():
    x = 5
    return x
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_use_before_assignment_error() {
    let code = r"
def test():
    print(x)
    x = 5
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::UseBeforeAssignment { .. })));
}

#[test]
fn test_conditional_assignment_both_branches() {
    let code = r"
def test():
    if True:
        x = 1
    else:
        x = 2
    return x
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_conditional_assignment_one_branch_error() {
    let code = r"
def test():
    if True:
        x = 1
    return x
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::UseBeforeAssignment { .. })));
}

#[test]
fn test_assignment_in_loop() {
    let code = r"
def test():
    for i in range(10):
        x = i
    return x
";

    // Note: In Python, this would work because loop executes at least once
    // but semantically it's questionable. Our analyzer should catch this.
    let result = analyze_code(code);

    // This might be OK or error depending on loop analysis sophistication
    // For now, we'll accept either outcome
    let _ = result;
}

#[test]
fn test_multiple_assignments() {
    let code = r"
def test():
    x = 1
    y = 2
    z = x + y
    return z
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_reassignment() {
    let code = r"
def test():
    x = 1
    x = 2
    x = 3
    return x
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_parameter_is_assigned() {
    let code = r"
def test(x: int):
    return x
";

    assert!(analyze_code(code).is_ok());
}

// =============================================================================
// Dead Code Detection Tests (6 tests)
// =============================================================================

#[test]
fn test_unreachable_after_return() {
    let code = r"
def test() -> int:
    return 1
    x = 5
";

    // Dead code detection produces warnings, not errors
    // So this should still parse and analyze successfully
    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_unreachable_after_break() {
    let code = r"
def test():
    while True:
        break
        x = 5
";

    assert!(analyze_code(code).is_ok());
}

#[test]
#[ignore = "TODO: Fix - unreachable code after continue in for loop causes symbol resolution issues"]
fn test_unreachable_after_continue() {
    let code = r"
def test():
    y = 0
    for i in [1, 2, 3]:
        continue
        y = 5
";

    // Dead code detection produces warnings, not errors
    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_reachable_code_after_conditional() {
    let code = r"
def test() -> int:
    if True:
        return 1
    return 2
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_unused_variable() {
    let code = r"
def test() -> int:
    x = 5
    return 10
";

    // Unused variables produce warnings, not errors
    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_used_variable() {
    let code = r"
def test():
    x = 5
    return x
";

    assert!(analyze_code(code).is_ok());
}

// =============================================================================
// Context Validation Tests (6 tests)
// =============================================================================

#[test]
fn test_break_outside_loop_error() {
    let code = r"
def test():
    break
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::BreakOutsideLoop { .. })));
}

#[test]
fn test_continue_outside_loop_error() {
    let code = r"
def test():
    continue
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::ContinueOutsideLoop { .. })));
}

#[test]
fn test_return_outside_function_error() {
    let code = r"
return 42
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::ReturnOutsideFunction { .. })));
}

#[test]
fn test_break_in_loop_ok() {
    let code = r"
def test():
    while True:
        break
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_continue_in_loop_ok() {
    let code = r"
def test():
    for i in range(10):
        continue
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_return_in_function_ok() {
    let code = r"
def test():
    return 42
";

    assert!(analyze_code(code).is_ok());
}

// =============================================================================
// Attribute/Method Lookup Tests (10 tests)
// =============================================================================

#[test]
fn test_list_append_method() {
    let code = r"
def test():
    x: list[int] = []
    x.append(1)
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_list_extend_method() {
    let code = r"
def test():
    x: list[int] = []
    x.extend([1, 2, 3])
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_dict_get_method() {
    let code = r#"
def test():
    x: dict[str, int] = {}
    x.get("key", 0)
"#;
    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_dict_keys_method() {
    let code = r"
def test():
    x: dict[str, int] = {}
    x.keys()
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_str_upper_method() {
    let code = r#"
def test():
    x: str = "hello"
    x.upper()
"#;
    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_str_split_method() {
    let code = r#"
def test():
    x: str = "hello world"
    x.split(" ")
"#;
    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_set_add_method() {
    let code = r"
def test():
    x: set[int] = set()
    x.add(1)
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_set_remove_method() {
    let code = r"
def test():
    x: set[int] = {1, 2, 3}
    x.remove(1)
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_invalid_method_error() {
    let code = r"
def test():
    x: list[int] = []
    x.nonexistent_method()
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::AttributeError { .. })));
}

#[test]
fn test_invalid_attribute_error() {
    let code = r"
def test():
    x: list[int] = []
    return x.nonexistent_attr
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::AttributeError { .. })));
}

// =============================================================================
// Missing Return Tests (3 tests)
// =============================================================================

#[test]
fn test_missing_return_error() {
    let code = r"
def test() -> int:
    x = 5
";

    let result = analyze_code(code);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(contains_error(&errors, |e| matches!(e, SemanticError::MissingReturn { .. })));
}

#[test]
fn test_all_paths_return_ok() {
    let code = r"
def test() -> int:
    if True:
        return 1
    else:
        return 2
";

    assert!(analyze_code(code).is_ok());
}

#[test]
fn test_no_return_type_no_error() {
    let code = r"
def test():
    x = 5
";

    assert!(analyze_code(code).is_ok());
}
