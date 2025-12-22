//! LLVM code generation for the Typhon programming language.
//!
//! This crate implements the LLVM backend for Typhon, transforming optimized MIR
//! (Mid-level Intermediate Representation) into executable machine code via LLVM.
//!
//! ## Architecture
//!
//! The code generation process follows these steps:
//!
//! 1. **Type Translation**: Convert MIR types to LLVM types
//! 2. **Object Layout Setup**: Declare Typhon object struct types in LLVM
//! 3. **Runtime Setup**: Declare runtime function signatures
//! 4. **Function Generation**: Translate MIR functions to LLVM IR
//! 5. **Module Finalization**: Verify and optimize LLVM IR
//! 6. **Output**: Generate object files or executables
//!
//! ## Example
//!
//! ```rust,ignore
//! use typhon_codegen_llvm::compile_to_object_file;
//! use typhon_mir::MIRModule;
//! use std::path::Path;
//!
//! # fn example(mir_module: MIRModule) -> Result<(), Box<dyn std::error::Error>> {
//! // Compile MIR module to object file
//! compile_to_object_file(&mir_module, Path::new("output.o"))?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod blocks;
pub mod context;
pub mod error;
pub mod functions;
pub mod instructions;
pub mod object_layout;
pub mod runtime;
pub mod types;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use context::CodegenContext;
use error::{CodegenError, CodegenResult};
use functions::generate_function;
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::targets::{
    CodeModel,
    FileType,
    InitializationConfig,
    RelocMode,
    Target as LLVMTarget,
    TargetMachine,
};
use object_layout::declare_object_types;
use runtime::declare_runtime_functions;
use typhon_mir::module::MIRModule;

/// Target configuration for code generation.
#[derive(Debug, Clone)]
pub struct Target {
    /// Target triple (e.g., "x86_64-unknown-linux-gnu").
    /// If None, uses the native target.
    pub triple: Option<String>,
    /// CPU name (e.g., "generic", "haswell").
    pub cpu: String,
    /// CPU features (e.g., "+avx2,+fma").
    pub features: String,
    /// Optimization level (None, Less, Default, Aggressive).
    pub opt_level: OptimizationLevel,
    /// Additional linker arguments.
    pub linker_args: Vec<String>,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            triple: None,
            cpu: "generic".to_string(),
            features: String::new(),
            opt_level: OptimizationLevel::Default,
            linker_args: vec![],
        }
    }
}

/// Compiles a MIR module to LLVM IR.
///
/// This is the main entry point for code generation. It creates an LLVM module,
/// translates all functions from MIR to LLVM IR, and returns the complete module
/// as a string containing LLVM IR.
///
/// ## Arguments
///
/// * `mir_module` - The MIR module to compile
///
/// ## Errors
///
/// Returns error if LLVM initialization fails, type translation fails,
/// or function generation fails.
///
/// ## Example
///
/// ```rust,ignore
/// use typhon_codegen_llvm::compile_module;
/// use typhon_mir::MIRModule;
///
/// # fn example(mir_module: MIRModule) -> Result<(), Box<dyn std::error::Error>> {
/// let llvm_ir = compile_module(&mir_module)?;
/// println!("{}", llvm_ir);
/// # Ok(())
/// # }
/// ```
pub fn compile_module(mir_module: &MIRModule) -> CodegenResult<String> {
    // Initialize LLVM
    LLVMTarget::initialize_native(&InitializationConfig::default())
        .map_err(CodegenError::LLVMInitError)?;

    // Create LLVM context and module
    let context = Context::create();
    let module = context.create_module(&mir_module.name);
    let builder = context.create_builder();

    // Create codegen context
    let mut ctx = CodegenContext::new(&context, module, builder);

    // Declare object types
    declare_object_types(&mut ctx)?;

    // Declare runtime functions
    declare_runtime_functions(&mut ctx)?;

    // Compile each function
    for mir_func in &mir_module.functions {
        generate_function(&mut ctx, mir_func)?;
    }

    // Verify module
    if let Err(err) = ctx.module.verify() {
        return Err(CodegenError::InvalidModule(err.to_string()));
    }

    // Return LLVM IR as string
    Ok(ctx.module.print_to_string().to_string())
}

