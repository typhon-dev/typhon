//! Utility functions and helpers used across the Typhon parser.
//!
//! This module provides utility functions, helper macros, and testing utilities
//! that are used throughout the Typhon parser implementation.

use typhon_source::types::{SourceSpan, Span};

use crate::lexer::TokenKind;

/// The size of a tab character in spaces.
pub const TAB_SIZE: usize = 4;

/// Text handling utilities and helpers.
pub mod text {
    /// Determines if a character can be the start of an identifier.
    ///
    /// In Typhon, identifiers can start with an underscore or an ASCII letter.
    ///
    /// ## Examples
    ///
    /// ```
    /// use typhon_parser::utils::text::is_id_start;
    ///
    /// assert!(is_id_start('a'));
    /// assert!(is_id_start('Z'));
    /// assert!(is_id_start('_'));
    /// assert!(!is_id_start('0'));
    /// assert!(!is_id_start('-'));
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_id_start(c: char) -> bool { c == '_' || c.is_ascii_alphabetic() }

    /// Determines if a character can be part of an identifier.
    ///
    /// In Typhon, identifiers can contain underscores, ASCII letters, and digits
    /// (except at the first position).
    ///
    /// ## Examples
    ///
    /// ```
    /// use typhon_parser::utils::text::is_id_continue;
    ///
    /// assert!(is_id_continue('a'));
    /// assert!(is_id_continue('Z'));
    /// assert!(is_id_continue('_'));
    /// assert!(is_id_continue('0'));
    /// assert!(!is_id_continue('-'));
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_id_continue(c: char) -> bool { c == '_' || c.is_ascii_alphanumeric() }

    /// Computes line starts for a source text.
    ///
    /// Returns a vector of byte offsets where each line begins.
    /// The first entry is always 0 (first line starts at byte 0).
    ///
    /// ## Examples
    ///
    /// ```
    /// use typhon_parser::utils::text::compute_line_starts;
    ///
    /// let content = "hello\nworld\n";
    /// let line_starts = compute_line_starts(content);
    /// assert_eq!(line_starts, vec![0, 6, 12]);
    /// ```
    #[must_use]
    pub fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut line_starts = vec![0]; // First line always starts at byte 0

        for (i, c) in content.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        line_starts
    }

    /// Calculates the number of spaces or equivalent tab spaces at a given position.
    ///
    /// This is useful for determining indentation levels. Tabs are counted as `TAB_SIZE` spaces.
    ///
    /// ## Examples
    ///
    /// ```
    /// use typhon_parser::utils::text::calculate_indentation;
    /// use typhon_parser::utils::TAB_SIZE;
    ///
    /// let content = "    hello"; // 4 spaces
    /// assert_eq!(calculate_indentation(content, 0), 4);
    ///
    /// let content = "\thello"; // 1 tab
    /// assert_eq!(calculate_indentation(content, 0), TAB_SIZE);
    /// ```
    #[must_use]
    pub fn calculate_indentation(content: &str, start_pos: usize) -> usize {
        let mut count = 0;

        for byte in content.bytes().skip(start_pos) {
            match byte {
                b' ' => count += 1,
                b'\t' => count += super::TAB_SIZE,
                _ => break,
            }
        }

        count
    }
}

/// Compute line start offsets for a source string.
///
/// Returns a vector of byte offsets where each line begins.
/// The first element is always 0 (first line starts at byte 0).
///
/// ## Examples
///
/// ```
/// use typhon_parser::utils::compute_line_starts;
///
/// let content = "hello\nworld\n";
/// let line_starts = compute_line_starts(content);
/// assert_eq!(line_starts, vec![0, 6, 12]);
/// ```
#[inline]
#[must_use]
pub fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0]; // First line always starts at offset 0
    for (i, c) in source.char_indices() {
        if c == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Determine if a character is valid as the first character in an identifier.
///
/// Valid first characters for identifiers in Typhon are ASCII alphabetic characters or underscore.
///
/// ## Examples
///
/// ```
/// use typhon_parser::utils::is_id_start;
///
/// assert!(is_id_start('a'));
/// assert!(is_id_start('Z'));
/// assert!(is_id_start('_'));
/// assert!(!is_id_start('0'));
/// assert!(!is_id_start('$'));
/// ```
#[inline]
#[must_use]
pub const fn is_id_start(c: char) -> bool { c == '_' || c.is_ascii_alphabetic() }

/// Determine if a character can be part of an identifier.
///
/// In Typhon, identifiers can contain underscores, ASCII letters, and digits
/// (except at the first position).
///
/// ## Examples
///
/// ```
/// use typhon_parser::utils::is_id_continue;
///
/// assert!(is_id_continue('a'));
/// assert!(is_id_continue('Z'));
/// assert!(is_id_continue('_'));
/// assert!(is_id_continue('0'));
/// assert!(!is_id_continue('$'));
/// ```
#[inline]
#[must_use]
pub const fn is_id_continue(c: char) -> bool { c == '_' || c.is_ascii_alphanumeric() }

/// Determine if a token can start a statement.
///
/// This is used by the parser to determine if a token can begin a statement,
/// which is useful for error recovery and statement boundary detection.
///
/// ## Examples
///
/// ```
/// use typhon_parser::utils::can_start_statement;
/// use typhon_parser::lexer::TokenKind;
///
/// assert!(can_start_statement(TokenKind::Def));
/// assert!(can_start_statement(TokenKind::If));
/// assert!(!can_start_statement(TokenKind::Plus));
/// ```
#[must_use]
pub const fn can_start_statement(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Assert
            | TokenKind::Break
            | TokenKind::Class
            | TokenKind::Continue
            | TokenKind::Def
            | TokenKind::For
            | TokenKind::From
            | TokenKind::Identifier
            | TokenKind::If
            | TokenKind::Import
            | TokenKind::Pass
            | TokenKind::Raise
            | TokenKind::Return
            | TokenKind::Try
            | TokenKind::While
            | TokenKind::With
    )
}

