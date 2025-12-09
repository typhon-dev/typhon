---
title: AGENTS.md
description: Main guidance for AI assistants working with the typhon repository
author: https://github.com/typhon-dev/typhon
version: 1.0
tags: [agents, guidance]
---

This file provides guidance to agents when working with code in this repository.

## Project Overview

Typhon is a statically typed programming language based on Python 3, implemented in Rust with LLVM as the backend.

## Coding Standards, Patterns, and Conventions

### Rust Style Guide

See the [Rust standards documentation](CONVENTIONS.md#rust-standards).

### Markdown Style Guide

See the [Markdown standards documentation](CONVENTIONS.md#markdown-standards).

### TOML Style Guide

See the [TOML standards documentation](CONVENTIONS.md#toml-standards).

### YAML Style Guide

See the [YAML standards documentation](CONVENTIONS.md#yaml-standards).

## Testing Patterns

See the [test organization guidelines documentation](CONVENTIONS.md#test-organization-guidelines).

## Non-obvious Implementation Details

- Hybrid memory management: reference counting, cycle detection, escape analysis
- LSP implementation uses document manager and analyzer engine for incremental analysis
- Type system is central and separates inference from checking
- LLVM pipeline for code generation with custom optimizations

## Build Commands

⚠️ **CRITICAL**: Run all `cargo` commands from the workspace root (use `--package <crate name>` for targeted commands). **DO NOT** attempt to `cd` into sub-packages. ⚠️

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
    ├── typhon-analyzer/      # Semantic analysis infrastructure
    ├── typhon-ast/           # Abstract Syntax Tree (AST) definitions
    ├── typhon-cli/           # Command-line interface
    ├── typhon-compiler/      # Core compiler components
    │   └── src/
    │       ├── backend/      # LLVM IR generation, code generation
    │       └── typesystem/   # Type checking and inference
    ├── typhon-lsp/           # Language Server Protocol implementation
    ├── typhon-parser/        # Lexer, parser
    ├── typhon-repl/          # Interactive REPL
    ├── typhon-runtime/       # Runtime support
    ├── typhon-source/        # Source file handling and position tracking
    └── typhon-stdlib/        # Standard library
```
