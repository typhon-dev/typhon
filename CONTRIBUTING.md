# Contributing to Typhon

Thank you for your interest in contributing to Typhon! This guide will help you get started with the development process.

## Development Setup

### Prerequisites

- Rust (stable channel, 1.70.0 or later)
- LLVM 18.1.0 or later
- Cargo and Rustup
- Git

### Getting Started

1. Fork the repository on GitHub
2. Clone your fork:

   ```bash
   git clone https://github.com/your-username/typhon.git
   cd typhon
   ```

3. Install dependencies:

   ```bash
   # On Ubuntu/Debian
   sudo apt-get install llvm-18-dev libclang-18-dev

   # On macOS (using Homebrew)
   brew install llvm@18
   ```

4. Build the project:

   ```bash
   cargo build
   ```

5. Run tests:

   ```bash
   cargo test
   ```

## Project Structure

The Typhon project is organized as follows:

- **compiler/**: Compiler components
  - **frontend/**: Lexer, parser, and AST
  - **middleend/**: AST transformations and optimization
  - **backend/**: LLVM IR generation and code generation
  - **typesystem/**: Type checking and inference
  - **driver/**: Compiler driver and CLI
- **runtime/**: Runtime support library
- **tools/**: Additional tools
  - **lsp/**: Language Server Protocol implementation
- **tests/**: Test suite
- **examples/**: Example Typhon programs
- **docs/**: Documentation

## Coding Guidelines

### Rust Style

- Follow the Rust Style Guide
- Run `cargo fmt` before committing
- Use `cargo clippy` to check for common issues
- Document all public APIs with doc comments

### Testing

- Write unit tests for all new functionality
- Ensure all tests pass with `cargo test`
- Add integration tests for end-to-end functionality
- Use property-based testing where appropriate

### Commit Messages

- Use clear and descriptive commit messages
- Follow the conventional commits format:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation changes
  - `test:` for adding or modifying tests
  - `refactor:` for code refactoring
  - `perf:` for performance improvements
  - `chore:` for maintenance tasks

## Pull Request Process

1. Create a new branch for your feature or bug fix
2. Make your changes and commit them
3. Push to your fork and submit a pull request
4. Wait for CI checks to pass
5. Address any review feedback

## Code of Conduct

Please follow our Code of Conduct when participating in the Typhon community. We aim to foster an inclusive and respectful environment for all contributors.

## License

By contributing to Typhon, you agree that your contributions will be licensed under the project's Apache 2.0 license.
