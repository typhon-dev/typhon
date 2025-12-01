//! Type definitions for source code representation.
//!
//! This module defines the core types used for representing source code positions, spans,
//! and files in the Typhon compiler. It provides the foundation for accurate source tracking,
//! which is essential for quality error messages and diagnostics.
//!
//! Key types include:
//!
//! - `Position`: Represents a specific location in source code with line, column, and byte offset
//! - `Span` and `SourceSpan`: Track ranges within source files
//! - `SourceFile`: Represents a complete source file with efficient position lookup
//! - `SourceManager`: Manages multiple source files with unique identifiers
//!
//! These types enable precise source location tracking throughout the compilation pipeline.

use std::fmt;
use std::ops::Range;
use std::path::PathBuf;

use rustc_hash::FxHashMap;

/// A unique identifier for a source file.
///
/// `FileID` is a newtype wrapper around `usize` that uniquely identifies
/// a source file within a `SourceManager`. `FileIDs` are assigned by the
/// `SourceManager` when source files are added to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileID(usize);

impl FileID {
    /// Creates a new `FileID` with the given value.
    #[must_use]
    pub const fn new(id: usize) -> Self { Self(id) }

    /// Returns the inner value of the `FileID`.
    #[must_use]
    pub const fn value(&self) -> usize { self.0 }
}

impl fmt::Display for FileID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "file:{}", self.0) }
}

/// A position in a source file.
///
/// Positions are 1-indexed for line and column, following common editor conventions.
/// The `offset` is 0-indexed, representing the byte offset from the start of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset from the start of the file (0-indexed)
    pub offset: usize,
}

impl Position {
    /// Creates a new position with the given line, column, and byte offset.
    #[must_use]
    pub const fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    /// Creates a new position at the start of a file (line 1, column 1, offset 0).
    #[must_use]
    pub const fn start_of_file() -> Self { Self { line: 1, column: 1, offset: 0 } }

    /// Returns true if this position precedes the other position.
    #[must_use]
    pub const fn precedes(&self, other: &Self) -> bool { self.offset < other.offset }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A source code representation that holds a reference to the code.
/// This type can be used where a more lightweight representation is needed.
#[derive(Debug)]
pub struct Source<'source> {
    /// The source code
    pub code: &'source str,
    /// Line start positions (in bytes)
    pub line_starts: Vec<usize>,
}

impl<'source> Source<'source> {
    /// Create a new source from a string
    #[must_use]
    pub fn new(code: &'source str) -> Self {
        let line_starts = Self::compute_line_starts(code);
        Self { code, line_starts }
    }

    /// Computes the byte offsets of all line starts in the content.
    fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut line_starts = vec![0]; // First line always starts at byte 0

        for (i, c) in content.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        line_starts
    }

    /// Get a slice of the source code corresponding to the given span
    #[must_use]
    pub fn slice(&self, span: Span) -> &'source str { &self.code[span.start..span.end] }

    /// Calculate the line and column numbers from a byte offset
    ///
    /// ## Panics
    ///
    /// Panics if the offset is greater than the length of the code.
    #[must_use]
    pub fn get_line_column(&self, offset: usize) -> (usize, usize) {
        assert!(
            offset <= self.code.len(),
            "byte offset {} out of range for code with length {}",
            offset,
            self.code.len()
        );

        // Binary search to find which line this offset is in
        match self.line_starts.binary_search(&offset) {
            // Exact match means it's at the start of a line
            Ok(line) => (line + 1, 1),

            // No exact match, i is the insertion point (which means we're on line i-1)
            Err(line) => {
                let line = line - 1;
                let line_start_offset = self.line_starts[line];
                let column = offset - line_start_offset + 1;

                (line + 1, column)
            }
        }
    }

    /// Get a position from a byte offset
    #[must_use]
    pub fn position_from_offset(&self, offset: usize) -> Position {
        let (line, column) = self.get_line_column(offset);
        Position::new(line, column, offset)
    }

    /// Get a position from a span's start
    #[must_use]
    pub fn position_from_span_start(&self, span: Span) -> Position {
        self.position_from_offset(span.start)
    }

    /// Get a position from a span's end
    #[must_use]
    pub fn position_from_span_end(&self, span: Span) -> Position {
        self.position_from_offset(span.end)
    }

    /// Convert a `Span` to a full Span with positions and `file_id`
    #[must_use]
    pub fn span_with_positions(&self, span: Span, file_id: FileID) -> SourceSpan {
        let start = self.position_from_offset(span.start);
        let end = self.position_from_offset(span.end);

        SourceSpan::new(start, end, file_id)
    }

    /// Get the line of source code containing the given position
    ///
    /// ## Panics
    ///
    /// Panics if the line index is out of bounds.
    #[must_use]
    pub fn line_at_position(&self, position: Position) -> &'source str {
        let line_idx = position.line - 1;
        assert!(line_idx < self.line_starts.len(), "Line index out of bounds");

        let start_offset = self.line_starts[line_idx];

        let end_offset = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1] - 1 // Exclude the newline
        } else {
            self.code.len()
        };

        &self.code[start_offset..end_offset]
    }
}

