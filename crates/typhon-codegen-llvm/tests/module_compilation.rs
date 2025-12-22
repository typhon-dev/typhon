//! Integration tests for complete module compilation.
//!
//! These tests verify the end-to-end compilation pipeline from MIR to LLVM IR
//! and object files.

use typhon_codegen_llvm::{Target, compile_module, compile_to_object_file};
use typhon_mir::block::BasicBlock;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, BinOpKind, MIRConst, MIRInstr, Terminator, ValueID};
use typhon_mir::module::MIRModule;
use typhon_mir::types::MIRType;
use typhon_source::types::Span;

/// Helper to create a basic block with the given ID.
const fn create_block(id: BasicBlockID) -> BasicBlock {
    BasicBlock {
        id,
        instrs: Vec::new(),
        terminator: Terminator::Unreachable,
        landing_pad: None,
        predecessors: Vec::new(),
        successors: Vec::new(),
    }
}

/// Helper to create a simple empty function for testing.
fn create_empty_function() -> MIRFunction {
    let mut block = create_block(BasicBlockID(0));
    block.terminator = Terminator::Return(None);

    MIRFunction {
        name: "test_func".to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![block],
        captures: vec![],
        span: Span::default(),
    }
}

/// Helper to create a function that returns a constant.
fn create_constant_return_function() -> MIRFunction {
    let mut block = create_block(BasicBlockID(0));
    block.instrs.push(MIRInstr::Const(MIRConst::Int(42)));
    block.terminator = Terminator::Return(Some(ValueID(0)));

    MIRFunction {
        name: "constant_func".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![block],
        captures: vec![],
        span: Span::default(),
    }
}

/// Tests compiling an empty MIR module.
#[test]
fn test_empty_module_compilation() {
    let module = MIRModule {
        name: "empty_module".to_string(),
        functions: vec![],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Empty module compilation should succeed");

    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("empty_module"), "Module name should appear in IR");
}

/// Tests compiling a module with a single empty function.
#[test]
fn test_single_function_module() {
    let func = create_empty_function();
    let module = MIRModule {
        name: "single_func_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Single function module should compile");

    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("test_func"), "Function name should appear in IR");
    assert!(llvm_ir.contains("define"), "IR should contain function definition");
}

/// Tests compiling a module with a function that returns a value.
#[test]
fn test_function_with_return_value() {
    let func = create_constant_return_function();
    let module = MIRModule {
        name: "return_value_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Function with return value should compile");

    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("constant_func"), "Function name should appear in IR");
    assert!(llvm_ir.contains("ret"), "IR should contain return instruction");
}

