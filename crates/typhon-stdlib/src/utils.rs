//! Utility functions and types for the Typhon language.

use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};

/// Convert a value to a string representation.
pub fn to_string<T: Display>(value: T) -> String { format!("{value}") }

/// Check if a string contains a substring.
#[must_use]
pub fn contains(s: &str, substr: &str) -> bool { s.contains(substr) }

/// Split a string by a delimiter.
#[must_use]
pub fn split<'a>(s: &'a str, delimiter: &str) -> Vec<&'a str> { s.split(delimiter).collect() }

/// Join a collection of strings with a delimiter.
pub fn join<T: AsRef<str>>(items: &[T], delimiter: &str) -> String {
    items.iter().map(AsRef::as_ref).collect::<Vec<&str>>().join(delimiter)
}

/// Trim whitespace from the beginning and end of a string.
#[must_use]
pub fn trim(s: &str) -> &str { s.trim() }

/// Get the length of a string.
#[must_use]
pub const fn len(s: &str) -> usize { s.len() }

/// Check if a string is empty.
#[must_use]
pub const fn is_empty(s: &str) -> bool { s.is_empty() }

/// Convert a string to lowercase.
#[must_use]
pub fn to_lowercase(s: &str) -> String { s.to_lowercase() }

/// Convert a string to uppercase.
#[must_use]
pub fn to_uppercase(s: &str) -> String { s.to_uppercase() }

/// Parse a string to an integer.
pub fn parse_int(s: &str) -> Result<i64, ParseIntError> { s.parse() }

/// Parse a string to a floating-point number.
pub fn parse_float(s: &str) -> Result<f64, ParseFloatError> { s.parse() }

/// Range generator function.
#[must_use]
pub fn range(start: i64, end: i64, step: i64) -> Vec<i64> {
    let mut result = Vec::new();
    let mut current = start;

    while (step > 0 && current < end) || (step < 0 && current > end) {
        result.push(current);
        current += step;
    }

    result
}
