// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/src/backend/codegen/symbol_table.rs
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

use std::collections::HashMap;
use std::rc::Rc;

use inkwell::values::BasicValueEnum;

use crate::typesystem::types::Type;

/// A symbol entry in the symbol table.
#[derive(Debug)]
pub struct SymbolEntry<'ctx> {
    /// The LLVM value representing the variable.
    pub value: BasicValueEnum<'ctx>,
    /// The type of the variable.
    pub ty: Rc<Type>,
    /// Whether the variable is mutable.
    pub mutable: bool,
}

/// A symbol table for tracking variables in scope.
#[derive(Debug)]
pub struct SymbolTable<'ctx> {
    /// Nested scopes, with the last one being the current scope.
    scopes: Vec<HashMap<String, SymbolEntry<'ctx>>>,
}

impl<'ctx> Default for SymbolTable<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ctx> SymbolTable<'ctx> {
    /// Create a new symbol table.
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());
        SymbolTable { scopes }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add a symbol to the current scope.
    pub fn add_symbol(&mut self, name: String, entry: SymbolEntry<'ctx>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, entry);
        }
    }

    /// Look up a symbol in the current scope chain.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry<'ctx>> {
        // Look in scopes from inner to outer
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }
}
