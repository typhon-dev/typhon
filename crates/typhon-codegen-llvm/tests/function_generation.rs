//! Function generation tests for LLVM codegen.
//!
//! Tests complete function generation including signature translation,
//! parameter setup, local allocation, and function verification.

use std::f64;

use inkwell::context::Context;
use typhon_codegen_llvm::context::CodegenContext;
use typhon_codegen_llvm::functions::generate_function;
use typhon_codegen_llvm::object_layout::declare_object_types;
use typhon_codegen_llvm::runtime::declare_runtime_functions;
use typhon_mir::block::BasicBlock;
use typhon_mir::function::{MIRFunction, MIRLocal, MIRParam};
use typhon_mir::instr::{
    BasicBlockID,
    BinOpKind,
    LocalID,
    MIRConst,
    MIRInstr,
    Terminator,
    ValueID,
};
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

/// Helper to create a test context with object types and runtime functions declared.
fn create_test_context(context: &Context) -> CodegenContext<'_> {
    let module = context.create_module("test");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(context, module, builder);

    declare_object_types(&mut ctx).expect("Failed to declare object types");
    declare_runtime_functions(&mut ctx).expect("Failed to declare runtime functions");

    ctx
}

/// Helper to create a basic MIR function for testing.
fn create_test_function(
    name: &str,
    params: Vec<(&str, MIRType)>,
    return_type: MIRType,
    locals: Vec<MIRLocal>,
    blocks: Vec<BasicBlock>,
) -> MIRFunction {
    let mir_params = params
        .into_iter()
        .enumerate()
        .map(|(idx, (name, ty))| MIRParam {
            name: name.to_string(),
            ty,
            local: LocalID(idx as u32),
        })
        .collect();

    MIRFunction {
        name: name.to_string(),
        params: mir_params,
        return_type,
        locals,
        blocks,
        captures: vec![],
        span: Span::default(),
    }
}

