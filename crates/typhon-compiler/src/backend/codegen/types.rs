use inkwell::values::{BasicValueEnum, FunctionValue};

use crate::backend::error::{CodeGenError, CodeGenResult};

/// Represents the result of code generation for an AST node.
#[derive(Debug)]
pub enum CodeGenValue {
    /// A basic LLVM value (integer, float, pointer, etc.).
    Basic(BasicValueEnum),
    /// A function value.
    Function(FunctionValue),
    /// No value (void).
    Void,
}

impl CodeGenValue {
    /// Convert to a basic value, returns an error if this is not a basic value.
    pub fn as_basic_value_enum(&self) -> CodeGenResult<BasicValueEnum> {
        match self {
            CodeGenValue::Basic(value) => Ok(*value),
            _ => Err(CodeGenError::code_gen_error("Expected a basic value".to_string(), None)),
        }
    }

    /// Convert to a function value, returns an error if this is not a function value.
    pub fn as_function_value(&self) -> CodeGenResult<FunctionValue> {
        match self {
            CodeGenValue::Function(value) => Ok(*value),
            _ => Err(CodeGenError::code_gen_error("Expected a function value".to_string(), None)),
        }
    }

    /// Create a new basic value from the given value.
    /// This ensures we're not holding references to temporary values.
    pub fn new_basic(value: BasicValueEnum) -> CodeGenValue {
        CodeGenValue::Basic(value)
    }

    /// Create a new function value from the given value.
    /// This ensures we're not holding references to temporary values.
    pub fn new_function(value: FunctionValue) -> CodeGenValue {
        CodeGenValue::Function(value)
    }
}
