//! Typhon REPL Library
//!
//! Provides programmatic access to the Typhon REPL functionality.

use anyhow::Result;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

/// Run the Typhon REPL
pub fn run(verbose: bool) -> Result<()> {
    // Print welcome message
    print_welcome_message();

    // Set up the REPL editor
    let mut rl = DefaultEditor::new()?;
    if let Err(e) = rl.load_history("typhon_history.txt") {
        if verbose {
            println!("No previous history: {}", e);
        }
    }

    // REPL loop
    loop {
        let readline = rl.readline("typhon> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;

                if line.trim() == "exit" || line.trim() == "quit" {
                    println!("Goodbye!");
                    break;
                }

                evaluate_input(&line, verbose);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    // Save history
    if let Err(e) = rl.save_history("typhon_history.txt") {
        eprintln!("Failed to save history: {}", e);
    }

    Ok(())
}

fn print_welcome_message() {
    println!("Typhon REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or press Ctrl-C to quit");
    println!("Type 'help' for a list of commands");
    println!();
}

fn evaluate_input(input: &str, verbose: bool) {
    // TODO: Implement actual evaluation
    // For now, just echo the input
    if verbose {
        println!("Evaluating: {}", input);
    }

    println!("Not yet implemented. Received: {}", input);
}
