//! Document management for the Typhon Language Server.

use std::collections::HashMap;
use std::ops::Range as StdRange;

use ropey::Rope;
use tower_lsp::lsp_types::{
    Position,
    Range,
    Url,
};

/// Represents a text document in the editor.
pub struct Document {
    /// URI of the document
    uri: Url,
    /// Version of the document
    version: i32,
    /// Document content as a rope data structure for efficient editing
    content: Rope,
}

impl Document {
    /// Creates a new document with the given URI, text, and version.
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        Self {
            uri,
            version,
            content: Rope::from_str(&text),
        }
    }

    /// Returns the URI of the document.
    pub fn uri(&self) -> &Url {
        &self.uri
    }

    /// Returns the current version of the document.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Returns the text of the document.
    pub fn text(&self) -> &str {
        self.content.as_str().unwrap_or("")
    }

    /// Updates the document with a change at the given range.
    pub fn update(&mut self, range: Range, text: String, version: i32) {
        // Update version
        self.version = version;

        // Get the start and end byte offsets
        let start_offset = self.position_to_byte_offset(range.start);
        let end_offset = self.position_to_byte_offset(range.end);

        // Delete the range and insert the new text
        if let (Some(start), Some(end)) = (start_offset, end_offset) {
            let byte_range = start..end;
            self.content.remove(byte_range);
            self.content.insert(start, &text);
        }
    }

    /// Replaces the entire document content.
    pub fn replace(&mut self, text: String, version: i32) {
        // Update version
        self.version = version;
        // Replace content
        self.content = Rope::from_str(&text);
    }

    /// Converts a Position (line, character) to a byte offset in the document.
    pub fn position_to_byte_offset(&self, position: Position) -> Option<usize> {
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= self.content.len_lines() {
            return None;
        }

        let line = self.content.line(line_idx);
        if char_idx > line.len_chars() {
            return None;
        }

        let byte_offset = line.char_to_byte(char_idx);
        let line_start_byte = self.content.line_to_byte(line_idx);
        Some(line_start_byte + byte_offset)
    }

    /// Converts a byte offset to a Position (line, character) in the document.
    pub fn byte_offset_to_position(&self, offset: usize) -> Position {
        let line_idx = self.content.byte_to_line(offset);
        let line = self.content.line(line_idx);
        let line_start_byte = self.content.line_to_byte(line_idx);
        let char_idx = line.byte_to_char(offset - line_start_byte);

        Position::new(line_idx as u32, char_idx as u32)
    }

    /// Converts a span (byte range) to an LSP range.
    pub fn range_from_span(&self, span: StdRange<usize>) -> Range {
        let start = self.byte_offset_to_position(span.start);
        let end = self.byte_offset_to_position(span.end);
        Range::new(start, end)
    }
}

/// Manages documents in the LSP server.
pub struct DocumentManager {
    /// Map of document URIs to Document instances
    documents: HashMap<Url, Document>,
}

impl DocumentManager {
    /// Creates a new document manager.
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Adds a document to the manager.
    pub fn add_document(&mut self, uri: Url, text: String, version: i32) {
        self.documents
            .insert(uri.clone(), Document::new(uri, text, version));
    }

    /// Removes a document from the manager.
    pub fn remove_document(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Gets a document by URI.
    pub fn get_document(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }

    /// Updates a document with a change at the given range.
    pub fn update_document(&mut self, uri: &Url, range: Range, text: String, version: i32) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.update(range, text, version);
        }
    }

    /// Replaces the entire content of a document.
    pub fn replace_document(&mut self, uri: &Url, text: String, version: i32) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.replace(text, version);
        }
    }

    /// Returns the list of all document URIs.
    pub fn document_uris(&self) -> Vec<Url> {
        self.documents.keys().cloned().collect()
    }
}
