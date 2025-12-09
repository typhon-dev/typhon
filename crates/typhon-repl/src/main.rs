//! Typhon REPL
//!
//! Interactive REPL (Read-Eval-Print Loop) for the Typhon programming language.

use anyhow::Result;
use clap::Parser;

/// The Typhon programming language REPL
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Load an initial file
    #[clap(short, long, value_parser)]
    file: Option<String>,

    /// Show verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let args = Args::parse();

    // If a file was specified, show a message (not yet implemented)
    if let Some(file_path) = &args.file {
        if args.verbose {
            println!("Loading file: {}", file_path);
        }
        println!("File loading not implemented yet");
    }

    // Run the REPL
    typhon_repl::run(args.verbose)
}
