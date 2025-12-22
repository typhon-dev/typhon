# typhon-codegen-llvm

LLVM code generation for the Typhon programming language.

This crate implements the LLVM backend for Typhon, transforming optimized MIR (Mid-level Intermediate Representation) into executable machine code via LLVM.

## Architecture

The crate is organized into the following modules:

- **`context`**: Central state management for code generation
  - `CodegenContext`: Manages LLVM context, module, builder, and caches
  - Tracks type mappings, value mappings, local variables, and basic blocks
  - Provides helper methods for accessing cached state

- **`types`**: MIR to LLVM type translation
  - `translate_type()`: Converts MIR types to LLVM types
  - Type caching for performance optimization
  - Handles primitives, containers, functions, and closures

- **`object_layout`**: Typhon object memory layout definitions
  - `declare_object_types()`: Declares all Typhon object struct types in LLVM
  - Defines common object header with ref_count and type_id
  - Specifies memory layout for all Typhon runtime types

- **`instructions`**: MIR instruction to LLVM IR translation
  - `translate_instruction()`: Converts individual MIR instructions to LLVM IR
  - Handles constants, memory operations, arithmetic, and object operations
  - Integrates with runtime for dynamic dispatch

- **`blocks`**: Basic block translation
  - `create_basic_blocks()`: Creates LLVM basic blocks for MIR blocks
  - Manages block-to-block control flow

- **`functions`**: Function code generation
  - `generate_function()`: Translates complete MIR functions to LLVM
  - `translate_function_signature()`: Converts function signatures
  - `setup_parameters()`: Allocates and initializes function parameters
  - `allocate_locals()`: Allocates stack space for local variables

- **`runtime`**: Runtime function declarations
  - `declare_runtime_functions()`: Declares all C-ABI runtime functions
  - Provides access to memory management, arithmetic operations, and object operations
  - Bridges generated code with typhon-runtime

- **`values`**: SSA value management
  - Tracks MIR ValueID to LLVM BasicValueEnum mappings
  - Manages value lifetimes during code generation

- **`error`**: Error types for code generation failures
  - `CodegenError`: Comprehensive error enum for all failure modes
  - Provides context-rich error messages

## Object Representation

All Typhon objects are heap-allocated with a common header structure:

```llvm
%TyphonObject = type {
    i64,    ; ref_count
    i64     ; type_id
}
```

Specific types extend this base structure with additional fields:

```llvm
%TyphonInt = type {
    i64,    ; ref_count
    i64,    ; type_id
    i64     ; value
}

%TyphonFloat = type {
    i64,    ; ref_count
    i64,    ; type_id
    double  ; value
}

%TyphonStr = type {
    i64,    ; ref_count
    i64,    ; type_id
    i8*,    ; data (UTF-8 bytes)
    i64     ; length
}

%TyphonList = type {
    i64,              ; ref_count
    i64,              ; type_id
    %TyphonObject**,  ; items
    i64,              ; length
    i64               ; capacity
}

%TyphonClosure = type {
    i64,              ; ref_count
    i64,              ; type_id
    i8*,              ; function pointer
    %TyphonObject**,  ; captured variables
    i64               ; num_captures
}
```

## Code Generation Pipeline

The code generation process follows these steps:

1. **Type Setup**: Declare all Typhon object struct types in LLVM
2. **Runtime Setup**: Declare all runtime function signatures
3. **Function Translation**: For each MIR function:
   - Translate function signature
   - Create LLVM basic blocks
   - Allocate parameters and locals
   - Translate MIR instructions to LLVM IR
   - Translate MIR terminators to LLVM terminators
4. **Module Finalization**: Verify LLVM IR and apply optimization passes
5. **Output**: Generate object file or executable

## Usage Examples

### Compiling to LLVM IR

```rust
use typhon_codegen_llvm::compile_module;
use typhon_mir::module::MIRModule;

let mir_module = MIRModule {
    name: "my_program".to_string(),
    functions: vec![/* your MIR functions */],
};
let llvm_ir = compile_module(&mir_module)?;

println!("{llvm_ir}");
```

### Compiling to Object File

```rust
use typhon_codegen_llvm::{compile_to_object_file, Target};
use typhon_mir::module::MIRModule;

let mir_module = MIRModule {
    name: "my_program".to_string(),
    functions: vec![/* your MIR functions */],
};
let target = Target::default();
let object_data = compile_to_object_file(&mir_module, &target)?;

// Write to file
std::fs::write("output.o", &object_data)?;
```

### Compiling to Executable

```rust
use std::path::Path;

use typhon_codegen_llvm::{compile_to_executable, Target};
use typhon_mir::module::MIRModule;

let mir_module = MIRModule {
    name: "my_program".to_string(),
    functions: vec![/* your MIR functions */],
};
let target = Target::default();

compile_to_executable(&mir_module, &target, Path::new("my_program"))?;
```

### Custom Target Configuration

```rust
use typhon_codegen_llvm::Target;
use inkwell::OptimizationLevel;

let target = Target {
    triple: Some("x86_64-unknown-linux-gnu".to_string()),
    cpu: "haswell".to_string(),
    features: "+avx2,+fma".to_string(),
    opt_level: OptimizationLevel::Aggressive,
    linker_args: vec!["-lm".to_string(), "-lpthread".to_string()],
};
```

## LLVM Version

This crate requires LLVM 18.1. The Inkwell dependency is configured with the `llvm18-1` feature.
