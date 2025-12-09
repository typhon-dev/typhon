# 1. Typhon Programming Language

Date: 2025-12-09

## Status

Accepted

## Context

Python has become one of the most popular programming languages due to its clean, readable syntax and
gentle learning curve. However, as projects scale, Python's dynamic typing leads to several challenges:

1. **Runtime Errors**: Type errors only discovered during execution, often in production
2. **Performance Limitations**: Interpreted execution and dynamic dispatch create overhead
3. **Tooling Constraints**: IDE support limited without static type information
4. **Deployment Friction**: Requiring end users to install a Python interpreter creates adoption
   barriers, and managing Python version compatibility across environments can be difficult
5. **Convention Enforcement**: Naming conventions (like `ALL_CAPS` for constants, `_private` for
   internal APIs) rely on code review rather than compiler enforcement

Python 3 introduced optional type hints via PEP 484, but they remain suggestions checked by external
tools like mypy rather than enforced by the language itself. The type hints are not used for
optimization and provide no runtime performance benefit.

Meanwhile, compiled languages like Rust, Go, and Swift demonstrate that readable syntax can coexist
with static typing and native code generation. These languages achieve 10-100x performance
improvements over interpreted languages while maintaining developer productivity.

The challenge: Can we preserve Python's beloved syntax and ease of use while adding the safety and
performance benefits of static typing and compilation?

## Decision

We will create **Typhon**, a statically typed programming language that uses Python 3 syntax but compiles to native code via LLVM.

### 1. Static Typing as a First-Class Feature

Typhon makes type annotations **required** rather than optional:

- **Function parameters**: Must include type annotations

  ```python
  def calculate_average(numbers: list[int]) -> float:  # Types required
      total: int = sum(numbers)
      return total / len(numbers)
  ```

- **Function return types**: Must include type annotation (unless void/None)
- **Variable declarations**: Type annotations required unless type can be inferred from initializer

  ```python
  x: int = 5        # Explicit type annotation
  y = 5             # OK: Type inferred as int from literal
  z: int            # Error: Must be initialized or have explicit type
  ```

### 2. Compilation to Native Code via LLVM

Typhon compiles to native machine code rather than interpreted bytecode:

- **AOT Compilation**: Ahead-of-time compilation produces standalone executables
- **LLVM Backend**: Leverage LLVM's mature optimization infrastructure
- **Performance Target**: Achieve 3-5x speedup over CPython through:
  - Native code generation
  - Type-specific optimizations
  - Escape analysis and memory optimization
  - No interpreter overhead

### 3. Enforced Naming Conventions

Typhon **enforces** specific Python naming conventions at compile time:

- **`ALL_CAPS` for constants**: An identifier using all uppercase letters and underscores is
  treated as a constant by the compiler

  ```python
  MAX_CONNECTIONS: int = 100  # Treated as constant
  _INTERNAL_CONST: int = 42   # Private constant (both conventions combined)
  ```

- **`_prefix` for private/internal**: Leading underscore indicates private API and is enforced across:
  - **Variables**: `_internal_state`
  - **Functions**: `_helper_function()`
  - **Methods**: `def _private_method(self)`
  - **Classes**: `class _InternalClass`
  - **Modules**: `_internal_module.ty`
  - **Packages**: `_internal_package/__init__.ty`

  ```python
  def _internal_helper() -> None:  # Private function
      pass

  class MyClass:
      _private_field: int                 # Private instance variable
      public_field: int                   # Public field

      def _private_method(self) -> None:  # Private method
          pass

  class _InternalHelper:                  # Private class
      pass
  ```

**Note**: Other naming conventions like `snake_case` for functions/variables and `CamelCase` for
classes are recommended but not enforced by the compiler.

### 4. Python Syntax Compatibility with Enhanced Semantics

Typhon maintains Python 3 syntax while changing semantics:

**Compatible:**

- All Typhon code is syntactically valid Python 3
- Python's AST structure is preserved
- Control flow, operators, and expressions work identically
- Standard library naming follows Python conventions

