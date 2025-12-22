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
        #[must_use]
        pub const fn new() -> Self { Self { items: Vec::new() } }

        /// Add an item to the list.
        pub fn append(&mut self, item: T) { self.items.push(item); }

        /// Get the length of the list.
        #[must_use]
        pub const fn len(&self) -> usize { self.items.len() }

        /// Check if the list is empty.
        #[must_use]
        pub const fn is_empty(&self) -> bool { self.items.is_empty() }
    }

    impl<T> Default for List<T> {
        fn default() -> Self { Self::new() }
    }
}

/// Dictionary collection implementation.
pub mod dict {
    use std::collections::HashMap;
    use std::hash::Hash;

    /// A basic dictionary type.
    #[derive(Clone, Debug)]
    pub struct Dict<K, V> {
        items: HashMap<K, V>,
    }

    impl<K, V> Dict<K, V>
    where K: Hash + Eq
    {
        /// Create a new empty dictionary.
        #[must_use]
        pub fn new() -> Self { Self { items: HashMap::new() } }

        /// Add or update a key-value pair.
        pub fn set(&mut self, key: K, value: V) { drop(self.items.insert(key, value)); }

        /// Check if the dictionary is empty.
        #[must_use]
        pub fn is_empty(&self) -> bool { self.items.is_empty() }
    }

    impl<K, V> Default for Dict<K, V>
    where K: Hash + Eq
    {
        fn default() -> Self { Self::new() }
    }
}
