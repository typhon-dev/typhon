# Typhon Programming Language Specification

## 1. Introduction

Typhon is a statically and strongly typed programming language based on Python 3. It aims to combine Python's elegant syntax and ease of use with the benefits of static type checking and compiled performance. This specification defines the syntax, semantics, and implementation requirements for the Typhon language.

### 1.1 Design Goals

- **Strongly and statically typed**: Catch type errors at compile time
- **Python compatibility**: Maintain Python 3's syntax and AST structure where possible
- **Performance**: Achieve 3-5x performance improvement over CPython through compilation
- **Safety**: Prevent common runtime errors through static analysis
- **Developer experience**: Provide comprehensive IDE support through LSP
- **Compilation**: Generate efficient machine code via LLVM

### 1.2 Relationship to Python

Typhon uses Python 3's syntax as its foundation but adds static typing. It maintains compatibility with Python's AST (Abstract Syntax Tree) structure while enforcing type safety at compile time. Typhon aims to feel familiar to Python developers while providing the benefits of a compiled language.

## 2. Syntax Specification

### 2.1 Lexical Structure

Typhon's lexical structure is identical to Python 3, including:

- Keywords
- Identifiers
- Literals (numeric, string, bytes, etc.)
- Operators
- Delimiters
- Indentation for block structure

### 2.2 Grammar

Typhon's grammar is based on Python 3's grammar with extensions for static typing:

```ebnf
# Extensions to Python 3 grammar for static typing
type_annotation: ':' expression
var_declaration: 'var' identifier type_annotation ['=' expression]
const_declaration: 'const' identifier type_annotation '=' expression
function_type: '(' [type_list] ')' '->' expression
generic_type: identifier '[' type_list ']'
type_list: expression (',' expression)*
```

### 2.3 Type Annotations

Type annotations in Typhon are required for function parameters, function return types, and variable declarations unless they can be inferred from initializers.

```python
# Function with type annotations
def add(a: int, b: int) -> int:
    return a + b

# Variable with type annotation
x: int = 5

# Variable with type inference
y = 5  # Type inferred as int

# Constants
const PI: float = 3.14159
```

## 3. Type System

### 3.1 Basic Types

Typhon includes the following basic types:

- `int`: Integer of arbitrary precision
- `float`: Double-precision floating point
- `bool`: Boolean
- `str`: UTF-8 string
- `bytes`: Byte sequence
- `None`: Null type (similar to `None` in Python)

### 3.2 Container Types

Container types in Typhon are generic and parameterized:

- `list[T]`: Mutable sequence of elements of type T
- `tuple[T1, T2, ...]`: Immutable sequence of elements with specified types
- `dict[K, V]`: Mapping from keys of type K to values of type V
- `set[T]`: Unordered collection of unique elements of type T

### 3.3 Function Types

Functions in Typhon have explicit types for parameters and return values:

```python
def process(data: list[int], factor: float = 1.0) -> list[float]:
    return [x * factor for x in data]
```

Function types can be specified using the arrow notation:

```python
Processor = Callable[[list[int], float], list[float]]
```

### 3.4 Class Types

Classes in Typhon define new types:

```python
class Point:
    def __init__(self, x: int, y: int):
        self.x: int = x
        self.y: int = y

    def distance(self, other: 'Point') -> float:
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5
```

### 3.5 Type Inference

Typhon performs type inference to determine types of expressions where explicit annotations are not provided:

- Local variables initialized in their declaration
- Loop variables in for-loops
- Return types of lambdas with simple expressions

### 3.6 Generics

Typhon supports generic types using square bracket notation:

```python
def first_element[T](elements: list[T]) -> T:
    return elements[0]
```

### 3.7 Union and Optional Types

Typhon supports union types and optional values:

```python
# Union type
result: int | str = process_data()

# Optional type (equivalent to T | None)
name: str | None = get_name()
```

### 3.8 Type Protocols

Typhon supports structural typing through protocols:

```python
protocol Serializable:
    def to_json(self) -> str: ...

def save_data(obj: Serializable, path: str) -> None:
    with open(path, 'w') as f:
        f.write(obj.to_json())
```

### 3.9 Type Aliases

