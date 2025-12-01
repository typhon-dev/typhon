---
title: Code and Documentation Standards and Conventions
description: Defines code and documentation standards for this project
tags: [code, standards, documentation, formatting, testing, style]
---

This document defines the coding standards, formatting rules, and testing organization principles for this project.

## Markdown Standards

### Document Structure

- Begin with frontmatter containing the `title`, `description`, and `tags` properties at a minimum
  - Include any other properties that enhance AI agent understanding or further detail the document's intended purpose
  - The frontmatter title serves as the document's heading level 1
- Begin with heading level 2 in the body of the document
- Use proper heading hierarchy (H2 > H3 > H4)
- Structure documents logically
- Full document structure example:

   ```markdown
   ---
   title: Document Structure Example
   description: A brief description of the document
   author: Your Name/Handle
   version: 1.0
   tags: [category, topic, workflow]
   ---

   This document <provide the purpose of the document>

   ## First Sub-Heading
   ```

### Line Breaks and Whitespace

- Break lines at the next word boundary once a length of 100 characters is exceeded
- Break lines at logical points
- Use 2 spaces for indentation
- A maximum of 1 blank line is allowed anywhere in the document
- Include a single blank line between paragraphs
- Include a single blank line before and after:
  - Headings
  - Code blocks
  - Ordered lists
  - Unordered lists
- Include a single blank line at the end of the document

### Lists

- Use hyphen (`-`) for unordered lists
- Use numbers (`1.`, `2.`) for sequential steps
- Indent nested lists 2 spaces
- Be consistent with punctuation

### Code Blocks

- Use fenced code blocks with triple backticks
- It is **REQUIRED** to specify language for syntax highlighting:
  - `rust` (Rust), `json` (JSON data)
  - `shell` (CLI examples), `bash` (shell scripts)
  - `text` (plain text), `sql` (SQL queries)
  - `markdown`, `html`, `toml`, `yaml`/`yml`, `xml`

### Links and References

- Use descriptive link text: `[Descriptive Text](link)`
- Use relative paths for internal links

### Tables

- Column separators must be vertically aligned:

  | Name        | Value             | Description                  |
  | ----------- | ----------------- | ---------------------------- |
  | First Item  | `table-example-1` | The first item in the table  |
  | Second Item | `table-example-1` | The second item in the table |

## Rust Standards

### Formatting and Style

- Follow the linting rules in `clippy.toml`
- Follow the formatting rules in `rustfmt.toml`
- Use blank lines to separate logical blocks of code
- Use Rust 2024 edition style conventions
- Maintain a maximum line length of 100 characters
- Use consistent spacing with 4 spaces for indentation
- Follow naming conventions: snake_case for functions/variables, CamelCase for types/traits

### Import Organization

- Group imports by StdExternalCrate (standard library, external crates, internal crates)
- Use module-level import granularity
- Use vertical layout for imports (one import per line)
- Order imports alphabetically within each group
- Place a blank line between import groups

Example:

```rust
use std::collections::VecDeque;
use std::ops::Range;

use logos::Logos;

use super::token::{Token, TokenKind};
use crate::common::Span;
```

### Code Organization

Module members should be organized in a consistent, logical structure following these principles:

- Organize module members into groups in the following order:
  1. Public modules (`pub mod`)
  2. Private modules
  3. Public constants and statics
  4. Private constants and statics
  5. Public types (`pub trait`, `pub struct`, `pub enum`)
  6. Private types
  7. Public functions
  8. Private functions
  9. Tests (at the end, in a `#[cfg(test)]` module)
- Alphabetize members within each group
- Place methods and implementations immediately after their type definitions
- Use consistent spacing between sections (one blank line)

#### Example file structure

