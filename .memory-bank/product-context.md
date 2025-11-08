---
title: Product Context
description: High-level overview of the project
tags: [memory-bank, documentation, product-context, project-overview, architecture, features]
---

This file provides a high-level overview of the project and the expected product that will be created.
Initially it is based upon project-brief.md (if provided) and all other available project-related
information in the working directory. This file is intended to be updated as the project evolves, and
should be used to inform all other modes of the project's goals and context.

## Project Goal

The Typhon project aims to create a statically typed programming language based on Python 3, implemented
in Rust with LLVM as the backend. This provides the benefits of Python's syntax with the performance
and safety of static typing.

## Key Features

- Python 3 compatible syntax
- Static type checking
- Rust implementation
- LLVM backend for optimized code generation
- LSP integration
- Type inference
- Compiled-only approach

## Overall Architecture

The project is organized into multiple crates:

- typhon-cli: Command-line interface
- typhon-compiler: Core compiler components (frontend, middleend, backend, typesystem)
- typhon-lsp: Language Server Protocol implementation
- typhon-repl: Interactive REPL
- typhon-runtime: Runtime support
- typhon-stdlib: Standard library

The compiler follows a traditional pipeline architecture with lexical analysis, parsing, semantic analysis, optimization, and code generation. The LLVM backend has been updated to support LLVM 18 with a thread-safe memory management model using Arc<Mutex<>> for managing mutable state.

## Compiler Architecture Highlights

- Frontend: Lexer, parser, and AST generation
- Type System: Static typing with inference, central to the compiler
- Middle-end: AST transformations and optimizations
- Backend: LLVM IR generation with LLVM 18 compatibility
- Memory Management: Hybrid approach using reference counting and cycle detection
- Thread-safe design: Separation of immutable context from mutable state

2025-10-20 04:39:00 - Initial creation of product context.
2025-10-20 05:00:00 - Updated key features with LSP integration, type inference, and compiled-only approach.
2025-11-07 23:25:00 - Updated architecture highlights with LLVM 18 compatibility and thread-safe design.
