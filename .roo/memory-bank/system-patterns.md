---
description: Documents recurring patterns and standards used in the Typhon project
tags: ["memory-bank", "documentation", "system-patterns", "coding-patterns", "architectural-patterns", "testing-patterns", "typhon"]
---
# System Patterns

This file documents recurring patterns and standards used in the project.

## Coding Patterns

- Imports must be grouped by StdExternalCrate (not by default)
- Imports use vertical layout (not mixed)
- Doc comments can use special identifiers (CPython, FastAPI, etc.)
- Rust 2024 edition style, modular crate structure with clear interfaces

## Architectural Patterns

- Crate-based modularization
- Clear separation between compiler stages
- Type system as central component
- Pipeline-based compiler design with distinct phases

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
