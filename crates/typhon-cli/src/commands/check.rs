//! Check command implementation

use std::path::PathBuf;

use anyhow::Result;

/// Type check a Typhon project or file without building
pub fn execute(input: Option<PathBuf>, all: bool, verbose: bool) -> Result<()> {
    let input_path = input.unwrap_or_else(|| PathBuf::from("."));

    if verbose {
        println!("Type checking: {}", input_path.display());
        if all {
            println!("Checking all files in workspace");
        }
    }

    // TODO: Implement actual type checking
    // This should:
    // 1. Parse the input file(s)
    // 2. Run type checking
    // 3. Report any type errors
    println!("Type checking not yet implemented");
    println!("Would check: {}", input_path.display());

    Ok(())
}
