//! Diagnostics and error reporting module.
//!
//! This module provides types and functions for reporting and formatting
//! diagnostic messages, such as errors, warnings, and notes. It includes:
//!
//! - `DiagnosticLevel`: Enum for categorizing diagnostics by severity
//! - `Diagnostic`: Struct representing a diagnostic message with source location
//! - `DiagnosticReporter`: Struct for collecting and formatting diagnostics
//! - `ParserError`: Error types that can occur during parsing
//!
//! The diagnostics system is designed to provide rich, contextual error messages
//! similar to those produced by rustc, with source code snippets, underlines,
//! and helpful suggestions.

mod error;
mod reporter;

// Re-export public types
pub use error::{
    Diagnostic,
    DiagnosticLevel,
    LexError,
    LexErrorBuilder,
    LexErrorKind,
    ParseErrorBuilder,
    ParseResult,
    ParseError,
};
pub use reporter::{DiagnosticReporter, format_error_context, format_with_line_numbers};
use typhon_source::types::SourceSpan;

/// Creates an "expected X, found Y" diagnostic
#[must_use]
pub fn expected_found_error(expected: &str, found: &str, span: SourceSpan) -> Diagnostic {
    Diagnostic::error(format!("Expected {expected}, found {found}"), span)
        .with_suggestion(format!("Try using {expected} here"))
}

/// Creates a "unexpected end of file" diagnostic
#[must_use]
pub fn unexpected_eof_error(expected: &str, span: SourceSpan) -> Diagnostic {
    Diagnostic::error(format!("Unexpected end of file, expected {expected}"), span)
}

/// Creates a "missing X" diagnostic
#[must_use]
pub fn missing_error(missing: &str, span: SourceSpan) -> Diagnostic {
    Diagnostic::error(format!("Missing {missing}"), span)
        .with_suggestion(format!("Add {missing} here"))
}

/// Creates an "invalid X" diagnostic
#[must_use]
pub fn invalid_error(item: &str, reason: &str, span: SourceSpan) -> Diagnostic {
    Diagnostic::error(format!("Invalid {item}: {reason}"), span)
}

/// Creates a "redefinition" diagnostic
#[must_use]
pub fn redefinition_error(name: &str, span: SourceSpan, original_span: SourceSpan) -> Diagnostic {
    Diagnostic::error(format!("Redefinition of '{name}'"), span)
        .with_note(format!("'{name}' was previously defined at {original_span}"))
}

/// Creates an "undefined" diagnostic
#[must_use]
pub fn undefined_error(kind: &str, name: &str, span: SourceSpan) -> Diagnostic {
    Diagnostic::error(format!("Undefined {kind}: '{name}'"), span)
}
