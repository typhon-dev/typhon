//! Typhon CLI
//!
//! Command-line interface for the Typhon programming language.

mod commands;
mod options;
mod utils;

use anyhow::Result;
use clap::Parser;
use typhon_compiler as compiler;
use typhon_runtime as runtime;

/// The Typhon programming language compiler and runtime
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Input file to compile
    #[clap(value_parser)]
    input: Option<String>,

    /// Output file
    #[clap(short, long, value_parser)]
    output: Option<String>,

    /// Emit LLVM IR
    #[clap(long)]
    emit_llvm: bool,

    /// Optimization level
    #[clap(short = 'O', long, default_value = "0")]
    opt_level: u8,

    /// Show verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let args = Args::parse();

    if args.verbose {
        println!("Typhon Compiler v{}", compiler::VERSION);
        println!("Runtime v{}", runtime::VERSION);
    }

    // Process the input file if provided
    if let Some(input) = args.input {
        compile_file(&input, &args)?;
    } else {
        println!("No input file provided. Use --help for usage information.");
    }

    Ok(())
}

fn compile_file(input: &str, args: &Args) -> Result<()> {
    // Read the source file
    let source = std::fs::read_to_string(input)?;

    // Compile the source
    if args.verbose {
        println!("Compiling {}...", input);
    }

    // TODO: Implement actual compilation process
    // For now, just print a placeholder message
    println!("Compilation not yet implemented");

    Ok(())
}
