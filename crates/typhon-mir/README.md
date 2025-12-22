# typhon-mir

Mid-level Intermediate Representation (MIR) for the Typhon compiler.

MIR is a typed, SSA-form IR that preserves Typhon semantics while being low-level enough for optimization and efficient code generation.

## Architecture

The crate is organized into the following modules:

- **[`types`](src/types.rs)**: MIR type system
  - [`MIRType`](src/types.rs): 13-variant type enum (Int, Float, Bool, Str, None, Object, Function, Closure, Tuple, List, Dict, Ref, Void)
  - [`TypeID`](src/types.rs): Unique identifier for class types

- **[`instr`](src/instr.rs)**: MIR instructions and identifiers
  - [`MIRInstr`](src/instr.rs): 20+ instruction variants covering:
    - Memory management (IncRef, DecRef)
    - Object operations (AllocObject, GetAttr, SetAttr, GetItem, SetItem)
    - Function calls (Call, MethodCall)
    - Constants and loads (Const, Load, Store, LoadGlobal, StoreGlobal)
    - Arithmetic (BinOp, UnOp)
    - Type operations (Cast, InstanceOf)
    - Closure operations (CreateClosure, GetCapture, SetCapture)
    - SSA (Phi nodes)
  - [`Terminator`](src/instr.rs): Control flow terminators (Branch, CondBranch, Invoke, Raise, Return, Unreachable)
  - [`MIRConst`](src/instr.rs): Constant values (Bool, Int, Float, Str, None)
  - [`ValueID`](src/instr.rs): Unique identifier for SSA values
  - [`LocalID`](src/instr.rs): Unique identifier for local variables
  - [`BasicBlockID`](src/instr.rs): Unique identifier for basic blocks
  - [`BinOpKind`](src/instr.rs): Binary operation kinds (Add, Sub, Mul, Div, etc.)
  - [`UnOpKind`](src/instr.rs): Unary operation kinds (Neg, Not, BitNot)

- **[`block`](src/block.rs)**: Basic blocks and control flow
  - [`BasicBlock`](src/block.rs): Single-entry, single-exit code blocks with:
    - Instruction sequence
    - Terminator
    - Predecessor/successor tracking
    - Optional landing pad for exception handling

- **[`function`](src/function.rs)**: Function representation
  - [`MIRFunction`](src/function.rs): Complete function with parameters, locals, basic blocks,
    captures
  - [`MIRParam`](src/function.rs): Function parameter (name, type, local ID)
  - [`MIRLocal`](src/function.rs): Local variable (name, type, mutability)
  - [`MIRCapture`](src/function.rs): Captured variable for closures

- **[`module`](src/module.rs)**: Module and global definitions
  - [`MIRModule`](src/module.rs): Compilation unit containing:
    - Functions ([`Vec<MIRFunction>`](src/function.rs))
    - Global variables ([`Vec<MIRGlobal>`](src/module.rs))
    - Type definitions ([`Vec<MIRTypeDef>`](src/module.rs))
    - Value name mapping ([`value_names`](src/module.rs)) for debug information
  - [`MIRGlobal`](src/module.rs): Global variable with type, initializer, mutability
  - [`MIRTypeDef`](src/module.rs): Class definition with fields, methods, inheritance
  - [`MIRField`](src/module.rs): Type field (name, type, offset)
  - [`MethodInfo`](src/module.rs): Method metadata for classes

- **[`builder`](src/builder.rs)**: Builder API for constructing MIR
  - [`FunctionBuilder`](src/builder.rs): Ergonomic API for building MIR functions
  - Block creation, instruction emission, local/value allocation

- **[`pretty`](src/pretty.rs)**: Pretty-printing and debugging
  - Human-readable MIR output for debugging
  - Display implementations for all MIR types

## MIR Design Principles

- **SSA Form**: Static Single Assignment enables powerful dataflow optimizations
- **Typed**: All values have explicit types for validation and optimization
- **Explicit Control Flow**: Basic blocks with explicit terminators form Control Flow Graph (CFG)
- **Typhon Semantics**: Direct representation of Typhon operations (attribute access, method calls, object model)
- **Optimization Substrate**: Suitable for constant folding, dead code elimination, inlining, and escape analysis
