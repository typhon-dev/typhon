//! Build command implementation

use std::path::PathBuf;

use anyhow::{Context, Result};

/// Build a Typhon project or file
pub fn execute(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    emit_llvm: bool,
    opt_level: u8,
    release: bool,
    verbose: bool,
) -> Result<()> {
    let input_path = input.unwrap_or_else(|| PathBuf::from("."));

    if verbose {
        println!("Building: {}", input_path.display());
        if let Some(ref out) = output {
            println!("Output: {}", out.display());
        }

        println!("Optimization level: {opt_level}");
        println!("Release mode: {release}");
        println!("Emit LLVM IR: {emit_llvm}");
    }

    // TODO: Implement actual build logic
    // This should:
    // 1. Parse the input file(s)
    // 2. Run type checking
    // 3. Generate LLVM IR
    // 4. Compile to executable (if not emit_llvm)
    println!("Build functionality not yet implemented");
    println!("Would build: {}", input_path.display());

    Ok(())
}
