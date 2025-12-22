# typhon-mir-builder

AST to MIR lowering for the Typhon compiler.

This crate transforms the high-level, typed Abstract Syntax Tree (AST) from semantic analysis into
Typhon's Mid-level Intermediate Representation (MIR) in SSA form.

## Architecture

The crate is organized into the following modules:

- **`context`**: Lowering context
  - `LoweringContext`: Main context for AST→MIR transformation
  - `LoopContext`: Context for break/continue statements
  - `ExceptionContext`: Context for try/except/finally blocks

- **`error`**: Lowering error types
  - `LoweringError`: Comprehensive error enum for lowering failures
  - `LoweringResult`: Result type for lowering operations

- **`expr`**: Expression lowering
  - Lowers AST expressions to MIR instructions
  - Handles literals, binary/unary operations, calls, attribute access, subscripting

- **`stmt`**: Statement lowering
  - Lowers AST statements to MIR instructions and control flow
  - Handles assignments, if/else, while loops, for loops, break/continue

- **`function`**: Function lowering
  - Lowers function definitions to MIR functions
  - Handles parameters, return statements, and function bodies

- **`ssa`**: SSA construction
  - Transforms lowered MIR into proper SSA form
  - Inserts phi nodes at control flow merge points

- **`optimize`**: Basic optimizations
  - Constant folding for arithmetic operations
  - Dead code elimination

## Lowering Strategy

- **Bottom-up for expressions**: Lower operands first, then the operation
- **Top-down for statements**: Create blocks first, then fill with lowered code
- **Single-pass lowering**: AST→MIR in one pass using visitor pattern
- **SSA construction**: Post-processing pass to insert phi nodes
