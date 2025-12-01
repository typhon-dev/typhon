//! Typhon Standard Library
//!
//! This library provides the standard library for the Typhon programming language.

pub mod builtins;
pub mod collections;
pub mod errors;
pub mod io;
pub mod utils;

/// Version of the Typhon standard library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error type for the standard library
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
