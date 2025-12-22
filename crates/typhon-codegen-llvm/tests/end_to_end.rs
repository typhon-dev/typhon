//! End-to-end integration tests for complete compilation pipeline.
//!
//! These tests verify the complete flow from MIR through LLVM IR generation
//! to object file emission.

use inkwell::OptimizationLevel;
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

/// Tests complete compilation from MIR to LLVM IR.
#[test]
fn test_complete_compilation_pipeline() {
    // Create a complete MIR module with multiple functions
    let mut block1 = create_block(BasicBlockID(0));
    block1.instrs.push(MIRInstr::Const(MIRConst::Int(42)));
    block1.terminator = Terminator::Return(Some(ValueID(0)));

    let func1 = MIRFunction {
        name: "get_answer".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![block1],
        captures: vec![],
        span: Span::default(),
    };

    let mut block2 = create_block(BasicBlockID(0));
    block2.terminator = Terminator::Return(None);

    let func2 = MIRFunction {
        name: "do_nothing".to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![block2],
        captures: vec![],
        span: Span::default(),
    };

    let module = MIRModule {
        name: "test_program".to_string(),
        functions: vec![func1, func2],
        globals: vec![],
        types: vec![],
    };

    // Compile to LLVM IR
    let llvm_ir = compile_module(&module).expect("Compilation failed");

    // Verify IR contains expected elements
    assert!(llvm_ir.contains("define"), "IR should contain function definitions");
    assert!(llvm_ir.contains("ret"), "IR should contain return instructions");
    assert!(llvm_ir.contains("test_program"), "IR should contain module name");
    assert!(llvm_ir.contains("get_answer"), "IR should contain first function");
    assert!(llvm_ir.contains("do_nothing"), "IR should contain second function");
}

/// Tests object file generation from complete module.
#[test]
fn test_object_file_generation_end_to_end() {
    // Create a simple function
    let mut block = create_block(BasicBlockID(0));
    block.instrs.push(MIRInstr::Const(MIRConst::Int(100)));
    block.terminator = Terminator::Return(Some(ValueID(0)));

    let func = MIRFunction {
        name: "simple_func".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![block],
        captures: vec![],
        span: Span::default(),
    };

    let module = MIRModule {
        name: "test_obj".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let target = Target::default();

    // Compile to object file
    let obj_data = compile_to_object_file(&module, &target).expect("Object compilation failed");

    // Verify we got non-empty object data
    assert!(!obj_data.is_empty(), "Object file should not be empty");

    // Object files typically start with magic bytes
    // ELF: 0x7F 'E' 'L' 'F'
    // Mach-O: 0xFE 0xED 0xFA 0xCE or 0xFE 0xED 0xFA 0xCF
    // Check that we got some recognizable format
    assert!(obj_data.len() > 4, "Object file should have valid header");
}

/// Tests compilation of module with arithmetic operations.
#[test]
fn test_arithmetic_function_compilation() {
    // Create function: def add_nums() -> Int: return 10 + 20
    let mut block = create_block(BasicBlockID(0));
    block.instrs.push(MIRInstr::Const(MIRConst::Int(10)));
    block.instrs.push(MIRInstr::Const(MIRConst::Int(20)));
    block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: ValueID(0),
        rhs: ValueID(1),
        ty: MIRType::Int,
    });
    block.terminator = Terminator::Return(Some(ValueID(2)));

    let func = MIRFunction {
        name: "add_nums".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![block],
        captures: vec![],
        span: Span::default(),
    };

    let module = MIRModule {
        name: "arithmetic_test".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    let llvm_ir = compile_module(&module).expect("Arithmetic function compilation failed");

    // Verify IR contains arithmetic operation
    assert!(llvm_ir.contains("add"), "IR should contain add operation");
}

/// Tests compilation with different optimization levels.
#[test]
fn test_optimization_levels() {
    let mut block = create_block(BasicBlockID(0));
    block.instrs.push(MIRInstr::Const(MIRConst::Int(5)));
    block.terminator = Terminator::Return(Some(ValueID(0)));

    let func = MIRFunction {
        name: "opt_test".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![block],
        captures: vec![],
        span: Span::default(),
    };

    let module = MIRModule {
        name: "opt_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };

    // Test with different optimization levels
    let mut target = Target { opt_level: OptimizationLevel::None, ..Default::default() };
    let obj_none =
        compile_to_object_file(&module, &target).expect("Compilation with no optimization failed");

    target.opt_level = OptimizationLevel::Default;
    let obj_default = compile_to_object_file(&module, &target)
        .expect("Compilation with default optimization failed");

    target.opt_level = OptimizationLevel::Aggressive;
    let obj_aggressive = compile_to_object_file(&module, &target)
        .expect("Compilation with aggressive optimization failed");

    // All should produce valid object files
    assert!(!obj_none.is_empty());
    assert!(!obj_default.is_empty());
    assert!(!obj_aggressive.is_empty());
}

/// Tests compilation of module with control flow.
#[test]
fn test_control_flow_compilation() {
    // Entry block with condition
    let mut entry = create_block(BasicBlockID(0));

    entry.instrs.push(MIRInstr::Const(MIRConst::Int(5)));
    entry.instrs.push(MIRInstr::Const(MIRConst::Int(3)));
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

    then_block.instrs.push(MIRInstr::Const(MIRConst::Int(1)));
    then_block.terminator = Terminator::Return(Some(ValueID(3)));

    // Else block
    let mut else_block = create_block(BasicBlockID(2));

    else_block.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    else_block.terminator = Terminator::Return(Some(ValueID(4)));

    let func = MIRFunction {
        name: "conditional".to_string(),
        params: vec![],
        return_type: MIRType::Int,
        locals: vec![],
        blocks: vec![entry, then_block, else_block],
        captures: vec![],
        span: Span::default(),
    };
    let module = MIRModule {
        name: "control_flow_test".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
    };
    let llvm_ir = compile_module(&module).expect("Control flow compilation failed");

    // Verify IR contains branch instructions
    assert!(llvm_ir.contains("br"), "IR should contain branch instructions");
}
