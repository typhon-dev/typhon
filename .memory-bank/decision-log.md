---
title: Decision Log
description: Records architectural and implementation decisions in the project
tags: ["memory-bank", "documentation", "decision-log", "architecture", "implementation", "decisions", "design"]
---
<!-- markdownlint-disable-file no-duplicate-heading -->

This file records architectural and implementation decisions using a list format.

---

## Decision

Implement Memory Bank for project context tracking

## Rationale

Memory Bank provides a structured way to maintain project knowledge, making it easier to track decisions, progress, and key technical details.

## Implementation Details

- Markdown files for different types of information
- Automated updates for significant changes
- Consistent timestamping for tracking evolution

---

## Decision

Adopt crates-based project structure with separate packages for compiler, runtime, LSP, etc.

## Rationale

Modular design allows better separation of concerns and more focused development. Each component can evolve independently while maintaining clean interfaces.

## Implementation Details

- typhon-cli: Command-line interface
- typhon-compiler: Core compiler components
- typhon-lsp: Language Server Protocol implementation
- typhon-repl: Interactive REPL
- typhon-runtime: Runtime support
- typhon-stdlib: Standard library

---

## Decision

Use LLVM as the backend for code generation

## Rationale

LLVM provides a mature optimization pipeline and multiple target support, saving significant development time and ensuring high-quality code generation.

## Implementation Details

- Integration via llvm-sys crate
- LLVM IR generation from Typhon's intermediate representation
- Leveraging LLVM's optimization passes

---

## Decision

Use Logos for lexing and LALRPOP for parsing

## Rationale

These libraries offer good performance and maintainability for Rust compiler projects, with strong typing and error reporting capabilities.

## Implementation Details

- Logos for efficient lexical analysis
- LALRPOP for grammar-based parser generation
- Custom error reporting integrations

---

## Decision

Create a common module for shared types between token and AST modules

## Rationale

Eliminates type mismatches between compiler stages and ensures consistent representation of shared concepts across the compiler.

## Implementation Details

- Common SourceInfo representation
- Shared location and span types
- Unified error representation

---

## Decision

Standardize on Box<Expression> for AST node references

## Rationale

Provides consistent memory management for AST nodes and avoids ownership confusion when passing nodes between compiler stages.

## Implementation Details

- All expression fields in AST nodes use Box<Expression>
- Consistent ownership model across the AST
- Clear patterns for traversal and transformation

---

## Decision

Handle LLVM API incompatibilities by properly managing Result types

## Rationale

Modern LLVM API (18+) returns Results instead of direct values for error handling, requiring updates throughout our code generator.

## Implementation Details

- Converting LLVM Builder method calls to handle Result types with proper error handling
- Consistent error propagation patterns
- Explicit handling of BuilderError types

---

## Decision

Update pointer handling to match LLVM 18's new API

## Rationale

LLVM 18 no longer differentiates between pointer types, requiring updates to our type mapping code.

## Implementation Details

- Using new LLVM Context-level ptr_type() method instead of type-specific ptr_type()
- Consistent pointer type creation throughout the codebase
- Addressing all deprecated API warnings

---

## Decision

Redesign memory management in CodeGenerator using Arc<Mutex<>> pattern

## Rationale

Provides thread-safe shared mutable access to state while maintaining proper lifetimes, addressing fundamental borrow checker issues in the code generator.

## Implementation Details

- Wrapping mutable compiler state in Arc<Mutex<>> for thread-safe interior mutability
- Proper synchronization of access to mutable state
- Clean separation between shared and exclusive access patterns

---

## Decision

Split CodeGenerator into immutable context and mutable state components

## Rationale

Separates immutable context from mutable state to avoid conflicting borrows, making the code safer and more maintainable.

## Implementation Details

- Immutable context contains LLVM context, target information, etc.
- Mutable state contains builder, current function, blocks, etc.
- Clear patterns for when to access each component

---

## Decision

Create comprehensive LLVM compatibility documentation

## Rationale

Helps future developers understand API changes and patterns, reducing the learning curve and preventing regression of compatibility issues.

## Implementation Details

- Documenting all architectural changes for future reference
- Creating a dedicated llvm-compatibility.md document
- Including examples of old and new API usage patterns

---

## Decision

Add support for Python-style boolean literals (True/False) and None as tokens in the AST

## Rationale

As a Python-based language, Typhon should follow Python's convention of using capitalized True and False for boolean literals and None as a null value, making code more familiar to Python developers.

## Implementation Details

- Added Literal::Bool(bool) and Literal::None variants in the AST's Literal enum
- Implemented True, False, and None as dedicated tokens in the lexer
- Ensured consistent handling between lexer and parser stages
- Used the same pattern for all three special literals to maintain consistency

---

## Decision

Refactor dependency management using workspace dependencies

## Rationale

Centralizing dependency management in the workspace root improves consistency, simplifies updates, and prevents version mismatches across crates.

## Implementation Details

- Moved all dependency declarations to workspace.dependencies section in root Cargo.toml
- Used .workspace = true syntax for dependencies in individual crates
- Organized external dependencies with clear comments and grouping

---

## Decision

Treat .ty files with same indent rules as .rs files

## Rationale

Consistent indentation across both Rust source and Typhon files improves developer experience and code readability.

## Implementation Details

- Updated .editorconfig to apply the same indent rules to .rs and .ty files
- Set indent_size = 4 for both file types
- Ensures code in both languages follows the same visual structure

---

2025-10-20 04:39:00 - Initial creation of decision log.
2025-10-20 05:05:00 - Added decisions on project structure, backend, and parsing libraries.
2025-11-07 22:08:00 - Added decisions related to LLVM compatibility fixes and type system improvements.
2025-11-07 23:24:00 - Updated with architectural decisions for memory management redesign and LLVM compatibility.
2025-11-08 18:50:00 - Added decisions about Python-style True/False/None keywords, dependency management, and file indentation.
