//! Error types for LLVM code generation.

use inkwell::support::LLVMString;
use thiserror::Error;
use typhon_mir::instr::{BasicBlockID, LocalID, ValueID};
use typhon_mir::types::MIRType;

/// Result type for LLVM code generation operations.
pub type CodegenResult<T> = Result<T, CodegenError>;

/// Errors that can occur during LLVM code generation.
#[derive(Debug, Error)]
pub enum CodegenError {
    /// Basic block not found.
    #[error("Block not found: {0:?}")]
    BlockNotFound(BasicBlockID),
    /// Code generation failed.
    #[error("Code generation failed: {0}")]
    CodegenError(String),
    /// Function not found.
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    /// Inkwell error.
    #[error("LLVM error: {0}")]
    InkwellError(String),
    /// Instruction translation failed.
    #[error("Instruction translation failed: {0}")]
    InstructionTranslationError(String),
    /// Invalid function.
    #[error("Invalid function: {0}")]
    InvalidFunction(String),
    /// Invalid module.
    #[error("Invalid module: {0}")]
    InvalidModule(String),
    /// I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Link error with details.
    #[error("Link error: {0}")]
    LinkError(String),
    /// Linking failed.
    #[error("Linking failed")]
    LinkFailed,
    /// Linking error with details.
    #[error("Linking error: {0}")]
    LinkingError(String),
    /// LLVM initialization failed.
    #[error("LLVM initialization failed: {0}")]
    LLVMInitError(String),
    /// Local variable not found.
    #[error("Local not found: {0:?}")]
    LocalNotFound(LocalID),
    /// Module initialization function not found.
    #[error("Module initialization function '__module_init__' not found")]
    ModuleInitNotFound,
    /// Object file generation failed.
    #[error("Object file generation failed: {0}")]
    ObjectFileError(String),
    /// Parameter not found.
    #[error("Parameter not found at index {0}")]
    ParameterNotFound(usize),
    /// Runtime function not found.
    #[error("Runtime function not found: {0}")]
    RuntimeFunctionNotFound(String),
    /// Struct type not found.
    #[error("Struct type not found: {0}")]
    StructTypeNotFound(String),
    /// Target error.
    #[error("Target error: {0}")]
    TargetError(String),
    /// Target initialization failed.
    #[error("Target initialization failed: {0}")]
    TargetInitError(String),
    /// Target machine creation failed.
    #[error("Target machine creation failed")]
    TargetMachineCreationFailed,
    /// Type translation failed.
    #[error("Type translation failed for {ty:?}: {reason}")]
    TypeTranslationError {
        /// The MIR type that failed to translate.
        ty: MIRType,
        /// The reason for the failure.
        reason: String,
    },
    /// Unsupported type encountered.
    #[error("Unsupported type")]
    UnsupportedType,
    /// Value not found.
    #[error("Value not found: {0:?}")]
    ValueNotFound(ValueID),
    /// Void type cannot be used as a value type.
    #[error("Void type cannot be used as a value type")]
    VoidTypeNotAllowed,
}

impl From<LLVMString> for CodegenError {
    fn from(err: LLVMString) -> Self { Self::InkwellError(err.to_string()) }
}
