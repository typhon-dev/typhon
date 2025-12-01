//! Diagnostic reporting system for the Typhon parser.
//!
//! This module provides the `DiagnosticReporter` struct, which is responsible for
//! collecting, formatting, and emitting diagnostic messages.

use std::fmt::Write as _;
use std::io::{self, Write};
use std::sync::Arc;

use typhon_source::types::{SourceManager, SourceSpan};

use super::error::{Diagnostic, DiagnosticLevel};

/// Collects and formats diagnostic messages.
///
/// The `DiagnosticReporter` is responsible for collecting diagnostic messages
/// during parsing and other compiler stages, and formatting them for
/// presentation to the user.
#[derive(Debug, Clone)]
pub struct DiagnosticReporter {
    /// Source manager for looking up spans
    source_manager: Arc<SourceManager>,
    /// Collection of diagnostics
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReporter {
    /// Create a new diagnostic reporter with the given source manager.
    #[must_use]
    pub const fn new(source_manager: Arc<SourceManager>) -> Self {
        Self { source_manager, diagnostics: Vec::new() }
    }

    /// Add a diagnostic to the collection.
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) { self.diagnostics.push(diagnostic); }

    /// Report a parser error.
    pub fn error<E>(&mut self, error: E) -> &mut Self
    where E: Into<Diagnostic> {
        self.add_diagnostic(error.into());
        self
    }

    /// Report a warning.
    pub fn warning(&mut self, message: String, span: SourceSpan) -> &mut Self {
        self.add_diagnostic(Diagnostic::warning(message, span));
        self
    }

    /// Report an info message.
    pub fn info(&mut self, message: String, span: SourceSpan) -> &mut Self {
        self.add_diagnostic(Diagnostic::info(message, span));
        self
    }

    /// Report a note.
    pub fn note(&mut self, message: String, span: SourceSpan) -> &mut Self {
        self.add_diagnostic(Diagnostic::note(message, span));
        self
    }

    /// Check if there are any error-level diagnostics.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error)
    }

    /// Get the number of diagnostics.
    #[must_use]
    pub const fn len(&self) -> usize { self.diagnostics.len() }

    /// Check if there are no diagnostics.
    #[must_use]
    pub const fn is_empty(&self) -> bool { self.diagnostics.is_empty() }

    /// Get the collected diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] { &self.diagnostics }

    /// Clear all diagnostics.
    pub fn clear(&mut self) { self.diagnostics.clear(); }

    /// Format and return all diagnostics as a string.
    #[must_use]
    pub fn emit(&self) -> String {
        let mut output = String::new();

        for diagnostic in &self.diagnostics {
            self.format_diagnostic(&mut output, diagnostic);
        }

        output
    }

    /// Print all diagnostics to the given writer.
    ///
    /// ## Errors
    ///
    /// Returns an I/O error if writing to the output writer fails.
    pub fn print<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for diagnostic in &self.diagnostics {
            self.print_diagnostic(writer, diagnostic)?;
        }

        Ok(())
    }

    /// Format a single diagnostic and append it to the given string.
    fn format_diagnostic(&self, output: &mut String, diagnostic: &Diagnostic) {
        let file = self.source_manager.get_file(diagnostic.span.file_id);

        if let Some(file) = file {
            let color = diagnostic.level.color_code();
            let reset = DiagnosticLevel::reset_code();

            // Format the header line
            if let Some(code) = &diagnostic.code {
                let _ = writeln!(
                    output,
                    "{}{}{} [{}]: {} at {}:{}:{}",
                    color,
                    diagnostic.level,
                    reset,
                    code,
                    diagnostic.message,
                    file.name,
                    diagnostic.span.start.line,
                    diagnostic.span.start.column
                );
            } else {
                let _ = writeln!(
                    output,
                    "{}{}{}: {} at {}:{}:{}",
                    color,
                    diagnostic.level,
                    reset,
                    diagnostic.message,
                    file.name,
                    diagnostic.span.start.line,
                    diagnostic.span.start.column
                );
            }

            // Format source code snippet
            if let Some(line_text) =
                self.source_manager.line_at_position(diagnostic.span.file_id, diagnostic.span.start)
            {
                // Format line number
                let _ = writeln!(output, "{:>4} | {}", diagnostic.span.start.line, line_text);

                // Format error underline - calculate the correct spacing for the underline
                let column = diagnostic.span.start.column;
                let underline_spaces = " ".repeat(column - 1); // Adjust for 1-indexed columns

                // Calculate the underline length based on the span
                let underline_length = if diagnostic.span.start.line == diagnostic.span.end.line {
                    // For single-line spans, use the actual range
                    (diagnostic.span.end.column - diagnostic.span.start.column).max(1)
                } else {
                    // For multi-line spans, underline to the end of this line
                    line_text.len() - (column - 1)
                };

                let underline = "^".repeat(underline_length);

                let _ = writeln!(output, "     | {underline_spaces}{color}{underline}{reset}");
            }

            // Format notes
            for note in &diagnostic.notes {
                let _ = writeln!(output, "note: {note}");
            }

            // Format suggestions
            for suggestion in &diagnostic.suggestions {
                let _ = writeln!(output, "suggestion: {suggestion}");
            }

            // Add a newline between diagnostics
            output.push('\n');
        } else {
            // If we don't have the file, fall back to a simpler format
            let _ = writeln!(output, "{}: {}", diagnostic.level, diagnostic.message);
        }
    }

    /// Print a single diagnostic to the given writer.
    fn print_diagnostic<W: Write>(
        &self,
        writer: &mut W,
        diagnostic: &Diagnostic,
    ) -> io::Result<()> {
        let mut formatted = String::new();
        self.format_diagnostic(&mut formatted, diagnostic);
        write!(writer, "{formatted}")
    }

    /// Generate a rich error message with context for the given diagnostic.
    ///
    /// This method generates a detailed error message that includes:
    /// - The source code snippet around the error
    /// - Underlines and arrows pointing to the exact location
    /// - Explanatory notes and suggestions
    #[must_use]
    pub fn rich_message(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();
        self.format_diagnostic(&mut output, diagnostic);
        output
    }

    /// Format diagnostics in a style similar to rustc.
    ///
    /// This produces a format that closely resembles rustc's error output:
    /// ```text
    /// error[E0001]: unexpected token
    ///   --> file.ty:10:5
    ///    |
    /// 10 |     let x = 1 + ;
    ///    |                 ^ expected expression
    ///    |
    /// ```
    #[must_use]
    pub fn format_rustc_style(&self) -> String {
        let mut output = String::new();

        for diagnostic in &self.diagnostics {
            if let Some(file) = self.source_manager.get_file(diagnostic.span.file_id) {
                let color = diagnostic.level.color_code();
                let reset = DiagnosticLevel::reset_code();

                // Header line
                if let Some(code) = &diagnostic.code {
                    let _ = writeln!(
                        output,
                        "{}{}{}[{}]: {}",
                        color, diagnostic.level, reset, code, diagnostic.message
                    );
                } else {
                    let _ = writeln!(
                        output,
                        "{}{}{}: {}",
                        color, diagnostic.level, reset, diagnostic.message
                    );
                }

                // File location line
                let _ = writeln!(
                    output,
                    "  --> {}:{}:{}",
                    file.name, diagnostic.span.start.line, diagnostic.span.start.column
                );

                // Empty line with pipe
                let _ = writeln!(output, "   |");

                // Source line with line number
                if let Some(line_text) = self
                    .source_manager
                    .line_at_position(diagnostic.span.file_id, diagnostic.span.start)
                {
                    let _ = writeln!(output, "{:>3} | {}", diagnostic.span.start.line, line_text);

                    // Underline line
                    let column = diagnostic.span.start.column;
                    let underline_spaces = " ".repeat(column - 1);
                    let underline_length = if diagnostic.span.start.line == diagnostic.span.end.line
                    {
                        (diagnostic.span.end.column - diagnostic.span.start.column).max(1)
                    } else {
                        line_text.len() - (column - 1)
                    };

                    let underline = "^".repeat(underline_length);

                    let _ = writeln!(
                        output,
                        "    | {}{}{}{} {}",
                        underline_spaces, color, underline, reset, diagnostic.message
                    );
                }

                // Empty line with pipe
                output.push_str("   |\n");

                // Notes
                for note in &diagnostic.notes {
                    let _ = writeln!(output, "   = note: {note}");
                }

                // Suggestions
                for suggestion in &diagnostic.suggestions {
                    let _ = writeln!(output, "   = suggestion: {suggestion}");
                }
            } else {
                // Fallback format if file isn't available
                let _ = writeln!(output, "{}: {}", diagnostic.level, diagnostic.message);
            }

            // Empty line between diagnostics
            output.push('\n');
        }

        output
    }
}

