---
title: System Patterns *Optional*
description: Documents recurring patterns and standards used in the project
tags: ["memory-bank", "documentation", "system-patterns", "coding-patterns", "architectural-patterns", "testing-patterns", "patterns", "standards", "code-organization", "architecture", "testing"]
---

This file documents recurring patterns and standards used in the project.

## Coding Patterns

- Imports must be grouped by StdExternalCrate (not by default)
- Imports use vertical layout (not mixed)
- Doc comments can use special identifiers (CPython, FastAPI, etc.)
- Rust 2024 edition style, modular crate structure with clear interfaces
- Use Box<Expression> for all expression fields in AST
- Prefix unused variables with underscore (_)
- Handle Result types from LLVM API calls with proper error handling
- Use explicit type conversions between related types
- Handle LLVM Builder methods as Result<T, BuilderError> with proper error propagation

## Architectural Patterns

- Crate-based modularization
- Clear separation between compiler stages
- Type system as central component
- Pipeline-based compiler design with distinct phases
- Common module for shared type definitions
- Thread-safe memory management with Arc<Mutex<>>
- Separation of immutable context from mutable state
- Proper lifetime management for LLVM objects
- Use Context::ptr_type() instead of type-specific ptr_type() methods
- Explicit error handling for LLVM operations

## Testing Patterns

- Unit tests in same file as code being tested
- Property-based testing with `proptest` for edge case discovery
- Snapshot testing with `insta` for complex outputs
- Performance benchmarks with `criterion`
- Mutation testing with `cargo-mutants` for critical components
- Test fixtures stored in `tests/fixtures/`
- Unit tests for components
- Integration tests for compiler phases

2025-10-20 04:39:00 - Initial creation of system patterns documentation.
2025-10-20 05:07:00 - Updated architectural patterns and testing patterns.
2025-11-07 22:09:00 - Added patterns identified during LLVM compatibility fixes.
2025-11-07 23:23:00 - Updated architectural patterns with LLVM 18 compatibility and new memory management model.
