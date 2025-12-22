//! Basic Blocks and Control Flow
//!
//! This module defines basic blocks and control flow structures for MIR.
//! Basic blocks are single-entry, single-exit code sequences with explicit control flow.

use crate::instr::{BasicBlockID, MIRInstr, Terminator};
use crate::types::TypeID;

/// A basic block in the MIR CFG
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Unique identifier
    pub id: BasicBlockID,
    /// Instructions in this block
    pub instrs: Vec<MIRInstr>,
    /// Terminator instruction
    pub terminator: Terminator,
    /// Exception landing pad (if this block can throw)
    pub landing_pad: Option<LandingPad>,
    /// Predecessors in the CFG
    pub predecessors: Vec<BasicBlockID>,
    /// Successors in the CFG
    pub successors: Vec<BasicBlockID>,
}

/// Exception handler (catch clause)
#[derive(Debug, Clone, Copy)]
pub struct ExceptionHandler {
    /// Exception type to catch (None = catch all)
    pub exception_type: Option<TypeID>,
    /// Handler block
    pub handler_block: BasicBlockID,
}

/// Exception landing pad
#[derive(Debug, Clone)]
pub struct LandingPad {
    /// Local variable to store caught exception
    pub exception_var: crate::instr::LocalID,
    /// Exception handlers (type, handler block)
    pub handlers: Vec<ExceptionHandler>,
    /// Cleanup block (finally clause)
    pub cleanup: Option<BasicBlockID>,
}
