// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-stdlib/src/utils.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! Utility functions and types for the Typhon language.

/// Convert a value to a string representation.
pub fn to_string<T: std::fmt::Display>(value: T) -> String {
    format!("{value}")
}

/// Check if a string contains a substring.
pub fn contains(s: &str, substr: &str) -> bool {
    s.contains(substr)
}

/// Split a string by a delimiter.
pub fn split<'a>(s: &'a str, delimiter: &str) -> Vec<&'a str> {
    s.split(delimiter).collect()
}

/// Join a collection of strings with a delimiter.
pub fn join<T: AsRef<str>>(items: &[T], delimiter: &str) -> String {
    items
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<&str>>()
        .join(delimiter)
}

/// Trim whitespace from the beginning and end of a string.
pub fn trim(s: &str) -> &str {
    s.trim()
}

/// Get the length of a string.
pub fn len(s: &str) -> usize {
    s.len()
}

/// Check if a string is empty.
pub fn is_empty(s: &str) -> bool {
    s.is_empty()
}

/// Convert a string to lowercase.
pub fn to_lowercase(s: &str) -> String {
    s.to_lowercase()
}

/// Convert a string to uppercase.
pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

/// Parse a string to an integer.
pub fn parse_int(s: &str) -> Result<i64, std::num::ParseIntError> {
    s.parse()
}

/// Parse a string to a floating-point number.
pub fn parse_float(s: &str) -> Result<f64, std::num::ParseFloatError> {
    s.parse()
}

/// Range generator function.
pub fn range(start: i64, end: i64, step: i64) -> Vec<i64> {
    let mut result = Vec::new();
    let mut current = start;

    while (step > 0 && current < end) || (step < 0 && current > end) {
        result.push(current);
        current += step;
    }

    result
}
