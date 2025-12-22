//! Tests for escape analysis.

use typhon_mir::instr::{LocalID, MIRInstr, Terminator, ValueID};
use typhon_mir::types::MIRType;
use typhon_mir_optimizer::passes::{EscapeAnalysis, EscapeState};

mod fixtures;

use fixtures::{create_function_with_instrs_and_terminator, create_module_with_function};

#[test]
fn test_no_escape_local() {
    // Create: object allocated and used only locally
    let instrs = vec![
        // Allocate object (ValueID 0)
        MIRInstr::AllocObject { type_id: 0, size: 16 },
        // Store to local (doesn't cause escape)
        MIRInstr::Store { local: LocalID(0), value: ValueID(0) },
        // Load from local (ValueID 1)
        MIRInstr::Load { local: LocalID(0), ty: MIRType::Object { type_id: Some(0) } },
    ];

    let func = create_function_with_instrs_and_terminator(
        "test_no_escape",
        instrs,
        Terminator::Return(None),
    );
    let module = create_module_with_function(func);

    let mut analysis = EscapeAnalysis::new();
    analysis.analyze_module(&module).expect("escape analysis should succeed");

    // ValueID(0) is the allocated object - should not escape
    assert_eq!(
        analysis.get_escape_state(ValueID(0)),
        Some(EscapeState::NoEscape),
        "locally-used object should not escape"
    );
    assert!(
        analysis.can_stack_allocate(ValueID(0)),
        "locally-used object should be stack-allocatable"
    );
}

#[test]
fn test_escape_via_return() {
    // Create: object escapes through return value
    let instrs = vec![
        // Allocate object (ValueID 0)
        MIRInstr::AllocObject { type_id: 0, size: 16 },
    ];

    let func = create_function_with_instrs_and_terminator(
        "test_return_escape",
        instrs,
        Terminator::Return(Some(ValueID(0))),
    );
    let module = create_module_with_function(func);

    let mut analysis = EscapeAnalysis::new();
    analysis.analyze_module(&module).expect("escape analysis should succeed");

    // ValueID(0) is returned - should have ReturnEscape
    assert_eq!(
        analysis.get_escape_state(ValueID(0)),
        Some(EscapeState::ReturnEscape),
        "returned object should have ReturnEscape"
    );
    assert!(
        !analysis.can_stack_allocate(ValueID(0)),
        "returned object should not be stack-allocatable"
    );
}

#[test]
fn test_escape_via_global() {
    // Create: object stored in global variable
    let instrs = vec![
        // Allocate object (ValueID 0)
        MIRInstr::AllocObject { type_id: 0, size: 16 },
        // Store to global (causes escape)
        MIRInstr::StoreGlobal { name: "global_var".to_string(), value: ValueID(0) },
    ];

    let func = create_function_with_instrs_and_terminator(
        "test_global_escape",
        instrs,
        Terminator::Return(None),
    );
    let module = create_module_with_function(func);

    let mut analysis = EscapeAnalysis::new();
    analysis.analyze_module(&module).expect("escape analysis should succeed");

    // ValueID(0) is stored in global - should have GlobalEscape
    assert_eq!(
        analysis.get_escape_state(ValueID(0)),
        Some(EscapeState::GlobalEscape),
        "globally-stored object should have GlobalEscape"
    );
    assert!(
        !analysis.can_stack_allocate(ValueID(0)),
        "globally-stored object should not be stack-allocatable"
    );
}

#[test]
fn test_escape_via_call() {
    // Create: object passed to external function
    let instrs = vec![
        // Allocate object (ValueID 0)
        MIRInstr::AllocObject { type_id: 0, size: 16 },
        // Load function to call (ValueID 1)
        MIRInstr::LoadGlobal {
            name: "external_func".to_string(),
            ty: MIRType::Function { params: vec![], return_type: Box::new(MIRType::Void) },
        },
        // Call function with object as argument
        MIRInstr::Call { callee: ValueID(1), args: vec![ValueID(0)], ty: MIRType::Void },
    ];

    let func = create_function_with_instrs_and_terminator(
        "test_call_escape",
        instrs,
        Terminator::Return(None),
    );
    let module = create_module_with_function(func);

    let mut analysis = EscapeAnalysis::new();
    analysis.analyze_module(&module).expect("escape analysis should succeed");

    // ValueID(0) is passed to function - should have ArgEscape
    let escape_state = analysis.get_escape_state(ValueID(0));
    assert!(
        matches!(escape_state, Some(EscapeState::ArgEscape | EscapeState::GlobalEscape)),
        "object passed to function should have ArgEscape or GlobalEscape, got {escape_state:?}"
    );
    assert!(
        !analysis.can_stack_allocate(ValueID(0)),
        "object passed to function should not be stack-allocatable"
    );
}

#[test]
fn test_no_escape_dead_store() {
    // Create: object stored but never used
    let instrs = vec![
        // Allocate object (ValueID 0)
        MIRInstr::AllocObject { type_id: 0, size: 16 },
        // Store to local but never read (dead store)
        MIRInstr::Store { local: LocalID(0), value: ValueID(0) },
        // Do something else unrelated
        MIRInstr::AllocObject { type_id: 1, size: 8 },
    ];

    let func = create_function_with_instrs_and_terminator(
        "test_dead_store",
        instrs,
        Terminator::Return(None),
    );
    let module = create_module_with_function(func);

    let mut analysis = EscapeAnalysis::new();
    analysis.analyze_module(&module).expect("escape analysis should succeed");

    // ValueID(0) is stored but never used - should not escape
    assert_eq!(
        analysis.get_escape_state(ValueID(0)),
        Some(EscapeState::NoEscape),
        "dead-stored object should not escape"
    );
    assert!(
        analysis.can_stack_allocate(ValueID(0)),
        "dead-stored object should be stack-allocatable"
    );
}