```rust
// Copyright and license header
//! Module documentation

use std::collections::HashMap;

// External crate imports
use some_crate::SomeType;

// Internal imports
use crate::utils::helper;

/// Module documentation
pub mod submodule;

// Private modules
mod private_module;

/// Public constants
pub const MAX_ITEMS: usize = 100;

// Private constants
const INTERNAL_TIMEOUT: u32 = 30;

/// Public trait documentation
pub trait PublicTrait {
    fn method(&self);
}

/// Public struct documentation
pub struct PublicStruct {
    pub field: String,
    internal_field: u32,
}

impl PublicStruct {
    /// Constructor
    pub fn new(value: String) -> Self {
        Self {
            field: value,
            internal_field: 0,
        }
    }

    /// Public method
    pub fn public_method(&self) -> String {
        format!("{}: {}", self.field, self.internal_field)
    }

    // Private method
    fn private_method(&self) -> u32 {
        self.internal_field * 2
    }
}

// Private structs should be defined here
struct PrivateStruct {
    // Fields
}

/// Public function documentation
pub fn public_function() {
    // Implementation
}

// Private functions
fn internal_helper() {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_function() {
        // Test implementation
    }
}
```

### String Formatting

- Use direct variable interpolation in format strings when possible
- Use `write!(f, "{expr}")` instead of `write!(f, "{}", expr)` for simple variables
- For complex expressions or when using formatting options, use the standard approach with curly braces and commas
- Use named parameters for clarity when a format string has many arguments

Examples:

```rust
// Preferred (direct variable interpolation)
write!(f, "{name}: {value}")

// Instead of
write!(f, "{}: {}", name, value)

// For formatting options, use the standard approach
write!(f, "{value:?}") // Debug formatting
write!(f, "{value:#?}") // Pretty debug formatting

// For complex expressions, use standard approach
write!(f, "{}", value.calculate() + offset)
```

### Documentation

- Add comments for all public modules, functions, types, and variables
- End sentences with a period
- Document all parameters and return values
- Include examples for complex functions
- Use consistent casing and terminology

## Test Organization Guidelines

### Test Types and Structure

1. **Unit Tests**
   - Place unit tests in a `#[cfg(test)]` module at the end of each source file
   - Use descriptive test function names prefixed with `test_`
   - Test both success and failure cases

   ```rust
   // Inside src/my_module.rs

   // Code being tested
   pub fn add(a: i32, b: i32) -> i32 {
       a + b
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_add() {
           assert_eq!(4, add(2, 2));
       }
   }
   ```

2. **Integration Tests**
   - Place in `tests/` directory at the project root
   - Each file is compiled as a separate crate
   - Import the library being tested as an external crate

   ```rust
   // Inside tests/integration_test.rs
   use my_crate;

   #[test]
   fn test_external_api() {
       assert_eq!(4, my_crate::add(2, 2));
   }
   ```

3. **Common Test Utilities**
   - Place shared test code in `tests/common/mod.rs`
   - Import in test files with `mod common;`