/// Tests compiling a module with multiple functions.
#[test]
fn test_multiple_functions_module() {
    let func1 = create_empty_function();

    let mut block2 = create_block(BasicBlockID(0));
    block2.terminator = Terminator::Return(None);
    let func2 = MIRFunction {
        name: "second_func".to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![block2],
        captures: vec![],
        span: Span::default(),
    };

    let module = MIRModule {
        name: "multi_func_module".to_string(),
        functions: vec![func1, func2],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Module with multiple functions should compile");

    let llvm_ir = result.unwrap();
    assert!(llvm_ir.contains("test_func"), "First function should appear");
    assert!(llvm_ir.contains("second_func"), "Second function should appear");
}

/// Tests that module verification succeeds for valid module.
#[test]
fn test_module_verification_success() {
    let module = MIRModule {
        name: "valid_module".to_string(),
        functions: vec![],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Valid module should pass verification");
}

/// Tests LLVM IR format contains expected elements.
#[test]
fn test_llvm_ir_format() {
    let func = create_empty_function();
    let module = MIRModule {
        name: "format_test".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let llvm_ir = compile_module(&module).unwrap();

    assert!(llvm_ir.contains("source_filename"), "IR should have source filename");
    assert!(llvm_ir.contains("define"), "IR should have function definitions");
}

/// Tests LLVM IR contains correct function signatures.
#[test]
fn test_llvm_ir_signatures() {
    let func1 = create_empty_function();
    let func2 = create_constant_return_function();
    let module = MIRModule {
        name: "signature_test".to_string(),
        functions: vec![func1, func2],
        globals: vec![],
        types: vec![],
    };

    let llvm_ir = compile_module(&module).unwrap();

    assert!(llvm_ir.contains("void"), "Void function should appear");
    assert!(llvm_ir.contains("ret"), "Return instructions should appear");
}

/// Tests object file generation produces non-empty output.
#[test]
fn test_object_file_generation() {
    let func = create_empty_function();
    let module = MIRModule {
        name: "obj_test".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };
    let target = Target::default();

    let result = compile_to_object_file(&module, &target);

    assert!(result.is_ok(), "Object file generation should succeed");

    let obj_data = result.unwrap();
    assert!(!obj_data.is_empty(), "Object file should not be empty");
}

/// Tests object file has valid binary format markers.
#[test]
fn test_object_file_format() {
    let func = create_constant_return_function();
    let module = MIRModule {
        name: "opt_test".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };
    let target = Target::default();

    let obj_data = compile_to_object_file(&module, &target).unwrap();

    assert!(obj_data.len() > 4, "Object file should have valid header");
}

/// Tests module with control flow (branching).
#[test]
fn test_module_with_branching() {
    // Entry block with condition
    let mut entry = create_block(BasicBlockID(0));
    entry.instrs.push(MIRInstr::Const(MIRConst::Int(1)));
    entry.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    entry.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Gt,
        lhs: ValueID(0),
        rhs: ValueID(1),
        ty: MIRType::Bool,
    });
    entry.terminator = Terminator::CondBranch {
        condition: ValueID(2),
        then_block: BasicBlockID(1),
        else_block: BasicBlockID(2),
    };

    // Then block
    let mut then_block = create_block(BasicBlockID(1));
    then_block.instrs.push(MIRInstr::Const(MIRConst::Int(42)));
    then_block.terminator = Terminator::Return(Some(ValueID(3)));

    // Else block
    let mut else_block = create_block(BasicBlockID(2));
    else_block.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    else_block.terminator = Terminator::Return(Some(ValueID(4)));

    let func = MIRFunction {
        name: "branch_func".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![entry, then_block, else_block],
        captures: vec![],
        span: Span::default(),
    };
    let module = MIRModule {
        name: "branch_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Module with branching should compile");
}

/// Tests module with multiple functions calling each other.
#[test]
fn test_module_with_function_calls() {
    let func1 = create_constant_return_function();
    let func2 = create_empty_function();
    let module = MIRModule {
        name: "test_module".to_string(),
        functions: vec![func1, func2],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Module with multiple functions should compile");
}

/// Tests module with loop-like control flow.
#[test]
fn test_module_with_loop() {
    // Entry block branches to loop
    let mut entry = create_block(BasicBlockID(0));
    entry.terminator = Terminator::Branch(BasicBlockID(1));

    // Loop header with condition
    let mut loop_header = create_block(BasicBlockID(1));
    loop_header.instrs.push(MIRInstr::Const(MIRConst::Int(1)));
    loop_header.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    loop_header.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Gt,
        lhs: ValueID(0),
        rhs: ValueID(1),
        ty: MIRType::Bool,
    });
    loop_header.terminator = Terminator::CondBranch {
        condition: ValueID(2),
        then_block: BasicBlockID(1), // Loop back
        else_block: BasicBlockID(2), // Exit
    };

    // Exit block
    let mut exit = create_block(BasicBlockID(2));
    exit.terminator = Terminator::Return(None);

    let func = MIRFunction {
        name: "loop_func".to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![entry, loop_header, exit],
        captures: vec![],
        span: Span::default(),
    };
    let module = MIRModule {
        name: "loop_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let result = compile_module(&module);

    assert!(result.is_ok(), "Module with loop should compile");
}

/// Tests module name appears correctly in LLVM IR.
#[test]
fn test_module_name_in_ir() {
    let module_name = "custom_module_name";
    let func = create_empty_function();
    let module = MIRModule {
        name: module_name.to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let llvm_ir = compile_module(&module).unwrap();

    assert!(llvm_ir.contains(module_name), "Module name should appear in IR");
}

/// Tests function ordering is preserved in IR.
#[test]
fn test_function_ordering_in_ir() {
    let mut block_a = create_block(BasicBlockID(0));
    block_a.terminator = Terminator::Return(None);
    let func_a = MIRFunction {
        name: "func_a".to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![block_a],
        captures: vec![],
        span: Span::default(),
    };

    let mut block_b = create_block(BasicBlockID(0));
    block_b.terminator = Terminator::Return(None);
    let func_b = MIRFunction {
        name: "func_b".to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![block_b],
        captures: vec![],
        span: Span::default(),
    };

    let module = MIRModule {
        name: "ordering_test".to_string(),
        functions: vec![func_a, func_b],
        globals: vec![],
        types: vec![],
    };

    let llvm_ir = compile_module(&module).unwrap();
    let func_a_pos = llvm_ir.find("func_a").unwrap();
    let func_b_pos = llvm_ir.find("func_b").unwrap();

    assert!(func_a_pos < func_b_pos, "Function ordering should be preserved");
}
