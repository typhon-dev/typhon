# typhon-mir-builder

AST to MIR lowering for the Typhon compiler.

This crate transforms the high-level, typed Abstract Syntax Tree (AST) from semantic analysis into
Typhon's Mid-level Intermediate Representation (MIR).

## Architecture

The crate is organized into the following modules:

- **[`context`](src/context.rs)**: Lowering context and state management
  - [`LoweringContext`](src/context.rs): Main context for AST→MIR transformation with semantic
    analysis integration
  - [`LoopContext`](src/context.rs): Context for break/continue statements
  - [`ExceptionContext`](src/context.rs): Context for try/except/finally blocks
  - [`CaptureInfo`](src/context.rs): Closure capture tracking

- **[`error`](src/error.rs)**: Lowering error types
  - [`LoweringError`](src/error.rs): Comprehensive error enum for lowering failures
  - [`LoweringResult`](src/error.rs): Result type for lowering operations

- **[`expr`](src/expr.rs)**: Expression lowering
  - Lowers AST expressions to MIR instructions
  - Handles literals, binary/unary operations, calls, attribute access, subscripting
  - Integrates with type environment for precise type information

- **[`stmt`](src/stmt.rs)**: Statement lowering
  - Lowers AST statements to MIR instructions and control flow
  - Handles assignments, if/else, while loops, break/continue
  - Note: For loops require iterator protocol (planned enhancement)

- **[`function`](src/function.rs)**: Function lowering
  - Lowers function definitions to MIR functions
  - Handles parameters, return statements, and function bodies
  - Supports recursive functions

- **[`class`](src/class.rs)**: Class lowering
  - Lowers class definitions with method mangling (`ClassName__method`)
  - Extracts fields from `__init__` method analysis
  - Handles inheritance and base class resolution

- **[`closure`](src/closure.rs)**: Closure lowering
  - Integrates with analyzer's capture analysis
  - Generates closure allocation and capture access instructions
  - Supports nested closures

- **[`symbol_resolution`](src/symbol_resolution.rs)**: Symbol and name resolution
  - [`NameClassification`](src/symbol_resolution.rs): Classifies names as local, captured, global,
    or builtin
  - Integrates with typhon-analyzer's symbol table
  - Provides helper functions for symbol lookup

- **[`type_mapping`](src/type_mapping.rs)**: Type system integration
  - Maps analyzer types to MIR types
  - Provides [`ToMIRType`](src/type_mapping.rs) trait for type conversion
  - Handles all Typhon type variants

## Semantic Analysis Integration

The MIR builder integrates with the [`typhon-analyzer`](../typhon-analyzer) crate to leverage type
and symbol information:

### Type Information

The [`LoweringContext`](src/context.rs) can be created with an
[`AnalysisContext`](../typhon-analyzer/src/context.rs) to access:

- **Type Environment**: Inferred and annotated types for all AST nodes
- **Symbol Table**: Variable scope, binding, and capture information
- **Type Queries**: Methods like `query_expr_type()` and `query_name_type()` retrieve precise type
  information

### Name Resolution

The integration provides:

- **Scope Classification**: Determines if names are local, captured, global, or builtin
- **Closure Analysis**: Uses analyzer's capture tracking for closure variable access
- **Symbol Lookup**: Direct access to symbol metadata for all identifiers

### Usage Modes

The builder supports two modes:

1. **With Semantics** (recommended): `LoweringContext::new_with_semantics(ast, module_name,
   semantic_context)`
   - Precise type information from type checker
   - Symbol resolution from symbol table
   - Better error messages with source spans

2. **Legacy Mode** (fallback): `LoweringContext::new(ast, module_name)`
   - Uses generic `Object` type as default
   - Basic name resolution without semantic context
   - Useful for testing or when semantic analysis unavailable

## Lowering Strategy

- **Bottom-up for expressions**: Lower operands first, then the operation
- **Top-down for statements**: Create blocks first, then fill with lowered code
- **Single-pass lowering**: AST→MIR in one pass with semantic queries
- **Type-directed**: Uses type information to generate appropriate MIR instructions
