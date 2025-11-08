// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/typesystem/mod.rs
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
//! Type system for the Typhon programming language.
//!
//! This module contains the implementation of Typhon's static type system, including
//! type definitions, type checking, type inference, and error handling.

/// Type definitions and utilities.
pub mod types;

/// Type checking and inference.
pub mod checker;

/// Type system errors.
pub mod error;

/// Tests for the type system.
#[cfg(test)]
mod tests;

// Re-exports for commonly used components
pub use self::checker::{
    TypeCheckResult,
    TypeChecker,
};
pub use self::error::{
    TypeError,
    TypeErrorKind,
    TypeErrorReport,
};
pub use self::types::{
    ClassType,
    FunctionType,
    GenericInstance,
    GenericParam,
    ListType,
    PrimitiveType,
    TupleType,
    Type,
    TypeCompatibility,
    TypeEnv,
    TypeId,
    TypeVar,
    UnionType,
};

/// Result type for type checking operations.
pub type Result<T> = std::result::Result<T, TypeError>;
