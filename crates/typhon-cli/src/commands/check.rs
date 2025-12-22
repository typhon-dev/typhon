//! Check command implementation
//!
//! This module implements the type checking functionality without building:
//! Source → Parser → AST → Analyzer (Symbol Collection, Name Resolution, Type Checking)

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use colored::Colorize;
use log::{info, warn};
use typhon_analyzer::analysis::{DeadCodeWarning, WarningSeverity};
use typhon_analyzer::analyze_module;
use typhon_analyzer::error::SemanticError;
use typhon_parser::parser::Parser;
use typhon_source::types::{FileID, Position, SourceManager};
use walkdir::WalkDir;

/// Type check a Typhon project or file without building
pub fn execute(input: Option<PathBuf>, all: bool, verbose: bool) -> Result<()> {
    let input_path = input.unwrap_or_else(|| PathBuf::from("."));

    if verbose {
        info!("Type checking: {}", input_path.display());
        if all {
            info!("Checking all files in workspace");
        }
    }

    // Track overall success and error count
    let (success, error_count, file_count) = if input_path.is_file() {
        // Single file check
        let (success, errors) = check_single_file(&input_path, verbose)?;

        (success, errors, 1)
    } else if input_path.is_dir() {
        // Directory check
        check_directory(&input_path, all, verbose)?
    } else {
        anyhow::bail!("Input path '{}' is not a valid file or directory", input_path.display());
    };

    // Print summary
    print_error_summary(error_count, file_count, success);

    // Return error if any errors were found
    if !success {
        anyhow::bail!("Type checking failed with {error_count} errors");
    }

    Ok(())
}

/// Checks files in a directory for semantic errors
fn check_directory(dir_path: &Path, all: bool, verbose: bool) -> Result<(bool, usize, usize)> {
    let files_to_check = resolve_input(dir_path, all)?;

    if files_to_check.is_empty() {
        if all {
            warn!("No .ty files found in directory: {}", dir_path.display());
        } else {
            anyhow::bail!(
                "Directory '{}' does not contain __main__.ty or __init__.ty",
                dir_path.display()
            );
        }
        return Ok((true, 0, 0));
    }

    let mut overall_success = true;
    let mut total_errors = 0;
    let file_count = files_to_check.len();

    if verbose {
        info!("Found {file_count} file(s) to check");
    }

    for file in &files_to_check {
        if verbose {
            info!("\nChecking file: {}", file.display());
        } else {
            info!("Checking: {}", file.display());
        }

        let (success, error_count) = check_single_file(file, verbose)?;

        overall_success &= success;
        total_errors += error_count;
    }

    Ok((overall_success, total_errors, file_count))
}

/// Checks a single file for semantic errors
fn check_single_file(file_path: &Path, verbose: bool) -> Result<(bool, usize)> {
    // Read the source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read input file: {}", file_path.display()))?;

    if verbose {
        info!("Parsing file: {}", file_path.display());
    }

    // Set up the parser
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file(file_path.to_string_lossy().to_string(), source.clone());
    let source_mgr_arc = Arc::new(source_manager);
    let mut parser = Parser::new(&source, file_id, source_mgr_arc.clone());

    // Parse the file
    let module_id = match parser.parse_module() {
        Ok(id) => id,
        Err(error) => {
            // Handle parsing errors - ParseError is a single error, not a vec
            warn!("Parsing error: {error}");

            return Ok((false, 1));
        }
    };

    if verbose {
        info!("✓ Parsing completed successfully");
        info!("Running semantic analysis...");
    }

    // Perform semantic analysis
    let result = analyze_module(parser.ast(), module_id);

    match result {
        Ok(context) => {
            if verbose {
                info!("✓ Type checking completed successfully");

                // Report warnings
                if !context.warnings().is_empty() {
                    warn!("Warnings:");
                    for warning in context.warnings() {
                        // Format and display the warning
                        let warning_msg = format_warning(warning);
                        warn!("{warning_msg}");
                    }
                }
            }

            Ok((true, 0))
        }
        Err(semantic_errors) => {
            // Format and print semantic errors
            warn!("Type checking errors:");
            for error in &semantic_errors {
                warn!("{}", format_semantic_error(error, &source_mgr_arc));
            }

            Ok((false, semantic_errors.len()))
        }
    }
}