Type aliases provide names for complex types:

```python
Matrix = list[list[float]]
Point = tuple[float, float]
```

## 4. Language Features

### 4.1 Supported Python Features

Typhon supports most Python features, including:

- Functions and lambdas
- Classes and inheritance
- Modules and imports
- Control flow statements (if, for, while, etc.)
- List, dictionary, and set comprehensions
- Exception handling
- Context managers
- Decorators
- String formatting
- Operator overloading
- F-strings
- Async/await
- Type annotations
- Pattern matching

### 4.2 Modified Features

Some Python features are modified to work with static typing:

- Duck typing is replaced with protocol-based structural typing
- Runtime type checking is replaced with compile-time type checking
- Dynamic attribute access requires static verification or special runtime constructs
- Metaprogramming is limited to compile-time operations

### 4.3 Special Runtime Constructs

Typhon provides special constructs for working with dynamic features in a type-safe way:

#### 4.3.1 Dynamic Attribute Access

```python
from typhon.runtime import dynamic

# Regular attribute access requires compile-time verification
obj.attribute  # Must be statically known

# Dynamic attribute access requires explicit annotation
attr_name: str = "attribute"
value = dynamic.getattr(obj, attr_name, default=None)  # Returns specified default if not found
```

#### 4.3.2 Reflection

```python
from typhon.runtime import reflect

# Get type information at runtime
t = reflect.typeof(obj)
fields = reflect.fields(obj)
```

#### 4.3.3 Type Casting

```python
# Safe cast with runtime check
value = cast(str, obj)  # Raises TypeError if obj is not a str

# Unsafe cast without runtime check
value = unsafe_cast(str, obj)  # Assumes obj is a str without checking
```

### 4.4 Restricted Features

Some Python features are restricted in Typhon for type safety:

- No `eval()` or `exec()` (use compile-time metaprogramming instead)
- No arbitrary attribute creation at runtime (must be defined in class)
- No arbitrary monkey-patching of classes or modules
- No dynamic code generation at runtime (use code generation at compile time)
- No dynamic class creation at runtime (use metaclasses at compile time)

## 5. Semantic Rules

### 5.1 Type Compatibility

Typhon uses nominal typing for classes and structural typing for protocols:

- A class is compatible with its superclass (subtyping)
- A class is compatible with protocols it implements (structural compatibility)
- Basic types are compatible through explicit subtyping relationships

### 5.2 Assignment Compatibility

Values can be assigned if their types are compatible:

```python
def requires_vehicle(v: Vehicle) -> None:
    pass

car: Car = Car()  # Car is a subclass of Vehicle
requires_vehicle(car)  # Valid, Car is a subtype of Vehicle
```

### 5.3 Type Conversion

Implicit conversions are not allowed between basic types:

```python
x: int = 5
y: float = x  # Error: Cannot implicitly convert int to float
z: float = float(x)  # OK: Explicit conversion
```

### 5.4 Function Overloading

Typhon supports function overloading based on parameter types:

```python
overload
def process(x: int) -> int:
    return x * 2

overload
def process(s: str) -> str:
    return s + s
```

### 5.5 Operator Semantics

Operators in Typhon follow Python's semantics but require compatible types:

```python
def __add__(self, other: Point) -> Point:  # Must specify parameter type
    return Point(self.x + other.x, self.y + other.y)
```

### 5.6 Memory Management

Typhon uses automatic memory management with optimization opportunities:

- Escape analysis to minimize heap allocations
- Region-based memory management for certain patterns
- Reference counting with cycle detection
- Compile-time ownership analysis for optimization

## 6. Compile-time Checks and Analysis

### 6.1 Type Checking

The compiler performs comprehensive type checking:

- Function calls with argument type compatibility
- Return value type compatibility
- Assignment type compatibility
- Operator type compatibility
- Generic type parameter constraints
- Protocol conformance

### 6.2 Flow-Sensitive Typing

Typhon performs flow-sensitive type analysis:

```python
def process(x: int | None) -> int:
    if x is None:
        return 0
    # Here, x is known to be int
    return x + 1
```

### 6.3 Exhaustiveness Checking

Pattern matching requires exhaustive coverage:

