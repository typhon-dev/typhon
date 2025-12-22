//! Error types for MIR optimization.

use thiserror::Error;
use typhon_mir::instr::{BasicBlockID, ValueID};

/// Result type for optimizer operations.
pub type OptimizerResult<T> = Result<T, OptimizerError>;

/// Errors that can occur during MIR optimization.
#[derive(Debug, Error)]
pub enum OptimizerError {
    /// Analysis pass failed.
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
    /// Control flow graph is malformed.
    #[error("CFG malformed: {0}")]
    CFGMalformed(String),
    /// Invalid block reference.
    #[error("Invalid block reference: {0:?}")]
    InvalidBlock(BasicBlockID),
    /// Invalid value reference.
    #[error("Invalid value reference: {0:?}")]
    InvalidValue(ValueID),
    /// SSA property violated.
    #[error("SSA violation: {0}")]
    SSAViolation(String),
}
