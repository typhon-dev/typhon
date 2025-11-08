// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-repl/src/main.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! Typhon REPL
//!
//! Interactive REPL (Read-Eval-Print Loop) for the Typhon programming language.

mod commands;
mod completion;
mod history;
mod utils;

use anyhow::Result;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

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

    // Print welcome message
    print_welcome_message();

    // Set up the REPL editor
    let mut rl = DefaultEditor::new()?;
    if let Err(e) = rl.load_history("typhon_history.txt") {
        if args.verbose {
            println!("No previous history: {}", e);
        }
    }

    // If a file was specified, load it
    if let Some(file_path) = &args.file {
        if args.verbose {
            println!("Loading file: {}", file_path);
        }
        // TODO: Implement file loading
        println!("File loading not implemented yet");
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

                evaluate_input(&line, args.verbose);
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
