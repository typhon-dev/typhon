# 2. Unified CLI Toolchain with Cargo-Style Subcommands

Date: 2025-12-07

## Status

Proposed

## Context

Typhon is implementing a unified CLI toolchain to replace its minimal, non-functional implementation. The language currently has separate binaries for different functionality (`typhon-cli`, `typhon-repl`, `typhon-lsp`). This fragmented approach creates several problems:

1. **Installation Complexity**: Users must install and manage multiple binaries, increasing friction for adoption
2. **Inconsistent UX**: Each tool may have different argument parsing, error formatting, and behavior patterns
3. **Version Skew**: Separate binaries can get out of sync, leading to compatibility issues
4. **Missing Developer Tools**: No built-in formatter, linter, test runner, or watch mode, forcing users to install external tools
5. **Slow Feedback Loops**: Full compilation is required even for simple type checking, slowing iterative development
6. **Python Developer Expectations**: Python developers expect integrated tooling (like Rye) but get fragmented utilities
7. **Compilation Bugs**: Current CLI has broken compilation path preventing basic usage

Modern language toolchains (Cargo for Rust, Deno for TypeScript, Go toolchain, Rye for Python) demonstrate that unified, single-binary approaches with subcommands provide superior developer experience. Research shows key patterns:

- **Cargo**: Single entry point, fast feedback (`cargo check`), built-in quality tools, intuitive subcommands
- **Rye**: Python ecosystem familiarity, project management, zero-config start
- **Deno**: Built-in formatter/linter/test runner, no external dependencies needed
- **Go**: Integrated tooling, fast compilation, simple command structure

Typhon needs to evolve from a simple compiler into a complete development toolchain to compete with established languages and meet modern developer expectations.

## Decision

We will redesign the Typhon CLI as a unified toolchain with the following architecture:

### 1. Unified CLI with Subcommands (Except LSP)

Replace fragmented binaries with one `typhon` executable containing core development functionality as subcommands:

```shell
typhon          # Launch REPL (default behavior)
typhon <file>   # Run file (shortcut for 'typhon run <file>')
typhon build    # Compile to executable
typhon run      # Compile and execute
typhon check    # Fast type checking only (no codegen)
typhon watch    # Auto-recompile on changes
typhon test     # Run tests
typhon fmt      # Format code
typhon lint     # Check code quality
typhon doc      # Generate documentation
typhon new      # Create new project
typhon init     # Initialize existing directory
```

**Note:** The Language Server Protocol (LSP) implementation remains a separate `typhon-lsp` binary for deployment flexibility, binary size optimization, and separation of concerns.

### 2. Three-Phase Implementation

**Phase 1 (MVP)**: Core compilation and base structure

- Unified CLI with subcommand structure
- Default REPL behavior (`typhon` launches REPL)
- File shortcut (`typhon file.ty` runs file)
- Integrate `typhon-repl` library
- All command stubs in organized structure
- Implement `build`, `run`, `check` command logic
- Fix compilation path

**Phase 2 (Essential)**: Developer workflow tools

- Watch mode for auto-recompilation
- Code formatter (Black-inspired)
- Linter with auto-fix
- Test framework and runner
- Project scaffolding (`new`, `init`)

**Phase 3 (Advanced)**: Production features

- Documentation generator
- Cross-compilation support
- Code coverage and benchmarks
- Workspace support

### 3. Fast Check Command

Implement `typhon check` that performs type checking without LLVM code generation, providing 10-100x faster feedback for iterative development. This addresses the critical need for fast feedback loops.

### 4. Built-in Quality Tools

Include formatter, linter, test runner, and documentation generator without requiring external dependencies. This eliminates tool fragmentation and ensures consistent experience.

### 5. Project Management

Add `typhon new` and `typhon init` commands with standard project structure and `typhon.toml` manifest, following Cargo's model for familiar, zero-config project setup.

### 6. Design Principles

