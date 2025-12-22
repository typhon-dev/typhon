//! MIR builder helpers for tests.

use rustc_hash::FxHashMap;
use typhon_mir::block::BasicBlock;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BasicBlockID, MIRInstr, Terminator};
use typhon_mir::module::MIRModule;
use typhon_mir::types::MIRType;
use typhon_source::types::Span;

/// Helper to create a test function with given instructions.
pub fn create_function_with_instrs(name: &str, instrs: Vec<MIRInstr>) -> MIRFunction {
    MIRFunction {
        name: name.to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs,
            terminator: Terminator::Return(None),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    }
}

/// Helper to create a test function with given instructions and custom terminator.
pub fn create_function_with_instrs_and_terminator(
    name: &str,
    instrs: Vec<MIRInstr>,
    terminator: Terminator,
) -> MIRFunction {
    MIRFunction {
        name: name.to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs,
            terminator,
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    }
}

/// Helper to create a module from a single function.
pub fn create_module_with_function(func: MIRFunction) -> MIRModule {
    MIRModule {
        name: "test_module".to_string(),
        functions: vec![func],
        globals: vec![],
        types: vec![],
        value_names: FxHashMap::default(),
    }
}