4. **Documentation Tests**
   - Write executable examples in doc comments using ```rust code blocks
   - Place doc tests in public items to document usage

   ```rust
   /// Adds two numbers together
   ///
   /// ## Examples
   ///
   /// ```
   /// use my_crate::add;
   /// assert_eq!(4, add(2, 2));
   /// ```
   pub fn add(a: i32, b: i32) -> i32 {
       a + b
   }
   ```

### Test Frameworks and Tools

1. **Table-Driven Tests**
   - Use `rstest` for parametrized testing
   - Group related test cases together

   ```rust
   use rstest::rstest;

   #[rstest]
   #[case(0, 0, 0)]
   #[case(1, 0, 1)]
   #[case(1, 1, 2)]
   #[case(5, -5, 0)]
   fn test_add(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
       assert_eq!(expected, add(a, b));
   }
   ```

2. **Property-Based Testing**
   - Use `proptest` for generative testing
   - Test properties that should hold for all inputs

   ```rust
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn test_add_commutative(a in -100..100, b in -100..100) {
           assert_eq!(add(a, b), add(b, a));
       }
   }
   ```

3. **Snapshot Testing**
   - Use `insta` for testing complex outputs
   - Validate outputs against stored snapshots

4. **Performance Testing**
   - Use `criterion` for performance benchmarks
   - Establish baselines and track performance over time

5. **Mutation Testing**
   - Use `cargo-mutants` for critical components
   - Verify test quality by ensuring mutated code is caught by tests

6. **Test Fixtures**
   - Store fixtures in `tests/fixtures/` directory
   - Use fixtures for complex test data and environments

### Testing Best Practices

1. **Naming Conventions**
   - Use descriptive test names that explain the scenario and expectation
   - Follow the pattern `test_<function_name>_<scenario>_<expectation>`

2. **Test Organization**
   - Structure tests using the Arrange-Act-Assert pattern
   - Group related tests together
   - Use sub-modules to organize test categories

3. **Test Coverage**
   - Test both success and error paths
   - Cover edge cases and corner cases
   - Aim for comprehensive but focused test cases

4. **Test Documentation**
   - Add explanatory comments for complex test scenarios
   - Document the purpose of test utilities and fixtures
   - Include references to issues or requirements being tested

## Module Export Conventions

### Module Organization

- Follow a consistent module hierarchy pattern:
  - Top-level crates expose modules with `pub mod` in `lib.rs`
  - Modules use `mod.rs` files to organize and re-export their contents
  - Each module should represent a logical group of related functionality

### Re-export Patterns

#### For Library Crates (`lib.rs`)

- Use `pub mod` to expose all modules intended for external use
- Do NOT use wildcard re-exports (`pub use module::*`) in `lib.rs`
- Keep the external API flat with exactly one module name segment between crate name and member:
  - Users should access as: `crate_name::module_name::MemberName`
  - Example: `typhon_ast::expressions::BinaryOp`

```rust
// Example lib.rs
//! Crate documentation

pub mod expressions;
pub mod declarations;
pub mod statements;
// ... other modules
```

#### For Multi-file Modules (`mod.rs` files)

- Use simple non-public `mod` declarations to include submodule files
- Use wildcard re-exports (`pub use <module>::*`) to expose all public items
- Re-export public items with consistent naming
- Group related items together
- Keep explicit imports for external dependencies at the top

```rust
// Example mod.rs for a multi-file module
//! Module documentation

mod base;
mod specific_item;
mod another_item;

// Re-export all public items
pub use base::*;
pub use specific_item::*;
pub use another_item::*;
```

#### For Sub-modules With Selective Exports

When selective re-exports are needed:

```rust
// Example for selective exports
mod base;
mod utils;
mod implementation;

// Explicit re-exports for specific items
pub use base::{BaseType, SpecificType};
// Wildcard re-exports for modules intended to be fully public
pub use implementation::*;
```

This pattern ensures that:

1. External code accesses members with exactly one module name segment
2. The public API is clean and well-organized
3. Internal module organization remains flexible
4. Code is maintainable and follows a consistent pattern

## Documentation Requirements

- Keep project documentation updated for all components
- Document APIs, interfaces, and configuration options
- Include usage examples and change history
- Follow established documentation structure and format
- Include GoDoc comments for all exported items
- Document parameters, returns, and errors
- Include examples for complex usage
- Document assumptions and side effects

## TOML Standards

### File Structure

- Begin with a comment describing the file's purpose when appropriate
- Organize sections logically, with related configuration options grouped together
- Use table headers (`[section_name]`) to create logical sections
- Use nested tables (`[parent.child]`) for hierarchical configurations
- Place common/important configuration options at the top
- Group related settings within their respective sections

Example:

```toml
[profile.dev]
  debug     = true # Full debug info
  opt-level = 0    # No optimizations for development

[profile.release]
  codegen-units = 1       # Optimize for better runtime performance
  debug         = false   # No debug info in release
  lto           = true    # Link-time optimization
  opt-level     = 3       # Maximum optimizations
  panic         = "abort" # Abort on panic (smaller binary)
