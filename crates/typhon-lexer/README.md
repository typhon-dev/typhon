# typhon-lexer

Lexer for the Typhon programming language.

This crate converts Typhon source code into a stream of tokens. It uses the
[`logos`](https://crates.io/crates/logos) crate for fast tokenization and adds Python-style
indentation handling on top: tracking `INDENT`/`DEDENT` tokens, implicit and explicit line
continuations, implicit string concatenation, soft keywords, and tab/space warnings.

The lexer is intentionally free of any dependency on the parser's diagnostics module:
diagnostics are accumulated locally on the lexer and drained by the consumer (typically the
parser) between tokens. This keeps the dependency graph acyclic so the lexer can be reused
by tools (LSP, formatter, REPL) without pulling in the full parser.

## Architecture

The crate is organized into the following modules:

- **[`lexer`](src/lexer.rs)**: Core [`Lexer`](src/lexer.rs) implementation
  - Wraps a `logos::Lexer` and adds Python-style indentation tracking
  - Accumulates [`LexError`](src/error.rs) and [`LexWarning`](src/error.rs) values for
    consumer drain via `Lexer::take_errors` / `Lexer::take_warnings`
- **[`token`](src/token.rs)**: Token definitions
  - [`Token`](src/token.rs): A lexed token with span and lexeme
  - [`TokenKind`](src/token.rs): Logos-derived enum of all token variants
  - [`BracketType`](src/token.rs): Open/close bracket classification
- **[`rules`](src/rules.rs)**: Helper rules used by the lexer
  - Soft keyword recognition (`match`, `case`, `type`, `_`)
  - String literal classification and implicit-concatenation joining
  - Line continuation and template-string interpolation detection
- **[`error`](src/error.rs)**: Lexer diagnostic types
  - [`LexError`](src/error.rs) and [`LexErrorKind`](src/error.rs)
  - [`LexErrorBuilder`](src/error.rs) fluent constructor
  - [`LexWarning`](src/error.rs) for non-fatal lexer diagnostics

## Usage

```rust,ignore
use typhon_lexer::{Lexer, TokenKind};
use typhon_source::types::FileID;

let mut lexer = Lexer::new("def greet(): ...", FileID::new(1));
let tokens: Vec<TokenKind> = (&mut lexer).map(|t| t.kind).collect();
let errors = lexer.take_errors();
let warnings = lexer.take_warnings();
```

Consumers convert [`LexError`](src/error.rs) / [`LexWarning`](src/error.rs) values to their own
diagnostic types via `From` impls (see `typhon-parser`'s `Diagnostic` for a reference
implementation).
