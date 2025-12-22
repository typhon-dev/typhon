//! Function Representation
//!
//! This module defines the function representation in MIR, including parameters,
//! local variables, and captured variables for closures.

use typhon_source::types::Span;

use crate::block::BasicBlock;
use crate::instr::LocalID;
use crate::types::MIRType;

/// MIR function
#[derive(Debug, Clone)]
pub struct MIRFunction {
    /// Function name
    pub name: String,
    /// Parameters
    pub params: Vec<MIRParam>,
    /// Return type
    pub return_type: MIRType,
    /// Local variables
    pub locals: Vec<MIRLocal>,
    /// Basic blocks (entry block is always `blocks[0]`)
    pub blocks: Vec<BasicBlock>,
    /// Captured variables (if this is a closure)
    pub captures: Vec<MIRCapture>,
    /// Source location
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct MIRParam {
    pub name: String,
    pub ty: MIRType,
    pub local: LocalID,
}

/// Local variable
#[derive(Debug, Clone)]
pub struct MIRLocal {
    pub name: Option<String>,
    pub ty: MIRType,
    pub mutable: bool,
}

/// Captured variable (for closures)
#[derive(Debug, Clone)]
pub struct MIRCapture {
    pub name: String,
    pub ty: MIRType,
    pub source_local: LocalID,
}