```

### Formatting

- Align entries vertically for better readability
- Align comments after entries vertically when appropriate
- Allow a maximum of 1 consecutive blank line between sections
- Use a column width of 100 characters
- Put trailing commas for multiline arrays
- Automatically expand arrays to multiple lines when they exceed column width
- Automatically collapse arrays if they fit in one line
- Reorder keys, arrays, and inline tables alphabetically:

   ```toml
   [workspace.package]
     authors     = ["Author Name <email@example.com>"]
     categories  = ["compilers", "development-tools"]
     description = "Project description"
     edition     = "2024"
     keywords    = ["compiler", "language"]
     license     = "Apache-2.0"
     readme      = "README.md"
     repository  = "https://github.com/username/project"
     version     = "0.1.0"
   ```

- **REQUIRED**: Always use standard table syntax over inline tables/mappings. Nested sections MUST use proper table headers rather than inline mappings
  - Incorrect (do NOT do this):

    ```toml
    [workspace]
      [workspace.dependencies]
        typhon-cli = { path = "crates/typhon-cli" }
    ```

  - Correct (do this instead):

    ```toml
    [workspace]
      [workspace.dependencies]
        [workspace.dependencies.typhon-cli]
          path = "crates/typhon-cli"
    ```

### Indentation Rules

- Indent subtables if they come in order
- Indent entries under tables
- Use 2 spaces for indentation
- Maintain consistent indentation throughout the file
- Add trailing newline at the end of the file

Example:

```toml
[workspace.lints.clippy.pedantic]
  level    = "warn"
  priority = -2

  # Diagnostics are not actionable
  large_stack_arrays = "allow"
```

### Quotation Rules

- Use double quotes for strings containing special characters
- Omit quotes for simple strings when possible
- Always use quotes for keys containing spaces or special characters
- Use consistent quoting style within a file

Example:

```toml
simple = value
special = "value with spaces"
"key with spaces" = "value"
```

## YAML Standards

### File Structure

- All YAML files should start with a comment specifying the URL of the applicable schema, when available

  ```yaml
  # yaml-language-server: $schema=https://schema-url.json
  ---
  key: value
  ```

- Use the YAML document start marker (`---`) after any schema declarations

  ```yaml
  ---
  server:
    host: localhost
    port: 8080
  ```

### Formatting

- Insert a blank line between all top-level object keys

  ```yaml
  server:
    host: 127.0.0.1
    port: 8080

  logging:
    level: info
    format: json
  ```

- Insert a blank line between all list items if the items are objects

  ```yaml
  services:
    - name: authentication
      enabled: true

    - name: email
      enabled: true
  ```

- Do not insert blank lines between simple key-value pairs within the same object
- Alphabetize all object keys within each object level
- Maintain consistent indentation using 2 spaces for each level

### Quotation Rules

- Omit quotes entirely except where explicitly required by YAML syntax
- When quotes are required, use double quotes (`"`) as the preferred format
- Single quotes (`'`) are permitted only when using double quotes would necessitate escaping
- Always use quotes for strings that could be misinterpreted as other YAML types

## Git Conventions

### Commit Message Format

Always use conventional commit messages following the format: `<type>(<scope>): <description>`

- Types:
  - `feat`: A new feature
  - `fix`: A bug fix
  - `docs`: Documentation only changes
  - `style`: Changes that do not affect code meaning
  - `refactor`: Code changes that neither fix bugs nor add features
  - `perf`: Code changes that improve performance
  - `test`: Adding or updating tests
  - `chore`: Changes to build process or tooling

Examples:

```text
feat(email): add support for OAuth authentication
fix: correct URL encoding for special characters
docs: update README with deployment instructions
```

### Extended Commit Format

For complex changes, provide additional context:

```text
<type>(<scope>): <description>

<body>

<footer>
```

- The body should explain the what and why (not the how)
- The footer should reference related issues and breaking changes

### Branching Strategy

- Use feature branches for all changes
- Branch names should follow the pattern: `<type>/<description>`
  - Examples: `feat/oauth-authentication`, `fix/token-refresh-error`
