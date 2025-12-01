//! Compiler driver module.
//!
//! This module provides the main driver for the Typhon compiler, which coordinates
//! the various phases of compilation including parsing, type checking, and code generation.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};
use std::fs::read_to_string;
use std::io::Error as IOError;
use std::path::Path;
use std::sync::Arc;

use inkwell::module::Module;
use typhon_parser::error::ParseError;
use typhon_parser::parser::Parser;

use crate::backend::{CodeGenError, CodeGenerator, CompilerContext};
use crate::typesystem::{TypeChecker, TypeError, TypeErrorKind};

/// Configuration options for the compiler driver.
#[derive(Debug, Clone)]
pub struct DriverConfig {
    /// Optimization level for the generated code.
    pub optimization_level: OptimizationLevel,
    /// Whether to emit debug information.
    pub emit_debug_info: bool,
    /// Whether to verify the generated LLVM module.
    pub verify_module: bool,
    /// Whether to print the generated LLVM IR to stderr.
    pub print_ir: bool,
}

/// Optimization level for code generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    /// No optimizations.
    None,
    /// Basic optimizations.
    Basic,
    /// Standard optimizations.
    Default,
    /// Aggressive optimizations.
    Aggressive,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Default,
            emit_debug_info: false,
            verify_module: true,
            print_ir: false,
        }
    }
}

/// Error type for the compiler driver.
#[derive(Debug)]
pub enum DriverError {
    /// Error from the parser.
    ParseError(ParseError),
    /// Error from the type checker.
    TypeError(TypeError),
    /// Error from code generation.
    CodeGenError(CodeGenError),
    /// Error when reading from a file.
    IOError(std::io::Error),
    /// Error when creating LLVM context.
    LLVMSetupError(String),
}

impl From<ParseError> for DriverError {
    fn from(err: ParseError) -> Self {
        DriverError::ParseError(err)
    }
}

impl From<TypeError> for DriverError {
    fn from(err: TypeError) -> Self {
        DriverError::TypeError(err)
    }
}

impl From<CodeGenError> for DriverError {
    fn from(err: CodeGenError) -> Self {
        DriverError::CodeGenError(err)
    }
}

impl From<IOError> for DriverError {
    fn from(err: IOError) -> Self {
        DriverError::IOError(err)
    }
}

impl Display for DriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            DriverError::ParseError(err) => write!(f, "Parse error: {err}"),
            DriverError::TypeError(err) => write!(f, "Type error: {err}"),
            DriverError::CodeGenError(err) => write!(f, "Code generation error: {err}"),
            DriverError::IOError(err) => write!(f, "IO error: {err}"),
            DriverError::LLVMSetupError(msg) => write!(f, "LLVM setup error: {msg}"),
        }
    }
}

impl Error for DriverError {}

/// Result type for the compiler driver.
pub type DriverResult<T> = Result<T, DriverError>;

/// Compiler driver responsible for coordinating the compilation pipeline.
pub struct Driver {
    /// Configuration options for the compiler.
    config: DriverConfig,
    /// Context for code generation.
    context: Arc<CompilerContext>,
}

impl Driver {
    /// Create a new compiler driver with default configuration.
    pub fn new(context: Arc<CompilerContext>, filename: &str) -> Self {
        Self { config: DriverConfig::default(), context }
    }

    /// Create a new compiler driver with the given configuration.
    pub fn with_config(mut self, config: DriverConfig) -> Self {
        self.config = config;
        self
    }

    /// Compile a source file to LLVM IR.
    pub fn compile_file(&mut self, path: &Path) -> DriverResult<String> {
        // Read the file content
        let source = read_to_string(path)?;
        let filename = path.file_name().and_then(|name| name.to_str()).unwrap_or("unknown");

        // Compile the source string
        self.compile_string(&source, filename)
    }

    /// Compile a source string to LLVM IR.
    pub fn compile_string(&mut self, source: &str, filename: &str) -> DriverResult<String> {
        // Clone Arc to share ownership
        let context = self.context.clone();

        // Run the pipeline with a borrowed reference
        self.run_pipeline(source, filename)?;

        // Get IR string
        let ir_string = context.llvm_module().to_string();

        if self.config.print_ir {
            eprintln!("{ir_string}");
        }

        Ok(ir_string)
    }

    /// Run all compiler phases on the given source.
    fn run_pipeline(&self, source: &str, filename: &str) -> DriverResult<Module> {
        let module = self.context.llvm_context().create_module(filename);

        // 1. Parse the source code to AST
        let mut parser = Parser::new(source);
        let ast = parser.parse()?;

        // 2. Run the type checker
        let mut type_checker = TypeChecker::new();
        let typed_ast = type_checker.check_module(&ast)?;

        if type_checker.has_errors() {
            return Err(DriverError::TypeError(TypeError::new(
                TypeErrorKind::Generic { message: "Type checking failed".to_string() },
                None,
            )));
        }

        // 3. Generate code
        let mut code_generator = CodeGenerator::new(self.context.clone());
        code_generator.compile(&ast.statements)?;

        // 4. Optimize the module if needed
        if self.config.optimization_level != OptimizationLevel::None {
            self.context.optimize_module();
        }

        // 5. Verify the module if configured to do so
        if self.config.verify_module && module.verify().is_err() {
            return Err(DriverError::CodeGenError(CodeGenError::code_gen_error(
                "Module verification failed".to_string(),
                None,
            )));
        }

        // Success with no return value
        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_string_simple() {
        let context = Arc::new(CompilerContext::new());
        let mut driver = Driver::new(context, "");
        let source = "x: int = 42";
        let result = driver.compile_string(source, "test.ty");
        assert!(result.is_ok(), "Compilation should succeed: {:?}", result.err());
    }
}