/// Create a new span that encompasses multiple spans.
///
/// This function takes a slice of spans and returns a new span that
/// starts at the earliest start position and ends at the latest end position.
/// All spans must be from the same file.
///
/// Returns `None` if the input slice is empty.
///
/// ## Examples
///
/// ```
/// use typhon_parser::utils::combine_spans;
/// use typhon_source::types::{FileID, Position, SourceSpan};
///
/// let file_id = FileID::new(1);
/// let span1 = SourceSpan::new(
///     Position::new(1, 1, 0),
///     Position::new(1, 5, 4),
///     file_id
/// );
/// let span2 = SourceSpan::new(
///     Position::new(1, 6, 5),
///     Position::new(1, 10, 9),
///     file_id
/// );
///
/// let combined = combine_spans(&[span1, span2]).unwrap();
/// assert_eq!(combined.start.offset, 0);
/// assert_eq!(combined.end.offset, 9);
/// ```
///
/// ## Panics
///
/// TODO: add context
#[must_use]
pub fn combine_spans(spans: &[SourceSpan]) -> Option<SourceSpan> {
    if spans.is_empty() {
        return None;
    }

    let first = spans[0];
    let last = spans[spans.len() - 1];

    // Ensure all spans are from the same file
    for span in spans {
        assert!((span.file_id == first.file_id), "Cannot combine spans from different files");
    }

    Some(SourceSpan { start: first.start, end: last.end, file_id: first.file_id })
}

/// Create a new `Span` that encompasses multiple `Span`s.
///
/// This function takes a slice of `Span`s and returns a new `Span` that
/// starts at the earliest start position and ends at the latest end position.
///
/// Returns `None` if the input slice is empty.
///
/// ## Examples
///
/// ```
/// use typhon_parser::utils::combine_simple_spans;
/// use typhon_source::types::Span;
///
/// let span1 = Span::new(0, 4);
/// let span2 = Span::new(5, 9);
///
/// let combined = combine_simple_spans(&[span1, span2]).unwrap();
/// assert_eq!(combined.start, 0);
/// assert_eq!(combined.end, 9);
/// ```
#[must_use]
pub fn combine_simple_spans(spans: &[Span]) -> Option<Span> {
    if spans.is_empty() {
        return None;
    }

    let mut start = spans[0].start;
    let mut end = spans[0].end;

    for span in spans.iter().skip(1) {
        start = start.min(span.start);
        end = end.max(span.end);
    }

    Some(Span { start, end })
}

// AST Node helper macros
// These will be implemented as the AST types are defined

/// Macro to create a binary operation AST node.
///
/// This macro simplifies the creation of binary operation nodes in the AST.
///
/// ## Examples
///
/// ```ignore
/// let left = /* some expression */;
/// let right = /* some expression */;
/// let span = /* some span */;
/// let op = BinaryOperator::Add;
///
/// let binary_op = binary_op!(span, left, op, right);
/// ```
#[macro_export]
macro_rules! binary_op {
    ($span:expr, $left:expr, $op:expr, $right:expr) => {
        Box::new($crate::ast::BinaryOp { span: $span, left: $left, op: $op, right: $right })
    };
}

/// Macro to create a unary operation AST node.
///
/// This macro simplifies the creation of unary operation nodes in the AST.
///
/// ## Examples
///
/// ```ignore
/// let operand = /* some expression */;
/// let span = /* some span */;
/// let op = UnaryOperator::Not;
///
/// let unary_op = unary_op!(span, op, operand);
/// ```
#[macro_export]
macro_rules! unary_op {
    ($span:expr, $op:expr, $operand:expr) => {
        Box::new(UnaryOp { span: $span, op: $op, operand: $operand })
    };
}

/// Implements common AST node functionality.
///
/// This macro implements the `ASTNode` trait for a given type.
///
/// ## Examples
///
/// ```ignore
/// struct MyNode {
///     source_info: SourceInfo,
///     // other fields...
/// }
///
/// impl MyNode {
///     fn source_info(&self) -> &SourceInfo {
///         &self.source_info
///     }
///
///     fn children(&self) -> Vec<&dyn ASTNode> {
///         vec![]
///     }
/// }
///
/// impl_ast_node!(MyNode, source_info, children);
/// ```
#[macro_export]
macro_rules! impl_ast_node {
    ($type:ty, $source_info_method:ident, $children_method:ident) => {
        impl ASTNode for $type {
            fn source_info(&self) -> &SourceInfo { self.$source_info_method() }

            fn children(&self) -> Vec<&dyn ASTNode> { self.$children_method() }

            fn is_expression(&self) -> bool {
                false // Default implementation, can be overridden
            }

            fn is_statement(&self) -> bool {
                false // Default implementation, can be overridden
            }

            fn is_type_expression(&self) -> bool {
                false // Default implementation, can be overridden
            }

            fn kind(&self) -> &'static str {
                stringify!($type) // Returns the type name as a string
            }

            fn span(&self) -> (usize, usize) {
                let source_info = self.source_info();
                (source_info.span.start, source_info.span.end)
            }

            fn to_string(&self) -> String { format!("{:?}", self) }
        }
    };
}

pub use crate::{binary_op, impl_ast_node, unary_op};
