# Typhon Documentation

This directory contains documentation for the Typhon programming language, its compiler, and tools.

## Documentation Structure

- **compiler/**: Documentation for the Typhon compiler architecture and implementation
  - **frontend/**: Lexer, parser, and syntax handling
    - **parser.md**: Details on the new parsing approach using ASTParser trait
  - **typesystem/**: Type system and type checking
  - **backend/**: Code generation and LLVM integration
  - **optimization/**: Compiler optimization passes
- **dev/**: Documentation for Typhon developers
  - **contributing.md**: Contribution guidelines
  - **architecture.md**: High-level architecture overview
  - **testing.md**: Testing guidelines and procedures
- **language/**: Typhon language documentation
  - **reference/**: Language reference manual
  - **tutorials/**: Tutorials and guides
  - **examples/**: Example programs
- **lsp/**: Language Server Protocol implementation documentation
- **tools/**: Documentation for Typhon tools
  - **formatter/**: Code formatter
  - **repl/**: Interactive REPL

## Main Documents

- [Language Specification](./language/specification.md): Complete Typhon language specification
- [Compiler Architecture](./compiler/architecture.md): Overview of the Typhon compiler design
- [LSP Implementation Plan](./lsp/implementation-plan.md): Design and implementation plan for Typhon LSP
- [Parser Implementation](./compiler/frontend/parser.md): Details on the new parsing approach using ASTParser trait

## Contributing to Documentation

When adding new documentation:

1. Place documents in the appropriate subdirectory
2. Use clear, descriptive file names
3. Include diagrams when they help clarify concepts
4. Link related documents to create a cohesive documentation structure
