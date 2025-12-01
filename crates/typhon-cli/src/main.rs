//! Typhon CLI
//!
//! Command-line interface for the Typhon programming language.

use std::fs::{File, read_to_string};
use std::io::Write;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser as ArgParser;
use typhon_compiler::backend::{CodeGenerator, CompilerContext};
use typhon_compiler::driver::Driver;
use typhon_parser::parser::Parser;

/// The Typhon programming language compiler and runtime
#[derive(ArgParser, Debug)]
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
        println!("Typhon Compiler v{}", typhon_compiler::VERSION);
        println!("Runtime v{}", typhon_runtime::VERSION);
    }

    // Process the input file if provided
    if let Some(input) = &args.input {
        compile_file(input, &args)?;
    } else {
        println!("No input file provided. Use --help for usage information.");
    }

    Ok(())
}

fn compile_file(input: &str, args: &Args) -> Result<()> {
    // Read the source file
    let source = read_to_string(input)?;

    // Compile the source
    if args.verbose {
        println!("Compiling {input}...");
    }

    let compiler_context = Arc::new(CompilerContext::new());

    // Implement full compilation process using the Driver
    let driver = Driver::new(compiler_context.clone(), input);

    // Lexical analysis and parsing
    driver.compile_file(Path::new(input)).context("Failed to compile file using driver")?;

    if args.verbose {
        println!("Parsing successful. AST created with {} statements.", module.statements.len());
    }

    // Create code generator
    let mut code_generator = CodeGenerator::new(compiler_context.clone());

    // Generate code
    code_generator.compile(&module.statements).context("Failed to generate code")?;

    // Get the generated LLVM IR
    let llvm_ir = compiler_context.module().to_string();

    // Output the LLVM IR or compile to an executable
    if args.emit_llvm {
        // Output LLVM IR to a file
        let output_path =
            args.output.clone().unwrap_or_else(|| format!("{}.ll", input.trim_end_matches(".ty")));

        let mut output_file = File::create(&output_path)?;
        write!(output_file, "{llvm_ir}")?;

        if args.verbose {
            println!("LLVM IR written to {output_path}");
        }
    } else {
        // For now, just output LLVM IR since executable compilation isn't implemented
        println!("{llvm_ir}");

        if args.verbose {
            println!("LLVM IR generated successfully.");
        }
    }

    Ok(())
}