**Enhanced:**

- Type annotations change from optional hints to required contracts
- Specific naming conventions enforced by compiler (`ALL_CAPS` constants, `_prefix` private)
- Static analysis replaces runtime type checking
- Compile-time errors replace runtime TypeErrors

**Example - Valid in both Python and Typhon:**

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"

MAX_RETRIES: int = 3

class Person:
    def __init__(self, name: str, age: int) -> None:
        self.name: str = name
        self.age: int = age
```

### 5. Restricted Dynamic Features

To enable static analysis and compilation, Typhon restricts certain Python dynamic features:

**Prohibited:**

- `eval()` and `exec()` - No runtime code execution
- Arbitrary attribute creation at runtime - Attributes must be declared in class
- Dynamic class creation at runtime - Classes defined at compile time
- Monkey-patching of classes/modules - No runtime modification

**Alternative Approaches:**

- Compile-time metaprogramming replaces runtime code generation
- Protocols and structural typing replace duck typing
- Explicit dynamic constructs (from `typhon.runtime`) for necessary dynamic operations

### 6. Compatibility Philosophy

**One-way compatibility:**

- All Typhon code is valid Python 3 syntax
- Not all Python 3 code is valid Typhon (due to type requirements)
- Migration path: Add type annotations to Python code to make it Typhon-compatible

**Tooling compatibility:**

- Python syntax highlighting works for Typhon
- Python AST tools can parse Typhon code
- Gradual migration possible by adding types incrementally

### 7. Developer Experience Priorities

- **Type Safety**: Catch errors at compile time, not runtime
- **Performance**: Native code execution without interpreter overhead
- **Tooling**: Rich IDE support via Language Server Protocol
- **Familiarity**: Python developers feel immediately comfortable
- **Clarity**: Explicit types serve as living documentation
- **Enforcement**: Compiler validates conventions, not just code review

## Consequences

### Positive Consequences

- **Type Safety**: Eliminates entire classes of runtime errors through compile-time checking
  - TypeErrors caught before code runs
  - Refactoring becomes safer with compiler verification
  - API contracts explicitly documented and enforced

- **Performance**: Achieves 3-5x improvement over CPython through native compilation
  - No interpreter overhead
  - Type-specific optimizations possible
  - Better memory management through escape analysis
  - Predictable performance characteristics

- **Convention Enforcement**: Compiler validates naming conventions automatically
  - `ALL_CAPS` constants enforced (not just suggested)
  - `_private` naming validated by compiler
  - Eliminates need for manual code review of style issues
  - Reduces cognitive load ("what is this?" becomes obvious from name)

- **Developer Experience**: Static typing enables powerful IDE features
  - Accurate code completion with type information
  - Go-to-definition works reliably
  - Safe refactoring with compiler verification
  - Instant error feedback during development

- **Code Quality**: Type annotations serve as living documentation
  - Function signatures document expected inputs/outputs
  - No divergence between documentation and implementation
  - Easier onboarding for new developers
  - Self-documenting APIs

- **Python Familiarity**: Maintains beloved Python syntax
  - Gentle learning curve for Python developers
  - Existing Python knowledge transfers directly
  - Code reads like Python but with explicit types
  - No new syntax to learn (just type annotations)

- **Migration Path**: Gradual adoption from Python possible
  - Add type annotations incrementally
  - Python tooling works on Typhon code
  - Can validate Typhon compatibility before full migration
  - Lower risk than complete language switch

### Negative Consequences

- **Verbosity**: Requires more explicit type annotations than Python
  - Every function parameter needs a type
  - Return types must be declared
  - Some variables need explicit types
  - More characters to type overall

- **Flexibility Loss**: Some dynamic Python patterns impossible in Typhon
  - Cannot use `eval()` or `exec()`
  - No runtime attribute creation
  - No monkey-patching
  - Duck typing replaced with protocols

- **Learning Curve**: Developers must understand type system concepts
  - Generic types and type parameters
  - Union types and optionals
  - Protocols vs classes
  - Type inference rules

- **Compilation Time**: AOT compilation slower than script execution
  - Must compile before running (vs immediate Python execution)
  - Edit-compile-run cycle vs edit-run cycle
  - Though `typhon check` provides fast type checking without codegen

- **Ecosystem Split**: Not directly compatible with Python packages
  - Cannot directly import Python modules (requires compatibility layer)
  - Split ecosystem during transition period
  - Need to reimplement or wrap Python libraries
  - May fragment community

- **Migration Effort**: Converting Python code to Typhon requires work
  - Must add type annotations to existing code
  - May need to restructure dynamic patterns
  - Cannot use certain Python idioms
  - Testing required to ensure correct types

- **Convention Rigidity**: Enforced naming may conflict with existing codebases
  - Legacy code may use different conventions
  - Integration with external systems may clash with naming rules
  - Less flexibility for special cases
  - Migration requires renaming violations

### Neutral Consequences

- **Binary Deployment**: Compiled executables vs interpreted scripts
  - Positive: No Python runtime needed for deployment, smaller attack surface
  - Negative: Larger binary sizes, platform-specific builds required

- **Error Location**: Compile-time errors vs runtime errors
  - Positive: Catch errors earlier in development cycle
  - Negative: Cannot run partially-correct code for experimentation

- **Type Annotation Syntax**: Uses Python 3 type hint syntax
  - Positive: Familiar to Python developers, no new syntax
  - Negative: Inherits some awkwardness (e.g., `list[int]` vs `List[int]`)

- **Static Analysis**: All code analyzable without execution
  - Positive: Better tooling, optimization opportunities
  - Negative: Precludes certain runtime flexibility

### Trade-offs Accepted

We accept **increased verbosity** (type annotations everywhere) to gain:

- Type safety and error prevention
- Performance through compilation
- Superior IDE tooling support
- Self-documenting code

We accept **reduced flexibility** (no eval/exec, limited dynamics) to gain:

- Static analyzability for optimization
- Predictable performance
- Compile-time error detection
- Better security posture

We accept **compilation overhead** (slower edit-run cycle) to gain:

- Native code performance at runtime
- Better deployment story (single executable)
- Type checking before execution
- Production performance benefits

We accept **ecosystem fragmentation** (not directly Python-compatible) to gain:

- Clean language design without legacy constraints
- Opportunity for breaking improvements
- Focus on compilation-friendly patterns
- Long-term better performance and safety

### What Becomes Easier

- **Error Detection**: Type errors caught immediately during development, not in production
- **Refactoring**: Compiler verifies all changes, making large-scale refactors safe
- **Code Review**: Focus on logic rather than style (conventions auto-enforced)
- **Documentation**: Types serve as precise API documentation
- **IDE Support**: Accurate completion, navigation, and refactoring tools
- **Performance Optimization**: Compiler can optimize based on type information
- **Code Understanding**: Types make code intent explicit and obvious
- **Deployment**: Single native executable, no runtime dependencies

### What Becomes More Difficult

- **Prototyping**: Must think about types upfront, cannot "script freely"
- **Dynamic Patterns**: Cannot use eval, runtime introspection, or monkey-patching
- **Python Integration**: Cannot directly use Python packages without FFI layer
- **Quick Scripts**: More ceremony required for simple programs
- **Migration**: Existing Python codebases need significant annotation work
- **Learning**: Must understand type system concepts beyond Python basics
- **Convention Changes**: Legacy code with non-standard naming requires renaming

### Success Metrics

The decision will be considered successful if:

- **Performance**: Achieve 3-5x speedup over CPython on benchmarks
- **Adoption**: Python developers can learn Typhon in < 1 week
- **Safety**: Eliminate >90% of runtime TypeErrors through compile-time checking
- **Tooling**: IDE support matches or exceeds Python's mypy-based tooling
- **Migration**: Provide clear path for Python→Typhon code conversion
- **Clarity**: Developers report types improve code understanding
- **Enforcement**: Convention violations caught at compile time
