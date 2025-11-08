// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-cli/src/main.rs
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
//! Typhon CLI
//!
//! Command-line interface for the Typhon programming language.

use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use anyhow::{
    Context,
    Result,
};
use clap::Parser;
use typhon_compiler::backend::CodeGenerator;
use typhon_compiler::backend::llvm::LLVMContext;
use typhon_compiler::driver::Driver;
use typhon_compiler::frontend::parser::Parser as TyphonParser;
use {
    typhon_compiler as compiler,
    typhon_runtime as runtime,
};

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

    // Implement full compilation process using the Driver
    let driver = Driver::new();

    // We can either use the driver's high-level API
    // driver.compile_file(Path::new(input)).context("Failed to compile file using driver")?;

    // Or use the individual components for more control:

    // Lexical analysis and parsing
    let mut parser = TyphonParser::new(&source);
    let module = parser.parse().context("Failed to parse source code")?;

    if args.verbose {
        println!(
            "Parsing successful. AST created with {} statements.",
            module.statements.len()
        );
    }

    // Create LLVM context for code generation
    let llvm_context = Rc::new(RefCell::new(LLVMContext::new("typhon_module")));

    // Create code generator
    let mut code_generator = CodeGenerator::new(llvm_context.clone());

    // Generate code
    code_generator
        .compile(&module.statements)
        .context("Failed to generate code")?;

    // Get the generated LLVM IR
    let llvm_ir = llvm_context.borrow().module().to_string();

    // Output the LLVM IR or compile to an executable
    if args.emit_llvm {
        // Output LLVM IR to a file
        let output_path = args
            .output
            .clone()
            .unwrap_or_else(|| format!("{}.ll", input.trim_end_matches(".ty")));

        let mut output_file = File::create(&output_path)?;
        write!(output_file, "{}", llvm_ir)?;

        if args.verbose {
            println!("LLVM IR written to {}", output_path);
        }
    } else {
        // For now, just output LLVM IR since executable compilation isn't implemented
        println!("{}", llvm_ir);

        if args.verbose {
            println!("LLVM IR generated successfully.");
        }
    }

    Ok(())
}