#[test]
fn test_simple_function_no_params() {
    // Test: def get_answer() -> Int: return 42
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let mut block = create_block(BasicBlockID(0));

    block.instrs.push(MIRInstr::Const(MIRConst::Int(42)));
    block.terminator = Terminator::Return(Some(ValueID(0)));

    let mir_func = create_test_function("get_answer", vec![], MIRType::Int, vec![], vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert_eq!(llvm_func.get_name().to_str().unwrap(), "get_answer");
    assert_eq!(llvm_func.count_params(), 0);
    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_single_parameter() {
    // Test: def identity(x: Int) -> Int: return x
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: false }];
    let mut block = create_block(BasicBlockID(0));

    block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    block.terminator = Terminator::Return(Some(ValueID(0)));

    let mir_func = create_test_function(
        "identity",
        vec![("x", MIRType::Int)],
        MIRType::Int,
        locals,
        vec![block],
    );
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert_eq!(llvm_func.count_params(), 1);
    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_multiple_parameters() {
    // Test: def add(a: Int, b: Int) -> Int: return a + b
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![
        MIRLocal { name: Some("a".to_string()), ty: MIRType::Int, mutable: false },
        MIRLocal { name: Some("b".to_string()), ty: MIRType::Int, mutable: false },
    ];
    let mut block = create_block(BasicBlockID(0));

    block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    block.instrs.push(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });
    block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: ValueID(0),
        rhs: ValueID(1),
        ty: MIRType::Int,
    });
    block.terminator = Terminator::Return(Some(ValueID(2)));

    let mir_func = create_test_function(
        "add",
        vec![("a", MIRType::Int), ("b", MIRType::Int)],
        MIRType::Int,
        locals,
        vec![block],
    );
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert_eq!(llvm_func.count_params(), 2);
    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_void_return() {
    // Test: def do_nothing() -> Void: return
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let mut block = create_block(BasicBlockID(0));

    block.terminator = Terminator::Return(None);

    let mir_func = create_test_function("do_nothing", vec![], MIRType::Void, vec![], vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.get_type().get_return_type().is_none(), "Expected void return type");
    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_float_return() {
    // Test: def get_pi() -> Float: return 3.14159
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let mut block = create_block(BasicBlockID(0));

    block.instrs.push(MIRInstr::Const(MIRConst::Float(f64::consts::PI)));
    block.terminator = Terminator::Return(Some(ValueID(0)));

    let mir_func = create_test_function("get_pi", vec![], MIRType::Float, vec![], vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_local_variable() {
    // Test: def compute() -> Int:
    //           x = 10
    //           return x
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: true }];
    let mut block = create_block(BasicBlockID(0));

    block.instrs.push(MIRInstr::Const(MIRConst::Int(10)));
    block.instrs.push(MIRInstr::Store { local: LocalID(0), value: ValueID(0) });
    block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    block.terminator = Terminator::Return(Some(ValueID(1)));

    let mir_func = create_test_function("compute", vec![], MIRType::Int, locals, vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_multiple_locals() {
    // Test: def calculate() -> Int:
    //           x = 5
    //           y = 10
    //           return x + y
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![
        MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: true },
        MIRLocal { name: Some("y".to_string()), ty: MIRType::Int, mutable: true },
    ];
    let mut block = create_block(BasicBlockID(0));

    // x = 5
    block.instrs.push(MIRInstr::Const(MIRConst::Int(5)));
    block.instrs.push(MIRInstr::Store { local: LocalID(0), value: ValueID(0) });
    // y = 10
    block.instrs.push(MIRInstr::Const(MIRConst::Int(10)));
    block.instrs.push(MIRInstr::Store { local: LocalID(1), value: ValueID(1) });
    // load x and y
    block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    block.instrs.push(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });
    // x + y
    block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: ValueID(2),
        rhs: ValueID(3),
        ty: MIRType::Int,
    });
    block.terminator = Terminator::Return(Some(ValueID(4)));

    let mir_func = create_test_function("calculate", vec![], MIRType::Int, locals, vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_conditional_branch() {
    // Test: def abs(x: Int) -> Int:
    //           if x < 0:
    //               return -x
    //           return x
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: false }];

    // Entry block
    let mut entry = create_block(BasicBlockID(0));

    entry.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    entry.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    entry.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Lt,
        lhs: ValueID(0),
        rhs: ValueID(1),
        ty: MIRType::Bool,
    });
    entry.terminator = Terminator::CondBranch {
        condition: ValueID(2),
        then_block: BasicBlockID(1),
        else_block: BasicBlockID(2),
    };

    // Then block (return -x)
    let mut then_block = create_block(BasicBlockID(1));

    then_block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    then_block.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    then_block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Sub,
        lhs: ValueID(4),
        rhs: ValueID(3),
        ty: MIRType::Int,
    });
    then_block.terminator = Terminator::Return(Some(ValueID(5)));

    // Else block (return x)
    let mut else_block = create_block(BasicBlockID(2));

    else_block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    else_block.terminator = Terminator::Return(Some(ValueID(6)));

    let mir_func = create_test_function(
        "abs",
        vec![("x", MIRType::Int)],
        MIRType::Int,
        locals,
        vec![entry, then_block, else_block],
    );
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert_eq!(llvm_func.count_basic_blocks(), 3); // 3 blocks: entry, then_block, else_block
    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_with_unconditional_branch() {
    // Test: def simple_branch() -> Int:
    //           goto next
    //       next:
    //           return 42
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let mut entry = create_block(BasicBlockID(0));

    entry.terminator = Terminator::Branch(BasicBlockID(1));

    let mut next = create_block(BasicBlockID(1));

    next.instrs.push(MIRInstr::Const(MIRConst::Int(42)));
    next.terminator = Terminator::Return(Some(ValueID(0)));

    let mir_func =
        create_test_function("simple_branch", vec![], MIRType::Int, vec![], vec![entry, next]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_call_no_args() {
    // Test: def caller() -> Int:
    //           f = get_function()
    //           return call(f, [])
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![MIRLocal {
        name: Some("f".to_string()),
        ty: MIRType::Function { params: vec![], return_type: Box::new(MIRType::Int) },
        mutable: false,
    }];
    let mut block = create_block(BasicBlockID(0));

    // Get function object (simulated with a constant for testing)
    block.instrs.push(MIRInstr::Const(MIRConst::None)); // Placeholder for function
    block.instrs.push(MIRInstr::Store { local: LocalID(0), value: ValueID(0) });
    // Load function
    block.instrs.push(MIRInstr::Load {
        local: LocalID(0),
        ty: MIRType::Function { params: vec![], return_type: Box::new(MIRType::Int) },
    });
    // Call with no args
    block.instrs.push(MIRInstr::Call { callee: ValueID(1), args: vec![], ty: MIRType::Int });
    block.terminator = Terminator::Return(Some(ValueID(2)));

    let mir_func = create_test_function("caller", vec![], MIRType::Int, locals, vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_call_with_args() {
    // Test: def caller() -> Int:
    //           f = get_function()
    //           x = 10
    //           y = 20
    //           return call(f, [x, y])
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![
        MIRLocal {
            name: Some("f".to_string()),
            ty: MIRType::Function {
                params: vec![MIRType::Int, MIRType::Int],
                return_type: Box::new(MIRType::Int),
            },
            mutable: false,
        },
        MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: false },
        MIRLocal { name: Some("y".to_string()), ty: MIRType::Int, mutable: false },
    ];

    let mut block = create_block(BasicBlockID(0));

    // Get function
    block.instrs.push(MIRInstr::Const(MIRConst::None));
    block.instrs.push(MIRInstr::Store { local: LocalID(0), value: ValueID(0) });
    // x = 10
    block.instrs.push(MIRInstr::Const(MIRConst::Int(10)));
    block.instrs.push(MIRInstr::Store { local: LocalID(1), value: ValueID(1) });
    // y = 20
    block.instrs.push(MIRInstr::Const(MIRConst::Int(20)));
    block.instrs.push(MIRInstr::Store { local: LocalID(2), value: ValueID(2) });
    // Load all
    block.instrs.push(MIRInstr::Load {
        local: LocalID(0),
        ty: MIRType::Function {
            params: vec![MIRType::Int, MIRType::Int],
            return_type: Box::new(MIRType::Int),
        },
    });
    block.instrs.push(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });
    block.instrs.push(MIRInstr::Load { local: LocalID(2), ty: MIRType::Int });
    // Call with args
    block.instrs.push(MIRInstr::Call {
        callee: ValueID(3),
        args: vec![ValueID(4), ValueID(5)],
        ty: MIRType::Int,
    });
    block.terminator = Terminator::Return(Some(ValueID(6)));

    let mir_func = create_test_function("caller", vec![], MIRType::Int, locals, vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_complete_arithmetic_function() {
    // Test: def complex_calc(a: Int, b: Int, c: Int) -> Int:
    //           x = a + b
    //           y = x * c
    //           z = y - a
    //           return z
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![
        MIRLocal { name: Some("a".to_string()), ty: MIRType::Int, mutable: false },
        MIRLocal { name: Some("b".to_string()), ty: MIRType::Int, mutable: false },
        MIRLocal { name: Some("c".to_string()), ty: MIRType::Int, mutable: false },
        MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: true },
        MIRLocal { name: Some("y".to_string()), ty: MIRType::Int, mutable: true },
        MIRLocal { name: Some("z".to_string()), ty: MIRType::Int, mutable: true },
    ];
    let mut block = create_block(BasicBlockID(0));

    // x = a + b
    block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    block.instrs.push(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });
    block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: ValueID(0),
        rhs: ValueID(1),
        ty: MIRType::Int,
    });
    block.instrs.push(MIRInstr::Store { local: LocalID(3), value: ValueID(2) });
    // y = x * c
    block.instrs.push(MIRInstr::Load { local: LocalID(3), ty: MIRType::Int });
    block.instrs.push(MIRInstr::Load { local: LocalID(2), ty: MIRType::Int });
    block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Mul,
        lhs: ValueID(3),
        rhs: ValueID(4),
        ty: MIRType::Int,
    });
    block.instrs.push(MIRInstr::Store { local: LocalID(4), value: ValueID(5) });
    // z = y - a
    block.instrs.push(MIRInstr::Load { local: LocalID(4), ty: MIRType::Int });
    block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Sub,
        lhs: ValueID(6),
        rhs: ValueID(7),
        ty: MIRType::Int,
    });
    block.instrs.push(MIRInstr::Store { local: LocalID(5), value: ValueID(8) });
    // return z
    block.instrs.push(MIRInstr::Load { local: LocalID(5), ty: MIRType::Int });
    block.terminator = Terminator::Return(Some(ValueID(9)));

    let mir_func = create_test_function(
        "complex_calc",
        vec![("a", MIRType::Int), ("b", MIRType::Int), ("c", MIRType::Int)],
        MIRType::Int,
        locals,
        vec![block],
    );
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert_eq!(llvm_func.count_params(), 3);
    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_complete_function_with_comparison() {
    // Test: def max(a: Int, b: Int) -> Int:
    //           if a > b:
    //               return a
    //           return b
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![
        MIRLocal { name: Some("a".to_string()), ty: MIRType::Int, mutable: false },
        MIRLocal { name: Some("b".to_string()), ty: MIRType::Int, mutable: false },
    ];

    // Entry: compare a > b
    let mut entry = create_block(BasicBlockID(0));

    entry.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    entry.instrs.push(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });
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

    // Then: return a
    let mut then_block = create_block(BasicBlockID(1));

    then_block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    then_block.terminator = Terminator::Return(Some(ValueID(3)));

    // Else: return b
    let mut else_block = create_block(BasicBlockID(2));

    else_block.instrs.push(MIRInstr::Load { local: LocalID(1), ty: MIRType::Int });
    else_block.terminator = Terminator::Return(Some(ValueID(4)));

    let mir_func = create_test_function(
        "max",
        vec![("a", MIRType::Int), ("b", MIRType::Int)],
        MIRType::Int,
        locals,
        vec![entry, then_block, else_block],
    );
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_complete_function_with_loop_simulation() {
    // Test function that simulates a simple loop with branching
    // def count() -> Int:
    //     i = 0
    //     goto loop
    // loop:
    //     i = i + 1
    //     if i < 10:
    //         goto loop
    //     return i
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let locals = vec![MIRLocal { name: Some("i".to_string()), ty: MIRType::Int, mutable: true }];

    // Entry: i = 0, goto loop
    let mut entry = create_block(BasicBlockID(0));

    entry.instrs.push(MIRInstr::Const(MIRConst::Int(0)));
    entry.instrs.push(MIRInstr::Store { local: LocalID(0), value: ValueID(0) });
    entry.terminator = Terminator::Branch(BasicBlockID(1));

    // Loop: i = i + 1, check condition
    let mut loop_block = create_block(BasicBlockID(1));

    loop_block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    loop_block.instrs.push(MIRInstr::Const(MIRConst::Int(1)));
    loop_block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Add,
        lhs: ValueID(1),
        rhs: ValueID(2),
        ty: MIRType::Int,
    });
    loop_block.instrs.push(MIRInstr::Store { local: LocalID(0), value: ValueID(3) });
    loop_block.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    loop_block.instrs.push(MIRInstr::Const(MIRConst::Int(10)));
    loop_block.instrs.push(MIRInstr::BinOp {
        op: BinOpKind::Lt,
        lhs: ValueID(4),
        rhs: ValueID(5),
        ty: MIRType::Bool,
    });
    loop_block.terminator = Terminator::CondBranch {
        condition: ValueID(6),
        then_block: BasicBlockID(1), // loop back
        else_block: BasicBlockID(2), // exit
    };

    // Exit: return i
    let mut exit = create_block(BasicBlockID(2));

    exit.instrs.push(MIRInstr::Load { local: LocalID(0), ty: MIRType::Int });
    exit.terminator = Terminator::Return(Some(ValueID(7)));

    let mir_func =
        create_test_function("count", vec![], MIRType::Int, locals, vec![entry, loop_block, exit]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    assert!(llvm_func.verify(true), "Function verification failed");
}

#[test]
fn test_function_verification_passes() {
    // Test that a well-formed function passes LLVM verification
    let context = Context::create();
    let mut ctx = create_test_context(&context);
    let mut block = create_block(BasicBlockID(0));
    block.instrs.push(MIRInstr::Const(MIRConst::Int(100)));
    block.terminator = Terminator::Return(Some(ValueID(0)));

    let mir_func = create_test_function("test_verify", vec![], MIRType::Int, vec![], vec![block]);
    let llvm_func = generate_function(&mut ctx, &mir_func).expect("Function generation failed");

    // Explicit verification test
    assert!(llvm_func.verify(true), "Function should pass LLVM verification");

    // Also verify the module
    assert!(ctx.module.verify().is_ok(), "Module should also pass verification");
}
