// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-lsp/src/lib.rs
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
//! Typhon Language Server Protocol implementation.
//!
//! This crate provides a Language Server Protocol (LSP) implementation for the
//! Typhon programming language. It uses the tower-lsp framework for implementing
//! the LSP protocol and integrates with the Typhon compiler for language analysis.

pub mod capabilities;
pub mod document;
pub mod handlers;
pub mod server;
pub mod utils;

#[cfg(test)]
pub mod tests;

// Re-export server struct for convenience
pub use server::TyphonLanguageServer;
