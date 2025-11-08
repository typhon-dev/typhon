// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-stdlib/src/collections.rs
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
//! Collection types for the Typhon language.

/// List collection implementation.
pub mod list {
    /// A basic list type.
    #[derive(Clone, Debug)]
    pub struct List<T> {
        items: Vec<T>,
    }

    impl<T> List<T> {
        /// Create a new empty list.
        pub fn new() -> Self {
            Self { items: Vec::new() }
        }

        /// Add an item to the list.
        pub fn append(&mut self, item: T) {
            self.items.push(item);
        }

        /// Get the length of the list.
        pub fn len(&self) -> usize {
            self.items.len()
        }

        /// Check if the list is empty.
        pub fn is_empty(&self) -> bool {
            self.items.is_empty()
        }
    }

    impl<T> Default for List<T> {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Dictionary collection implementation.
pub mod dict {
    use std::collections::HashMap;

    /// A basic dictionary type.
    #[derive(Clone, Debug)]
    pub struct Dict<K, V> {
        items: HashMap<K, V>,
    }

    impl<K, V> Dict<K, V>
    where
        K: std::hash::Hash + Eq,
    {
        /// Create a new empty dictionary.
        pub fn new() -> Self {
            Self {
                items: HashMap::new(),
            }
        }

        /// Add or update a key-value pair.
        pub fn set(&mut self, key: K, value: V) {
            self.items.insert(key, value);
        }

        /// Check if the dictionary is empty.
        pub fn is_empty(&self) -> bool {
            self.items.is_empty()
        }
    }

    impl<K, V> Default for Dict<K, V>
    where
        K: std::hash::Hash + Eq,
    {
        fn default() -> Self {
            Self::new()
        }
    }
}
