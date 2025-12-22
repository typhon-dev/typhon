//! Common test utilities for MIR optimizer tests.

use rstest::{fixture, rstest};
use typhon_mir::block::BasicBlock;
use typhon_mir::function::{MIRFunction, MIRLocal, MIRParam};
use typhon_mir::instr::{BasicBlockID, LocalID, Terminator};
use typhon_mir::types::MIRType;
use typhon_source::types::Span;

/// Helper function to create an empty test function.
pub fn create_empty_function(name: &str) -> MIRFunction {
    MIRFunction {
        name: name.to_string(),
        params: vec![],
        return_type: MIRType::Void,
        locals: vec![],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![],
            terminator: Terminator::Return(None),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    }
}

/// Helper function to create a simple test function.
pub fn create_test_function(name: &str) -> MIRFunction {
    MIRFunction {
        name: name.to_string(),
        params: vec![MIRParam { name: "x".to_string(), ty: MIRType::Int, local: LocalID(0) }],
        return_type: MIRType::Int,
        locals: vec![MIRLocal { name: Some("x".to_string()), ty: MIRType::Int, mutable: true }],
        blocks: vec![BasicBlock {
            id: BasicBlockID(0),
            instrs: vec![],
            terminator: Terminator::Return(None),
            landing_pad: None,
            predecessors: vec![],
            successors: vec![],
        }],
        captures: vec![],
        span: Span::default(),
    }
}

// pub struct TestData {
//     name: String,
//     id: usize,
// }

// #[fixture]
// fn name(name: String) -> String {
//     return name
// }

// #[fixture]
// fn id(id: usize) -> usize {
//     return id
// }

// #[fixture]
// fn test_data(name: String, id: usize) -> TestData {
//     return TestData { name, id }
// }

// #[rstest]
// fn test_with_data(test_data: TestData) { // <--- how to parametrize name and id?

// }

// #[fixture]
// fn user(#[default("Alice")] name: &str, #[default(22)] age: u8) -> User { User::new(name, age) }

// #[rstest]
// fn is_alice(user: User) { assert_eq!(user.name(), "Alice") }

// #[rstest]
// fn is_22(user: User) { assert_eq!(user.age(), 22) }

// #[rstest]
// fn is_bob(#[with("Bob")] user: User) { assert_eq!(user.name(), "Bob") }

// #[rstest]
// fn is_42(#[with("", 42)] user: User) { assert_eq!(user.age(), 42) }