/// A source file representation.
///
/// Contains the content of the file, its name, and precomputed line start positions
/// for efficient line/column lookup.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Identifier of the file
    pub id: FileID,
    /// Name of the file (usually a path)
    pub name: String,
    /// Path to the file, if available
    pub path: Option<PathBuf>,
    /// Content of the file
    pub content: String,
    /// Byte offsets of line starts (0-indexed, first entry is always 0)
    pub line_starts: Vec<usize>,
}

impl SourceFile {
    /// Creates a new source file with the given ID, name, and content.
    #[must_use]
    pub fn new(id: FileID, name: String, content: String) -> Self {
        let line_starts = Self::compute_line_starts(&content);
        Self { id, name, path: None, content, line_starts }
    }

    /// Creates a new source file with the given ID, name, path, and content.
    #[must_use]
    pub fn with_path(id: FileID, name: String, path: PathBuf, content: String) -> Self {
        let line_starts = Self::compute_line_starts(&content);
        Self { id, name, path: Some(path), content, line_starts }
    }

    /// Computes the byte offsets of all line starts in the content.
    fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut line_starts = vec![0]; // First line always starts at byte 0

        for (i, c) in content.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        line_starts
    }

    /// Converts a byte offset to a Position.
    ///
    /// Returns the Position corresponding to the given byte offset, using binary search
    /// on the `line_starts` array to efficiently find the line number.
    ///
    /// ## Panics
    ///
    /// Panics if the byte offset is greater than the length of the file's content.
    #[must_use]
    pub fn position_from_offset(&self, byte_offset: usize) -> Position {
        assert!(
            byte_offset <= self.content.len(),
            "byte offset {} out of range for file with length {}",
            byte_offset,
            self.content.len()
        );

        // Binary search to find which line this offset is in
        match self.line_starts.binary_search(&byte_offset) {
            // Exact match means it's at the start of a line
            Ok(line) => Position::new(line + 1, 1, byte_offset),

            // No exact match, i is the insertion point (which means we're on line i-1)
            Err(line) => {
                let line = line - 1;
                let line_start_offset = self.line_starts[line];
                let column = byte_offset - line_start_offset + 1;

                Position::new(line + 1, column, byte_offset)
            }
        }
    }

    /// Returns the text at the given span.
    ///
    /// ## Panics
    ///
    /// Panics if the span is not within this file or if the span's range is invalid.
    #[must_use]
    pub fn text_at_span(&self, span: SourceSpan) -> &str {
        assert_eq!(span.file_id, self.id, "Span is from a different file");

        &self.content[span.byte_range()]
    }

    /// Returns the text at the given simple span.
    ///
    /// ## Panics
    ///
    /// Panics if the span's range is invalid.
    #[must_use]
    pub fn text_at_simple_span(&self, span: Span) -> &str { &self.content[span.start..span.end] }

    /// Returns the line of text containing the given position.
    ///
    /// ## Panics
    ///
    /// Panics if the position is not within this file.
    #[must_use]
    pub fn line_at_position(&self, position: Position) -> &str {
        let line_idx = position.line - 1;
        let start_offset = self.line_starts[line_idx];

        let end_offset = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1] - 1 // Exclude the newline
        } else {
            self.content.len()
        };

        &self.content[start_offset..end_offset]
    }
}

/// A unified source information container used throughout the compiler
/// This is for compatibility with the old codebase.
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
    #[must_use]
    pub const fn new(span: Span) -> Self {
        // This is a simple implementation; in a real compiler,
        // you'd compute actual line and column numbers
        Self { span, line: 0, column: 0 }
    }
}

impl Default for SourceInfo {
    fn default() -> Self { Self::new(Span::new(0, 0)) }
}

/// A manager for source files.
///
/// The `SourceManager` keeps track of all source files and assigns unique `FileIDs` to them.
/// It provides methods for adding files, retrieving files by ID, and converting between
/// byte offsets and positions.
#[derive(Debug, Default, Clone)]
pub struct SourceManager {
    /// Map from `FileID` to `SourceFile`
    files: FxHashMap<FileID, SourceFile>,
    /// Next available file ID
    next_id: usize,
}

