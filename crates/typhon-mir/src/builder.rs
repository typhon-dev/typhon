//! MIR Builder API
//!
//! This module provides an ergonomic builder API for constructing MIR functions.
//! The builder manages SSA value allocation, basic block creation, and instruction emission.

use typhon_source::types::Span;

use crate::block::BasicBlock;
use crate::function::{MIRCapture, MIRFunction, MIRLocal, MIRParam};
use crate::instr::{BasicBlockID, LocalID, MIRInstr, Terminator, ValueID};
use crate::types::MIRType;

/// Builder for constructing MIR functions
#[derive(Debug)]
pub struct FunctionBuilder {
    name: String,
    params: Vec<MIRParam>,
    return_type: MIRType,
    locals: Vec<MIRLocal>,
    blocks: Vec<BasicBlock>,
    captures: Vec<MIRCapture>,
    current_block: Option<BasicBlockID>,
    next_value_id: u32,
    next_local_id: u32,
    next_block_id: u32,
}

impl FunctionBuilder {
    /// Create a new function builder
    #[must_use]
    pub fn new(name: String, params: Vec<(String, MIRType)>, return_type: MIRType) -> Self {
        let mut builder = Self {
            name,
            params: Vec::new(),
            return_type,
            locals: Vec::new(),
            blocks: Vec::new(),
            captures: Vec::new(),
            current_block: None,
            next_value_id: 0,
            next_local_id: 0,
            next_block_id: 0,
        };

        // Create locals for parameters
        for (param_name, param_ty) in params {
            let local_id =
                builder.allocate_local(Some(param_name.clone()), param_ty.clone(), false);
            builder.params.push(MIRParam { name: param_name, ty: param_ty, local: local_id });
        }

        builder
    }

    /// Add an instruction to the current block and return its value ID
    ///
    /// # Panics
    ///
    /// Panics if no current block is set via [`switch_to_block`](Self::switch_to_block).
    pub fn add_instruction(&mut self, instr: MIRInstr) -> ValueID {
        let value_id = self.allocate_value();
        if let Some(block_id) = self.current_block {
            let block = self.get_block_mut(block_id);
            block.instrs.push(instr);
        } else {
            panic!("No current block set");
        }

        value_id
    }

    /// Allocate a new local variable
    pub fn allocate_local(&mut self, name: Option<String>, ty: MIRType, mutable: bool) -> LocalID {
        let id = LocalID(self.next_local_id);
        self.next_local_id += 1;
        self.locals.push(MIRLocal { name, ty, mutable });

        id
    }

    /// Allocate a new SSA value
    pub const fn allocate_value(&mut self) -> ValueID {
        let id = ValueID(self.next_value_id);
        self.next_value_id += 1;

        id
    }

    /// Build the final MIR function
    #[must_use]
    pub fn build(self) -> MIRFunction {
        MIRFunction {
            name: self.name,
            params: self.params,
            return_type: self.return_type,
            locals: self.locals,
            blocks: self.blocks,
            captures: self.captures,
            span: Span::new(0, 0),
        }
    }

    /// Create a new basic block
    pub fn create_block(&mut self) -> BasicBlockID {
        let id = BasicBlockID(self.next_block_id);
        self.next_block_id += 1;

        let block = BasicBlock {
            id,
            instrs: Vec::new(),
            terminator: Terminator::Unreachable,
            landing_pad: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        self.blocks.push(block);

        id
    }

    /// Check if the current block is terminated
    ///
    /// Returns `true` if the current block has a terminator other than `Unreachable`.
    #[must_use]
    pub fn is_current_block_terminated(&self) -> bool {
        if let Some(block_id) = self.current_block
            && let Some(block) = self.blocks.iter().find(|b| b.id == block_id)
        {
            return !matches!(block.terminator, Terminator::Unreachable);
        }

        false
    }

    /// Set the terminator for the current block
    ///
    /// # Panics
    ///
    /// Panics if no current block is set via [`switch_to_block`](Self::switch_to_block).
    pub fn set_terminator(&mut self, terminator: Terminator) {
        if let Some(block_id) = self.current_block {
            let block = self.get_block_mut(block_id);
            block.terminator = terminator;
        } else {
            panic!("No current block set");
        }
    }

    /// Set captured variables for this function (for closures)
    pub fn set_captures(&mut self, captures: Vec<MIRCapture>) { self.captures = captures; }

    /// Switch to building a specific block
    pub const fn switch_to_block(&mut self, block_id: BasicBlockID) {
        self.current_block = Some(block_id);
    }

    fn get_block_mut(&mut self, id: BasicBlockID) -> &mut BasicBlock {
        self.blocks.iter_mut().find(|b| b.id == id).expect("Block not found")
    }
}
