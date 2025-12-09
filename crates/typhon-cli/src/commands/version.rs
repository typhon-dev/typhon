//! Version command implementation

use std::env;

/// Show version information
pub(crate) fn execute(detailed: bool) {
    if detailed {
        println!("Typhon CLI v{}", env!("CARGO_PKG_VERSION"));
        println!("Typhon Compiler v{}", typhon_runtime::VERSION);
        println!("Typhon Runtime v{}", typhon_runtime::VERSION);
        println!();
        println!("Build information:");
        println!("  Rust version: {}", env!("CARGO_PKG_RUST_VERSION"));
    } else {
        println!("typhon {}", env!("CARGO_PKG_VERSION"));
    }
}