/// Formats a semantic error with source context for display
fn format_semantic_error(error: &SemanticError, source_manager: &Arc<SourceManager>) -> String {
    let mut output = String::new();

    // Format the error message
    let _ = writeln!(output, "{}: {error}", "error".red().bold());

    // Add source context if span is available
    if let Some(span) = error.span() {
        // TODO: Improve error location handling when the analyzer provides SourceSpan with file_id
        // For now, we'll use a simplified approach with the existing Span type

        // Assume we're working with the first file
        let file_id = source_manager.get_file(FileID::new(1));

        if let Some(file) = file_id {
            let start_pos = Position::new(1, 1, span.start);
            let end_pos = Position::new(1, 1, span.end);
            let _ = writeln!(output, "  --> {}:{}:{}", file.name, start_pos.line, start_pos.column);
            output.push_str("   |\n");

            // Add the source line
            if let Some(line_text) = source_manager.line_at_position(file.id, start_pos) {
                let _ = writeln!(output, "{:>3} | {line_text}", start_pos.line);

                // Add the error underline
                let column = start_pos.column;
                let underline_spaces = " ".repeat(column - 1);
                let underline_length = if start_pos.line == end_pos.line {
                    (end_pos.column - start_pos.column).max(1)
                } else {
                    line_text.len() - (column - 1)
                };

                let underline = "^".repeat(underline_length);
                let _ = writeln!(output, "    | {underline_spaces}{} {error}", underline.red());
            }

            output.push_str("   |\n");
        }
    }

    output
}

/// Format a dead code warning for display
fn format_warning(warning: &DeadCodeWarning) -> String {
    let severity = match warning.severity {
        WarningSeverity::Info => "info".blue(),
        WarningSeverity::Warning => "warning".yellow(),
    };

    format!("{severity}: {}", warning.message)
}

/// Prints a summary of the type checking results
fn print_error_summary(error_count: usize, file_count: usize, success: bool) {
    info!("\n{} summary:", "Type checking".bold());
    info!("  Files checked: {file_count}");

    if success {
        info!("  {}  No errors found", "✓".green().bold());
    } else {
        info!("  {}  {error_count} error(s) found", "✗".red().bold());
    }
}

/// Resolves which files to check based on input path and flags
fn resolve_input(path: &Path, all: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.try_exists().is_err() {
        anyhow::bail!("Input path '{}' is not a valid file or directory", path.display());
    }

    if path.is_file() {
        // Single file
        files.push(path.to_path_buf());

        return Ok(files);
    }

    if all {
        // Find all .ty files recursively
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            let path = entry.path();
            if let Some(extension) = path.extension()
                && extension == "ty"
            {
                files.push(path.to_path_buf());
            }
        }

        // Sort files for consistent output
        files.sort();
    } else {
        // Look for __main__.ty or __init__.ty
        let main_file = path.join("__main__.ty");
        if main_file.exists() && main_file.is_file() {
            files.push(main_file);

            return Ok(files);
        }

        let init_file = path.join("__init__.ty");
        if init_file.exists() && init_file.is_file() {
            files.push(init_file);

            return Ok(files);
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_resolve_files_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ty");

        fs::write(&file_path, "").unwrap();

        let files = resolve_input(&file_path, false).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);
    }

    #[test]
    fn test_resolve_files_directory_with_main() {
        let dir = tempdir().unwrap();
        let main_file = dir.path().join("__main__.ty");

        fs::write(&main_file, "").unwrap();

        let files = resolve_input(dir.path(), false).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], main_file);
    }

    #[test]
    fn test_resolve_files_all_flag() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("test1.ty");
        let file2 = dir.path().join("test2.ty");
        let subdir = dir.path().join("subdir");

        fs::create_dir(&subdir).unwrap();

        let file3 = subdir.join("test3.ty");

        fs::write(&file1, "").unwrap();
        fs::write(&file2, "").unwrap();
        fs::write(&file3, "").unwrap();

        let resolved = resolve_input(dir.path(), true).unwrap();

        assert_eq!(resolved.len(), 3);
    }
}
