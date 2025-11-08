// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/common/mod.rs
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