/// Formats a multiline error message with line indicators.
///
/// This utility function takes a multiline string and formats it with line numbers
/// and pipes to create a more readable error message similar to rustc's output.
#[must_use]
pub fn format_with_line_numbers(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut result = String::new();

    for (i, line) in lines.iter().enumerate() {
        let _ = writeln!(result, "{:>3} | {}\n", i + 1, line);
    }

    result
}

/// Helper function to create an error context around a code snippet.
///
/// This function takes a source text, a span, and the number of context lines
/// to include before and after the error, and returns a formatted string with
/// line numbers and the error location highlighted.
#[must_use]
pub fn format_error_context(
    _source: &str,
    span: SourceSpan,
    context_lines: usize,
    source_manager: &SourceManager,
) -> String {
    let Some(file) = source_manager.get_file(span.file_id) else {
        return "Error: Source file not found".to_string();
    };
    let start_line = span.start.line;
    let end_line = span.end.line;
    let first_line = start_line.saturating_sub(context_lines);
    let last_line = std::cmp::min(end_line + context_lines, file.line_starts.len());
    let mut result = String::new();

    // Add header
    let _ = writeln!(result, "--> {}:{}:{}", file.name, start_line, span.start.column);
    result.push_str("   |\n");

    // Add context lines before the error
    for line_num in first_line..start_line {
        if let Some(line) = source_manager.line_at_position(
            span.file_id,
            file.position_from_offset(file.line_starts[line_num - 1]),
        ) {
            let _ = writeln!(result, "{line_num:>3} | {line}");
        }
    }

    // Add error lines
    for line_num in start_line..=end_line {
        if line_num > file.line_starts.len() {
            break;
        }

        if let Some(line) = source_manager.line_at_position(
            span.file_id,
            file.position_from_offset(file.line_starts[line_num - 1]),
        ) {
            writeln!(result, "{line_num:>3} | {line}").unwrap();

            // Add underline for the error
            if line_num == start_line {
                let column = span.start.column;
                let underline_spaces = " ".repeat(column - 1);
                let underline_length = if start_line == end_line {
                    (span.end.column - span.start.column).max(1)
                } else {
                    line.len() - (column - 1)
                };

                let underline = "^".repeat(underline_length);
                let _ = writeln!(result, "    | {underline_spaces}{underline}");
            }
        }
    }

    // Add context lines after the error
    for line_num in (end_line + 1)..=last_line {
        if let Some(line) = source_manager.line_at_position(
            span.file_id,
            file.position_from_offset(file.line_starts[line_num - 1]),
        ) {
            let _ = writeln!(result, "{line_num:>3} | {line}");
        }
    }

    result.push_str("   |\n");

    result
}
