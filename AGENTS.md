---
title: AGENTS.md
description: Main guidance for AI assistants working with the typhon repository
author: https://github.com/typhon-dev/typhon
version: 1.0
tags: ["agents", "guidance"]
---

This file provides guidance to agents when working with code in this repository.

## Project Overview

Typhon is a statically typed programming language based on Python 3, implemented in Rust with LLVM as the backend.

## Memory Bank Requirements (**MANDATORY - DO NOT SKIP**)

Load the full instructions and detailed requirements from [memory-bank.md](docs/contributing/memory-bank.md).

DO NOT proceed past this point until the memory bank is initialized (if needed) and status is [MEMORY BANK: ACTIVE].

## Coding Standards, Patterns, and Conventions

### Rust Style Guide

See the [Rust standards documentation](docs/contributing/style-guide.md#rust-standards).

### Markdown Style Guide

See the [Markdown standards documentation](docs/contributing/style-guide.md#markdown-standards).

### TOML Style Guide

See the [TOML standards documentation](docs/contributing/style-guide.md#toml-standards).

### YAML Style Guide

See the [YAML standards documentation](docs/contributing/style-guide.md#yaml-standards).

## Testing Patterns

See the [test organization guidelines documentation](docs/contributing/style-guide.md#test-organization-guidelines).

## Non-obvious Implementation Details

- Hybrid memory management: reference counting, cycle detection, escape analysis
- LSP implementation uses document manager and analyzer engine for incremental analysis
- Type system is central and separates inference from checking
- LLVM pipeline for code generation with custom optimizations

## Build Commands

- `cargo clippy` - Checks all packages to catch common mistakes and improve Rust code
- `cargo clippy --package typhon-cli` - Checks a specific package to catch common mistakes and improve Rust code
- `cargo check` - Analyze all packages and report errors, but don't build object files
- `cargo check --package typhon-lsp` - Analyze a specific package and report errors, but don't build object files
- `cargo build` - Build project
- `cargo build --package typhon-repl` - Build a specific package
- `cargo test --package typhon-runtime` - Run tests for a specific component
- `cargo test -- --nocapture` - Run tests with stdout/stderr output

## Project Structure

```shell
typhon/
└── crates/
    ├── typhon-cli/           # Command-line interface
    ├── typhon-compiler/      # Core compiler components
    │   └── src/
    │       ├── driver/       # Compiler driver
    │       ├── backend/      # LLVM IR generation, code generation
    │       ├── frontend/     # Lexer, parser, AST
    │       ├── middleend/    # AST transformations, optimization
    │       └── typesystem/   # Type checking and inference
    ├── typhon-lsp/           # Language Server Protocol implementation
    ├── typhon-repl/          # Interactive REPL
    ├── typhon-runtime/       # Runtime support
    └── typhon-stdlib/        # Standard library
```
