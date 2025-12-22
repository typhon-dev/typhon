//! Tests for SSA construction pass.

use typhon_mir_optimizer::passes::construct_ssa;

mod fixtures;

use fixtures::{create_empty_function, create_test_function};

#[test]
fn test_ssa_construction_simple() {
    let mut func = create_test_function("simple");

    // SSA construction should succeed on a simple function
    let result = construct_ssa(&mut func);

    assert!(result.is_ok(), "SSA construction should succeed: {result:?}");
}

#[test]
fn test_ssa_construction_empty() {
    let mut func = create_empty_function("empty");

    // SSA construction should succeed on an empty function
    let result = construct_ssa(&mut func);

    assert!(result.is_ok(), "SSA construction should succeed on empty function: {result:?}");
}