/// Compiles a MIR module to an object file.
///
/// This function performs complete compilation from MIR to native object code,
/// including LLVM IR generation, optimization, and object file emission.
/// Returns the object file contents as a byte vector.
///
/// ## Arguments
///
/// * `mir_module` - The MIR module to compile
/// * `target` - Target configuration for compilation
///
/// ## Errors
///
/// Returns error if compilation fails, target initialization fails,
/// or object file generation fails.
///
/// ## Example
///
/// ```rust,ignore
/// use typhon_codegen_llvm::{compile_to_object_file, Target};
/// use typhon_mir::MIRModule;
///
/// # fn example(mir_module: MIRModule) -> Result<(), Box<dyn std::error::Error>> {
/// let target = Target::default();
/// let object_data = compile_to_object_file(&mir_module, &target)?;
/// std::fs::write("output.o", &object_data)?;
/// # Ok(())
/// # }
/// ```
pub fn compile_to_object_file(mir_module: &MIRModule, target: &Target) -> CodegenResult<Vec<u8>> {
    // Initialize LLVM
    LLVMTarget::initialize_all(&InitializationConfig::default());

    // Create LLVM context and module
    let context = Context::create();
    let module = context.create_module(&mir_module.name);
    let builder = context.create_builder();

    // Create codegen context
    let mut ctx = CodegenContext::new(&context, module, builder);

    // Declare object types and runtime
    declare_object_types(&mut ctx)?;
    declare_runtime_functions(&mut ctx)?;

    // Compile functions
    for mir_func in &mir_module.functions {
        generate_function(&mut ctx, mir_func)?;
    }

    // Verify module
    if let Err(err) = ctx.module.verify() {
        return Err(CodegenError::InvalidModule(err.to_string()));
    }

    // Get target triple
    let target_triple =
        target.triple.as_ref().map_or_else(TargetMachine::get_default_triple, |triple| {
            inkwell::targets::TargetTriple::create(triple)
        });

    let llvm_target = LLVMTarget::from_triple(&target_triple)
        .map_err(|e| CodegenError::TargetInitError(e.to_string()))?;

    // Create target machine
    let target_machine = llvm_target
        .create_target_machine(
            &target_triple,
            &target.cpu,
            &target.features,
            target.opt_level,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| {
            CodegenError::TargetInitError("Failed to create target machine".to_string())
        })?;

    // Emit object file to memory buffer and return as Vec<u8>
    let buffer = target_machine
        .write_to_memory_buffer(&ctx.module, FileType::Object)
        .map_err(|e| CodegenError::CodegenError(e.to_string()))?;

    Ok(buffer.as_slice().to_vec())
}

/// Compiles a MIR module to an executable file.
///
/// This performs complete compilation including LLVM IR generation,
/// object file emission, and linking with the runtime library.
///
/// ## Arguments
///
/// * `mir_module` - The MIR module to compile
/// * `target` - Target configuration for compilation
/// * `output_path` - Path where the executable will be written
///
/// ## Errors
///
/// Returns error if compilation fails, linking fails, or file I/O fails.
///
/// ## Example
///
/// ```rust,ignore
/// use typhon_codegen_llvm::{compile_to_executable, Target};
/// use typhon_mir::MIRModule;
/// use std::path::Path;
///
/// # fn example(mir_module: MIRModule) -> Result<(), Box<dyn std::error::Error>> {
/// let target = Target::default();
/// compile_to_executable(&mir_module, &target, Path::new("my_program"))?;
/// # Ok(())
/// # }
/// ```
pub fn compile_to_executable(
    mir_module: &MIRModule,
    target: &Target,
    output_path: &Path,
) -> CodegenResult<()> {
    // Compile to object file in memory
    let object_data = compile_to_object_file(mir_module, target)?;

    // Write object file to temporary location
    let temp_obj = output_path.with_extension("o");
    let mut file = File::create(&temp_obj)
        .map_err(|e| CodegenError::LinkingError(format!("Failed to write object file: {e}")))?;
    file.write_all(&object_data)
        .map_err(|e| CodegenError::LinkingError(format!("Failed to write object file: {e}")))?;

    // Determine system linker
    let linker = if cfg!(target_os = "macos") {
        "clang"
    } else if cfg!(target_os = "linux") {
        "gcc"
    } else if cfg!(target_os = "windows") {
        "link.exe"
    } else {
        "ld"
    };

    // Link with system linker
    let mut cmd = Command::new(linker);
    cmd.arg(&temp_obj).arg("-o").arg(output_path);

    // Add additional linker arguments
    for arg in &target.linker_args {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .map_err(|err| CodegenError::LinkingError(format!("Failed to run linker: {err}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CodegenError::LinkingError(format!("Linker failed: {stderr}")));
    }

    // Clean up temporary object file
    drop(std::fs::remove_file(&temp_obj));

    Ok(())
}
