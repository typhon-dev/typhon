//! REPL command implementation

use anyhow::Result;

/// Execute the REPL
pub fn execute(verbose: bool) -> Result<()> {
    if verbose {
        println!("Starting Typhon REPL...");
    }

    // Launch the REPL from typhon-repl crate
    // For now, we'll use a simple approach - in the future we can integrate more deeply
    typhon_repl::run(verbose)
}
