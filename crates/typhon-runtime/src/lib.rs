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
