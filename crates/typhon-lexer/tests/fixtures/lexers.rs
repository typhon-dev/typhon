//! `#[fixture]` constructors for [`Lexer`] instances used in tests.
//!
//! The default fixture, [`lexer`], is parametrised over a `&str` source so
//! callers can override it with `#[with(source)]`. A fixed [`FileID`] is used
//! across all fixtures to keep span comparisons stable.

use rstest::fixture;
use typhon_lexer::Lexer;
use typhon_source::types::FileID;

/// Stable [`FileID`] used by every test lexer so span equality assertions are
/// deterministic.
pub const TEST_FILE_ID: FileID = FileID::new(1);

/// Default [`Lexer`] fixture.
///
/// Use `#[with("…")]` on a test argument typed `Lexer<'_>` to override the
/// source. Without an override the fixture lexes an empty string, which is
/// only useful for callers that immediately drive the lexer themselves.
#[fixture]
pub fn lexer<'src>(#[default("")] source: &'src str) -> Lexer<'src> {
    Lexer::new(source, TEST_FILE_ID)
}
