// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/mod.rs
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
//! Code generation module for the Typhon compiler.
//!
//! This module provides code generation from Typhon AST to LLVM IR.
//! The architecture is designed to handle LLVM's complex lifetime requirements by:
//!
//! 1. Separating immutable context from mutable state
//! 2. Using `Arc<Mutex<>>` for thread-safe shared mutable access
//! 3. Carefully managing borrowing patterns to avoid conflicts
//! 4. Ensuring proper lifetimes for LLVM objects
//!
//! The main components are:
//! - `CodeGenContext`: Immutable context for code generation
//! - `CodeGenState`: Mutable state for code generation
//! - `CodeGenerator`: Main code generator that combines context and state
//! - `SymbolTable`: Tracks variables in scope during code generation

mod context;
mod expressions;
mod functions;
mod generator;
mod operations;
mod statements;
mod symbol_table;
mod types;
mod visitor;

pub use context::CodeGenContext;
pub use expressions::CodeGenExpressions;
pub use functions::{compile, gen_function};
pub use generator::{CodeGenState, CodeGenerator};
pub use operations::CodeGenOperations;
pub use statements::CodeGenStatements;
pub use symbol_table::{SymbolEntry, SymbolTable};
pub use types::{CodeGenValue, get_llvm_type};
pub use visitor::{DefaultNodeVisitor, NodeVisitor};