- **Cargo-Inspired Simplicity**: Single tool, consistent UX, sensible defaults
- **Fast Feedback**: Prioritize quick check/validation over full compilation
- **Python Familiarity**: Use terminology and patterns Python developers expect
- **Zero-Config Start**: Work immediately for simple projects, scale to complex ones
- **Progressive Disclosure**: Simple commands for beginners, powerful flags for experts

### 7. REPL as Default with File Shortcut

- `typhon` (no arguments) launches interactive REPL
- `typhon <file>` is shorthand for `typhon run <file>`
- Mimics Python's behavior (`python3` launches REPL, `python3 script.py` runs script)
- No existing users means no backward compatibility concerns - clean slate design

### 8. LSP Separation Rationale

The LSP server remains a separate `typhon-lsp` binary because:

- **Binary Size**: LSP has heavy dependencies (tower-lsp, tokio async runtime)
- **Deployment Flexibility**: IDEs can install/update LSP independently
- **Separation of Concerns**: LSP is infrastructure, not a direct developer tool
- **Different Use Pattern**: Called by editors, not developers directly

## Consequences

### Positive Consequences

1. **Simplified Installation**: Single binary to download and install, no dependency management
2. **Consistent UX**: Uniform argument parsing, error messages, and behavior across all commands
3. **Version Coherence**: All tools guaranteed compatible, shipped together
4. **Fast Feedback**: `check` command provides near-instant type checking for iterative development
5. **Complete Toolchain**: All essential tools built-in (fmt, lint, test, doc) without external dependencies
6. **Better Onboarding**: `typhon new` creates working projects with best practices out of the box
7. **Familiar Patterns**: Cargo-style commands feel natural to Rust devs, project structure familiar to Python devs
8. **Watch Mode**: Auto-recompilation eliminates manual rebuild steps during development
9. **Improved Marketing**: "Complete toolchain in one binary" is compelling for adoption
10. **Future-Proof**: Architecture supports adding new commands as needs emerge

### Negative Consequences

1. **Initial Development Effort**: Significant work to implement full toolchain (6-10 weeks total)
2. **Maintenance Burden**: We own all tools rather than relying on ecosystem (but ensures quality and integration)
3. **Learning Curve**: Users need to learn subcommand structure (mitigated by familiar Cargo patterns and Python-like defaults)
4. **LSP Separate Binary**: Users must install two binaries if using IDE features (mitigated by package managers)

### Neutral Consequences

1. **Configuration Format**: TOML chosen over Python-native formats (can add `pyproject.toml` compatibility later)
2. **Black-Style Formatting**: Opinionated formatter reduces configurability but eliminates style debates
3. **Built-in Test Framework**: Less flexibility than ecosystem approach but ensures consistency
4. **Crate Structure**: New crates needed (`typhon-fmt`, `typhon-lint`, `typhon-doc`, `typhon-test`)

### What Becomes Easier

- **Getting Started**: Download one binary, run `typhon new myapp`, start coding
- **Daily Development**: `typhon watch check` provides instant feedback on every save
- **Code Quality**: Built-in `fmt` and `lint` with auto-fix eliminate manual formatting
- **Testing**: Integrated test runner with parallel execution and familiar patterns
- **Distribution**: Users install one tool, not multiple, simplifying deployment documentation
- **CI Integration**: Single binary with all tools simplifies CI setup

### What Becomes More Difficult

- **LSP Installation**: Separate binary means two things to install (but common in language ecosystems)
- **Third-Party Tools**: Harder to replace built-in tools with alternatives (can add plugin system later)
- **Initial Implementation**: More upfront work than maintaining minimal compiler
- **Binary Size**: Including all dev tools increases binary size (acceptable trade-off)

### Risk Mitigation

- **Incremental Implementation**: Three phases allow testing and feedback before full completion
- **Clean Slate Advantage**: No existing users means no backward compatibility constraints
- **Clear Documentation**: Comprehensive help text and examples for every command
- **Modular Architecture**: Organized command structure enables independent development
- **Python-Familiar Defaults**: REPL default behavior and file shortcuts reduce learning curve