```python
def describe(shape: Shape) -> str:
    match shape:
        case Circle():
            return "circle"
        case Rectangle():
            return "rectangle"
        # Error: Missing case for Triangle
```

### 6.4 Null Safety

Nullable types require explicit handling:

```python
def process(name: str | None) -> str:
    if name is None:
        return "Anonymous"
    # Here, name is known to be str
    return name.upper()
```

## 7. Error Handling

### 7.1 Exception Handling

Typhon supports Python-style exception handling:

```python
try:
    process_data()
except ValueError as e:
    handle_error(e)
finally:
    cleanup()
```

### 7.2 Result Types

For expected error cases, Typhon provides a Result type:

```python
def parse_int(s: str) -> Result[int, str]:
    try:
        return Ok(int(s))
    except ValueError:
        return Err("Invalid integer")

result = parse_int("123")
match result:
    case Ok(value):
        print(f"Parsed: {value}")
    case Err(message):
        print(f"Error: {message}")
```

### 7.3 Error Reporting

Compile-time errors provide detailed information:

- Error location (file, line, column)
- Error type and description
- Relevant type information
- Suggestions for fixing the error

## 8. Compilation Pipeline

### 8.1 Parsing

The Typhon compiler parses source code into an AST matching Python 3's structure.

### 8.2 Type Checking

The AST undergoes static type checking and semantic analysis.

### 8.3 AST Transformations

The typed AST is transformed for optimization and code generation.

### 8.4 LLVM IR Generation

The transformed AST is converted to LLVM Intermediate Representation.

### 8.5 Optimization

LLVM optimizations are applied to the IR.

### 8.6 Code Generation

LLVM generates native machine code for the target platform.

## 9. Standard Library

### 9.1 Core Modules

Typhon's standard library includes type-safe versions of Python's core modules:

- `typhon.builtins`: Basic built-in functions and types
- `typhon.collections`: Data structures (list, dict, set, etc.)
- `typhon.io`: Input/output operations
- `typhon.os`: Operating system interfaces
- `typhon.math`: Mathematical functions and constants
- `typhon.concurrent`: Concurrency and parallelism

### 9.2 Additional Modules

Typhon adds modules for features specific to the language:

- `typhon.unsafe`: Operations that bypass the type system
- `typhon.ffi`: Foreign function interface
- `typhon.reflect`: Runtime type information
- `typhon.compile`: Compile-time metaprogramming
- `typhon.async`: Asynchronous programming utilities

### 9.3 External Dependencies

Typhon's standard library can interoperate with:

- C libraries through FFI
- Rust libraries through direct binding
- Python packages through a compatibility layer

## 10. LSP Implementation

### 10.1 Language Server Features

The Typhon Language Server Protocol implementation provides:

- Code completion
- Hover information
- Go-to-definition
- Find references
- Rename refactoring
- Error diagnostics
- Code formatting
- Code actions

### 10.2 IDE Integration

The language server can be integrated with:

- Visual Studio Code
- JetBrains IDEs
- Vim/Neovim
- Emacs
- Other LSP-compatible editors

## 11. Future Extensions

Areas for future language evolution:

- Dependent types
- Effect systems
- Refinement types
- Stronger metaprogramming capabilities
- Formal verification
- Domain-specific language embedding

## 12. Implementation Guidance

### 12.1 Compiler Architecture

The Typhon compiler should be implemented in Rust with a modular architecture:

- Frontend: Parsing, type checking
- Middle-end: Analysis, transformation
- Backend: Code generation, LLVM integration

### 12.2 Runtime System

The runtime system should provide:

- Memory management
- Exception handling
- Type information
- Foreign function interface
- Standard library implementation

### 12.3 Testing Strategy

The implementation should include:

- Unit tests for compiler components
- Integration tests for the compilation pipeline
- Conformance tests for language features
- Performance benchmarks
- Compatibility tests with Python code

## Appendix A: Grammar Specification

(Detailed EBNF grammar for Typhon)

## Appendix B: AST Structure

(Detailed description of Typhon's AST structure based on Python 3)

## Appendix C: Type System Semantics

(Formal semantics of the type system)

## Appendix D: Standard Library API

(Complete API reference for the standard library)
