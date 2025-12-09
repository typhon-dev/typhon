//! Typhon CLI
//!
//! Command-line interface for the Typhon programming language.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

/// The Typhon programming language compiler and toolchain
#[derive(Parser, Debug)]
#[clap(
    name = "typhon",
    version,
    about = "The Typhon programming language compiler and toolchain",
    long_about = None
)]
struct CLI {
    /// Show verbose output
    #[clap(short, long, global = true)]
    verbose: bool,
    /// Subcommand to execute (defaults to REPL if none provided)
    #[clap(subcommand)]
    command: Option<Command>,
    /// Input file to run (alternative to using 'run' subcommand)
    #[clap(value_parser)]
    file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build a Typhon project or file
    Build {
        /// Input file or directory to build
        #[clap(value_parser)]
        input: Option<PathBuf>,
        /// Output file
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,
        /// Emit LLVM IR
        #[clap(long)]
        emit_llvm: bool,
        /// Optimization level (0-3)
        #[clap(short = 'O', long, default_value = "0")]
        opt_level: u8,
        /// Build in release mode
        #[clap(short, long)]
        release: bool,
    },

    /// Type check a Typhon project or file without building
    Check {
        /// Input file or directory to check
        #[clap(value_parser)]
        input: Option<PathBuf>,
        /// Check all files in workspace
        #[clap(long)]
        all: bool,
    },

    /// Generate documentation
    Doc {
        /// Open documentation in browser after building
        #[clap(long)]
        open: bool,
        /// Generate docs without dependencies
        #[clap(long)]
        no_deps: bool,
    },

    /// Format Typhon source files
    Fmt {
        /// Files or directories to format
        #[clap(value_parser)]
        paths: Vec<PathBuf>,
        /// Check if files are formatted without modifying
        #[clap(long)]
        check: bool,
    },

    /// Initialize a new Typhon project in the current directory
    Init {
        /// Name of the project (defaults to directory name)
        #[clap(value_parser)]
        name: Option<String>,
    },

    /// Lint Typhon source files
    Lint {
        /// Files or directories to lint
        #[clap(value_parser)]
        paths: Vec<PathBuf>,
        /// Automatically fix issues where possible
        #[clap(long)]
        fix: bool,
    },

    /// Create a new Typhon project
    New {
        /// Name of the project
        #[clap(value_parser)]
        name: String,
        /// Template to use
        #[clap(short, long, value_parser)]
        template: Option<String>,
    },

    /// Run a Typhon file
    Run {
        /// Input file to run
        #[clap(value_parser)]
        file: PathBuf,
        /// Arguments to pass to the program
        #[clap(value_parser)]
        args: Vec<String>,
    },

    /// Run tests
    Test {
        /// Test name or pattern to run
        #[clap(value_parser)]
        pattern: Option<String>,
        /// Run tests in release mode
        #[clap(short, long)]
        release: bool,
        /// Run ignored tests
        #[clap(long)]
        ignored: bool,
    },

    /// Show version information
    Version {
        /// Show detailed version information
        #[clap(short, long)]
        detailed: bool,
    },

    /// Watch for changes and rebuild
    Watch {
        /// Command to run on changes (build, test, check)
        #[clap(value_parser, default_value = "build")]
        command: String,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let cli = CLI::parse();

    // Determine what action to take
    match (cli.command, cli.file) {
        // Explicit subcommand provided
        (Some(command), None) => execute_command(command, cli.verbose),

        // File argument provided without subcommand - run it
        (None, Some(file)) => commands::run::execute(file, Vec::new(), cli.verbose),

        // No subcommand or file - launch REPL
        (None, None) => commands::repl::execute(cli.verbose),

        // Both subcommand and file argument provided - error
        (Some(_), Some(_)) => {
            anyhow::bail!("Cannot specify both a subcommand and a file argument")
        }
    }
}

fn execute_command(command: Command, verbose: bool) -> Result<()> {
    match command {
        Command::Build { input, output, emit_llvm, opt_level, release } => {
            commands::build::execute(input, output, emit_llvm, opt_level, release, verbose)
        }
        Command::Check { input, all } => commands::check::execute(input, all, verbose),
        Command::Doc { open, no_deps } => commands::doc::execute(open, no_deps, verbose),
        Command::Fmt { paths, check } => commands::fmt::execute(paths, check, verbose),
        Command::Init { name } => commands::init::execute(name, verbose),
        Command::Lint { paths, fix } => commands::lint::execute(paths, fix, verbose),
        Command::New { name, template } => commands::new::execute(name, template, verbose),
        Command::Run { file, args } => commands::run::execute(file, args, verbose),
        Command::Test { pattern, release, ignored } => {
            commands::test::execute(pattern, release, ignored, verbose)
        }
        Command::Version { detailed } => {
            commands::version::execute(detailed);

            Ok(())
        }
        Command::Watch { command } => commands::watch::execute(command, verbose),
    }
}
