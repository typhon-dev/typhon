//! Canned Typhon source snippets shared across test files.
//!
//! Keeping these in one place avoids drift between tests that exercise the
//! same construct from different angles (e.g. token kinds vs. spans).

/// A two-line indented block: `if True:` followed by `    pass`.
pub const IF_TRUE_PASS: &str = "if True:\n    pass";

/// A nested two-level indented block.
pub const NESTED_INDENT: &str = "if True:\n    if False:\n        pass\n";

/// A function call split across lines via implicit bracket continuation.
pub const IMPLICIT_BRACKET_CONTINUATION: &str = "f(\n    1,\n    2,\n)";

/// An expression split across lines with an explicit backslash.
pub const EXPLICIT_BACKSLASH_CONTINUATION: &str = "1 + \\\n2";

/// Two adjacent string literals that should be lexed as a single combined
/// [`StringLiteral`](typhon_lexer::TokenKind::StringLiteral) token via
/// implicit string concatenation.
pub const IMPLICIT_STRING_CONCAT: &str = "\"hello \" \"world\"";

/// A block whose dedent is forced by end-of-file rather than an outdented
/// line.
pub const EOF_TERMINATED_DEDENT: &str = "if True:\n    pass";
