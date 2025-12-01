//! Utility functions for the Typhon Language Server.

use std::path::Path;

use tower_lsp::lsp_types::{Position, Range, Url};

/// Converts a `Span` to an LSP `Range`.
///
/// This function is used to convert compiler token spans to LSP ranges
/// for reporting diagnostics and other features.
pub fn span_to_range(source: &str, span: &Span) -> Range {
    let start_pos = byte_offset_to_position(source, span.start);
    let end_pos = byte_offset_to_position(source, span.end);

    Range::new(start_pos, end_pos)
}

/// Convert a byte offset in the source to an LSP position.
///
/// This function calculates the line and character position for a given byte offset
/// in the source text.
pub fn byte_offset_to_position(source: &str, offset: usize) -> Position {
    let offset = std::cmp::min(offset, source.len());
    let mut line = 0;
    let mut character = 0;
    let mut current_offset = 0;

    for c in source.chars().take(offset) {
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
        current_offset += c.len_utf8();
    }

    Position::new(line, character as u32)
}

/// Convert an LSP position to a byte offset in the source.
///
/// This function calculates the byte offset for a given line and character position
/// in the source text.
pub fn position_to_byte_offset(source: &str, position: &Position) -> Option<usize> {
    let lines: Vec<&str> = source.lines().collect();

    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    if position.character as usize > line.len() {
        return None;
    }

    // Calculate byte offset by adding up all previous lines plus the character offset
    let mut offset = 0;

    // Add bytes for all previous lines (including newlines)
    for i in 0..position.line as usize {
        offset += lines[i].len() + 1; // +1 for the newline
    }

    // Add byte offset for the characters in the current line
    let mut char_count = 0;
    for c in line.chars() {
        if char_count >= position.character as usize {
            break;
        }
        offset += c.len_utf8();
        char_count += 1;
    }

    Some(offset)
}

/// Converts a file path to an LSP URI.
pub fn path_to_uri(path: &Path) -> Url {
    Url::from_file_path(path).unwrap_or_else(|_| {
        // Fallback for invalid paths
        Url::parse(&format!("file://{}", path.display())).unwrap_or_else(|_| {
            // Last resort for truly invalid paths
            Url::parse("file:///unknown").unwrap()
        })
    })
}

/// Converts an LSP URI to a file path.
pub fn uri_to_path(uri: &Url) -> Option<std::path::PathBuf> {
    uri.to_file_path().ok()
}

/// Finds the word at the given position in the text.
pub fn word_at_position(text: &str, position: &Position) -> Option<(String, Range)> {
    let offset = position_to_byte_offset(text, position)?;

    // Find the start of the word
    let mut start = offset;
    while start > 0 {
        let prev_char = text[start - 1..start].chars().next().unwrap_or(' ');
        if !prev_char.is_alphanumeric() && prev_char != '_' {
            break;
        }
        start -= 1;
    }

    // Find the end of the word
    let mut end = offset;
    while end < text.len() {
        let next_char = text[end..end + 1].chars().next().unwrap_or(' ');
        if !next_char.is_alphanumeric() && next_char != '_' {
            break;
        }
        end += 1;
    }

    if start == end {
        return None;
    }

    let word = text[start..end].to_string();
    let range =
        Range::new(byte_offset_to_position(text, start), byte_offset_to_position(text, end));

    Some((word, range))
}
