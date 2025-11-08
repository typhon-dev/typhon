// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-runtime/src/lib.rs
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
//! Typhon Runtime Support Library
//!
//! This library provides runtime support for the Typhon programming language.

pub mod builtins;
pub mod errors;
pub mod memory;
pub mod object;
pub mod vm;

/// Version of the Typhon runtime
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error type for the runtime
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Initialize the runtime
pub fn initialize() -> Result<()> {
    // TODO: Implement runtime initialization
    Ok(())
}

/// Execute bytecode in the VM
pub fn execute(_bytecode: &[u8]) -> Result<()> {
    // TODO: Implement bytecode execution
    todo!("Implement bytecode execution")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_initialization() {
        assert!(initialize().is_ok());
    }
}
