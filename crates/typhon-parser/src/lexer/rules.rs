//! Additional rules for lexical analysis in the Typhon language.
//!
//! This module contains helper functions for the lexer, including:
//! - Rules for string literal handling
//! - Rules for handling indentation
//! - Rules for handling line continuations
//! - Helper functions for token processing
//! - Soft keyword handling
//! - Template string interpolation detection

use std::sync::OnceLock;

use rustc_hash::FxHashMap;

use super::token::{Token, TokenKind};

/// Get a global map of soft keywords for efficient lookups
pub fn soft_keywords() -> &'static FxHashMap<&'static str, TokenKind> {
    static KEYWORDS: OnceLock<FxHashMap<&'static str, TokenKind>> = OnceLock::new();

    KEYWORDS.get_or_init(|| {
        let mut map = FxHashMap::default();
        let _ = map.insert("match", TokenKind::Match);
        let _ = map.insert("case", TokenKind::Case);
        let _ = map.insert("type", TokenKind::Type);
        let _ = map.insert("_", TokenKind::Underscore);

        map
    })
}

/// Check if an identifier is a soft keyword in the current context
///
/// This function checks if an identifier is actually a soft keyword. Soft keywords only
/// act as keywords in specific contexts, but can be used as regular identifiers elsewhere.
#[must_use]
pub fn check_soft_keyword(lexeme: &str) -> Option<TokenKind> {
    soft_keywords().get(lexeme).copied()
}

/// Determines if a token is in template string interpolation context
///
/// This function checks if we're inside template string interpolation, which affects
/// how brackets are processed. In template strings, brackets need special handling for
/// interpolation sections.
#[must_use]
pub const fn is_in_template_string_context(kind: TokenKind, in_brackets: usize) -> bool {
    matches!(kind, TokenKind::TmplStringLiteral | TokenKind::MultilineTmplStringLiteral)
        && in_brackets > 0
}

/// Check if a token represents a string literal
///
/// This function checks if a token is any type of string literal, which is
/// useful for handling string concatenation.
#[must_use]
pub const fn is_string_literal(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::StringLiteral
            | TokenKind::RawStringLiteral
            | TokenKind::FmtStringLiteral
            | TokenKind::TmplStringLiteral
            | TokenKind::BytesLiteral
            | TokenKind::MultilineStringLiteral
            | TokenKind::MultilineFmtStringLiteral
            | TokenKind::MultilineTmplStringLiteral
            | TokenKind::MultilineBytesLiteral
    )
}

/// Check if a token is a line continuation
///
/// This function checks if a token represents a line continuation, either
/// explicit (via backslash) or implicit (inside brackets).
#[must_use]
pub fn is_line_continuation(kind: TokenKind, in_brackets: usize, next_char: Option<char>) -> bool {
    // Inside brackets, line continuations are implicit
    if in_brackets > 0 {
        return true;
    }

    // Check for explicit line continuation with backslash
    if kind == TokenKind::Newline && next_char == Some('\\') {
        return true;
    }

    false
}

/// Calculate the indentation level for a line
///
/// This function calculates the indentation level for a line, taking into
/// account Python's rules for tabs vs spaces.
#[must_use]
pub fn calculate_indentation(line: &str) -> (usize, bool) {
    let mut indent_level = 0;
    let mut mixed_indentation = false;

    // Count the number of spaces and tabs at the beginning of the line
    for ch in line.chars() {
        match ch {
            ' ' => indent_level += 1,
            '\t' => {
                // Tab counts as 8 spaces in Python
                indent_level += 8;
                mixed_indentation = true;
            }
            _ => break,
        }
    }

    (indent_level, mixed_indentation)
}

/// Extract the actual content from a string literal
///
/// This function removes the quotes and any prefixes from a string literal.
#[must_use]
pub fn extract_string_content<'src>(token: &Token<'src>) -> &'src str {
    match token.kind {
        TokenKind::StringLiteral => {
            // For normal string literals, remove the quotes
            &token.lexeme[1..token.lexeme.len() - 1]
        }
        TokenKind::RawStringLiteral => {
            // For raw string literals, remove the 'r' prefix and quotes
            &token.lexeme[2..token.lexeme.len() - 1]
        }
        TokenKind::FmtStringLiteral => {
            // For formatted string literals, remove the 'f' prefix and quotes
            &token.lexeme[2..token.lexeme.len() - 1]
        }
        TokenKind::TmplStringLiteral => {
            // For template string literals, remove the 't' prefix and quotes
            &token.lexeme[2..token.lexeme.len() - 1]
        }
        TokenKind::BytesLiteral => {
            // For bytes literals, remove the 'b' prefix and quotes
            &token.lexeme[2..token.lexeme.len() - 1]
        }
        TokenKind::MultilineStringLiteral => {
            // For multiline string literals, remove the triple quotes
            &token.lexeme[3..token.lexeme.len() - 3]
        }
        TokenKind::MultilineFmtStringLiteral => {
            // For multiline formatted string literals, remove the 'f' prefix and triple quotes
            &token.lexeme[4..token.lexeme.len() - 3]
        }
        TokenKind::MultilineTmplStringLiteral => {
            // For multiline template string literals, remove the 't' prefix and triple quotes
            &token.lexeme[4..token.lexeme.len() - 3]
        }
        TokenKind::MultilineBytesLiteral => {
            // For multiline bytes literals, remove the 'b' prefix and triple quotes
            &token.lexeme[4..token.lexeme.len() - 3]
        }
        _ => token.lexeme, // Default case should not happen for string literals
    }
}

/// Join two string literals together
///
/// This function joins two string literals together, handling the different
/// types of string literals.
#[must_use]
pub fn join_string_literals<'src>(
    token1: &Token<'src>,
    token2: &Token<'src>,
    source: &'src str,
) -> Token<'src> {
    // Create a new range that covers both tokens
    let combined_span = token1.span.start..token2.span.end;

    // Extract the full lexeme from the source
    let combined_lexeme = &source[token1.span.start..token2.span.end];

    // Create a new token with the same kind as the first token
    Token::new(token1.kind, combined_lexeme, combined_span)
}