impl SourceManager {
    /// Creates a new empty `SourceManager`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: FxHashMap::default(),
            next_id: 1, // Start from 1, reserve 0 for dummy spans
        }
    }

    /// Adds a new source file and returns its `FileID`.
    pub fn add_file(&mut self, name: String, content: String) -> FileID {
        let id = FileID::new(self.next_id);
        self.next_id += 1;

        let file = SourceFile::new(id, name, content);
        drop(self.files.insert(id, file));

        id
    }

    /// Adds a new source file with a path and returns its `FileID`.
    pub fn add_file_with_path(&mut self, name: String, path: PathBuf, content: String) -> FileID {
        let id = FileID::new(self.next_id);
        self.next_id += 1;

        let file = SourceFile::with_path(id, name, path, content);
        drop(self.files.insert(id, file));

        id
    }

    /// Returns the source file with the given ID, if it exists.
    #[must_use]
    pub fn get_file(&self, id: FileID) -> Option<&SourceFile> { self.files.get(&id) }

    /// Returns the position corresponding to the given byte offset in the given file.
    #[must_use]
    pub fn position_from_offset(&self, file_id: FileID, byte_offset: usize) -> Option<Position> {
        self.get_file(file_id).map(|file| file.position_from_offset(byte_offset))
    }

    /// Returns the text at the given span, if the file exists.
    #[must_use]
    pub fn text_at_span(&self, span: SourceSpan) -> Option<&str> {
        self.get_file(span.file_id).map(|file| file.text_at_span(span))
    }

    /// Returns the text at the given simple span for the given file.
    #[must_use]
    pub fn text_at_simple_span(&self, file_id: FileID, span: Span) -> Option<&str> {
        self.get_file(file_id).map(|file| file.text_at_simple_span(span))
    }

    /// Returns the line of text containing the given position, if the file exists.
    #[must_use]
    pub fn line_at_position(&self, file_id: FileID, position: Position) -> Option<&str> {
        self.get_file(file_id).map(|file| file.line_at_position(position))
    }
}

/// A span in a source file, representing a range between two positions.
///
/// Spans are used to track the location of language constructs in the source code.
/// They are essential for error reporting, as they allow the compiler to point to
/// specific parts of the source code when reporting errors or warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    /// Starting position of the span
    pub start: Position,
    /// Ending position of the span (exclusive)
    pub end: Position,
    /// File identifier
    pub file_id: FileID,
}

impl SourceSpan {
    /// Creates a new span with the given start and end positions and file ID.
    #[must_use]
    pub const fn new(start: Position, end: Position, file_id: FileID) -> Self {
        Self { start, end, file_id }
    }

    /// Returns the byte range of this span.
    #[must_use]
    pub const fn byte_range(&self) -> Range<usize> { self.start.offset..self.end.offset }

    /// Creates a new span that encompasses both input spans.
    ///
    /// Both spans must be in the same file.
    ///
    /// ## Panics
    ///
    /// Panics if the spans are from different files.
    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        assert_eq!(self.file_id, other.file_id, "Cannot combine spans from different files");

        let start = if self.start.precedes(&other.start) { self.start } else { other.start };
        let end = if self.end.precedes(&other.end) { other.end } else { self.end };

        Self { start, end, file_id: self.file_id }
    }

    /// Returns a new span with the same file ID and start position, but with a different end position.
    #[must_use]
    pub const fn with_end(&mut self, end: Position) -> &mut Self {
        self.end = end;
        self
    }

    /// Returns a new span with the same file ID and start position, but with a different end position.
    #[must_use]
    pub const fn with_start(&mut self, start: Position) -> &mut Self {
        self.start = start;
        self
    }
}

impl Default for SourceSpan {
    fn default() -> Self {
        Self { start: Position::new(0, 0, 0), end: Position::new(0, 0, 0), file_id: FileID(0) }
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.file_id, self.start, self.end)
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        Self::new(Position::new(1, 1, span.start), Position::new(1, 1, span.end), FileID(0))
    }
}

/// A simple span that only contains start and end offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Start offset
    pub start: usize,
    /// End offset
    pub end: usize,
}

impl Span {
    /// Creates a new simple span with the given start and end offsets.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self { Self { start, end } }

    /// Merges two spans, creating a new span that covers both.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }
}

impl Default for Span {
    fn default() -> Self { Self::new(0, 0) }
}

impl From<SourceSpan> for Span {
    fn from(source_span: SourceSpan) -> Self {
        Self::new(source_span.start.offset, source_span.end.offset)
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self { Self { start: range.start, end: range.end } }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self { span.start..span.end }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}
