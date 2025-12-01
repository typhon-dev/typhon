//! Common types and utilities shared across the compiler.

use std::ops::Range;

/// A span in the source code that supports Copy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub fn merge(&self, other: &Span) -> Self {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

// Implement Default trait for Span
impl Default for Span {
    fn default() -> Self {
        Span::new(0, 0)
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

/// A unified source information container used throughout the compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceInfo {
    /// The span of the source code
    pub span: Span,
    /// The line number
    pub line: usize,
    /// The column number
    pub column: usize,
}

impl SourceInfo {
    /// Create a new source info
    pub fn new(span: Span) -> Self {
        // This is a simple implementation; in a real compiler,
        // you'd compute actual line and column numbers
        SourceInfo {
            span,
            line: 0,
            column: 0,
        }
    }
}

// Implement Default trait for SourceInfo
impl Default for SourceInfo {
    fn default() -> Self {
        SourceInfo::new(Span::new(0, 0))
    }
}
