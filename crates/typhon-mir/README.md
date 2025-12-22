# typhon-mir

Mid-level Intermediate Representation (MIR) for the Typhon compiler.

MIR is a typed, SSA-form IR that preserves Typhon semantics while being low-level enough for optimization and efficient code generation.

## Architecture

The crate is organized into the following modules:

- **`types`**: MIR type system
  - `MIRType`: Comprehensive type enum covering all MIR types (Int, Float, Bool, String, Object, Function, etc.)
  - Type representation for boxed Typhon objects, functions, closures, and containers

- **`instruction`**: MIR instructions and operands
  - `Instruction`: All MIR instructions (arithmetic, memory, object operations, calls)
  - `Operand`: Instruction operands (registers, constants)
  - `Register`: SSA register with unique ID and type
  - `Constant`: Constant values (Int, Float, Bool, String, None)

- **`block`**: Basic blocks and control flow
  - `BasicBlock`: Single-entry, single-exit code blocks with instructions and terminator
  - `Terminator`: Control flow instructions (Return, Jump, Branch, Unreachable)
  - `BlockID`: Unique identifier for basic blocks

- **`function`**: Function representation
  - `MIRFunction`: Complete function with parameters, locals, basic blocks, and control flow
  - Function metadata including name, return type, and entry block

- **`module`**: Module and global definitions
  - `MIRModule`: Compilation unit containing functions and globals
  - `Global`: Global variable definitions with initializers

- **`builder`**: Builder API for constructing MIR
  - `FunctionBuilder`: Ergonomic API for building MIR functions
  - Block creation, instruction emission, register allocation

- **`display`**: Pretty-printing and debugging
  - `Display` implementations for human-readable MIR output
  - Debug formatting for development and testing

## MIR Design Principles

- **SSA Form**: Static Single Assignment enables powerful dataflow optimizations
- **Typed**: All values have explicit types for validation and optimization
- **Explicit Control Flow**: Basic blocks with explicit terminators form Control Flow Graph (CFG)
- **Typhon Semantics**: Direct representation of Typhon operations (attribute access, method calls, object model)
- **Optimization Substrate**: Suitable for constant folding, dead code elimination, inlining, and escape analysis
