//! Build command implementation
//!
//! This module implements the full compilation pipeline:
//! Source → Parser → AST → Analyzer → MIR Builder → LLVM Codegen

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use log::{debug, info, warn};
use typhon_ast::nodes::AnyNode;
use typhon_codegen_llvm::compile_module;
use typhon_mir_builder::context::LoweringContext;
use typhon_mir_optimizer::config::OptimizerConfig;
use typhon_mir_optimizer::optimize_module_with_config;
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

/// Build a Typhon project or file
#[allow(clippy::too_many_arguments)]
pub fn execute(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    emit_llvm: bool,
    opt_level: u8,
    release: bool,
    _verbose: bool,
    module: Option<&str>,
) -> Result<()> {
    let input_path = input.unwrap_or_else(|| PathBuf::from("."));

    // Resolve the actual file to compile
    let source_file = resolve_input_file(&input_path, module)?;

    info!("Building: {}", source_file.display());
    if let Some(ref out) = output {
        debug!("Output: {}", out.display());
    }

    debug!("Optimization level: {opt_level}");
    debug!("Release mode: {release}");
    debug!("Emit LLVM IR: {emit_llvm}");

    // Step 1: Read source file
    let source = fs::read_to_string(&source_file)
        .with_context(|| format!("Failed to read input file: {}", source_file.display()))?;

    debug!("Source file read successfully ({} bytes)", source.len());

    // Step 2: Parse source → AST
    let mut source_manager = SourceManager::new();
    let file_id =
        source_manager.add_file(source_file.to_string_lossy().to_string(), source.clone());
    let mut parser = Parser::new(&source, file_id, Arc::new(source_manager));

    info!("Parsing source code...");

    let module_id =
        parser.parse_module().context("Failed to parse module - syntax errors detected")?;

    info!("✓ Parsing completed successfully");

    // Step 3: Lower AST → MIR
    info!("Lowering AST to MIR...");

    let module_name =
        source_file.file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string();

    let mut lowering_context = LoweringContext::new(parser.ast(), module_name);

    // Get the module node and lower all function declarations
    if let Some(module_node) = parser.ast().get_node(module_id)
        && let AnyNode::Module(module) = &module_node.data
    {
        for &stmt_id in &module.statements {
            if let Some(stmt_node) = parser.ast().get_node(stmt_id)
                && let AnyNode::FunctionDecl(_) = &stmt_node.data
            {
                lowering_context
                    .lower_function(stmt_id)
                    .context("Failed to lower function to MIR")?;
            }
        }
    }

    let mut mir_module = lowering_context.build();

    info!("✓ MIR generation completed");
    debug!("  Functions: {}", mir_module.functions.len());

    // Step 3.5: Optimize MIR
    if opt_level > 0 {
        info!("Running MIR optimization passes (level {opt_level})...");

        let optimizer_config = match opt_level {
            1 => OptimizerConfig::level1(),
            2 => OptimizerConfig::level2(),
            3.. => OptimizerConfig::level3(),
            _ => OptimizerConfig::level0(),
        };

        optimize_module_with_config(&mut mir_module, &optimizer_config)
            .context("Failed to optimize MIR")?;

        info!("✓ MIR optimization completed");
    } else {
        debug!("Optimization disabled (level 0)");
    }

    // Step 4: Compile MIR → LLVM IR
    info!("Generating LLVM IR...");

    let llvm_ir = compile_module(&mir_module).context("Failed to compile MIR to LLVM IR")?;

    info!("✓ LLVM IR generation completed");
    debug!("  IR size: {} bytes", llvm_ir.len());

    // Step 5: Write output
    let output_path = determine_output_path(&source_file, output, emit_llvm);

    if emit_llvm {
        // Write LLVM IR
        fs::write(&output_path, llvm_ir)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        info!("✓ LLVM IR written to: {}", output_path.display());
    } else {
        // For now, still write LLVM IR and inform user
        // TODO: Implement full executable generation via LLVM
        fs::write(&output_path, llvm_ir)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        warn!("Full executable generation not yet implemented");
        info!("LLVM IR written to: {}", output_path.display());
        info!(
            "To compile to executable, use: clang {} -o {}",
            output_path.display(),
            source_file.file_stem().and_then(|s| s.to_str()).unwrap_or("output")
        );
    }

    info!("✓ Build completed successfully!");

    Ok(())
}

/// Determine the output path based on input, explicit output, and `emit_llvm` flag
fn determine_output_path(input: &Path, output: Option<PathBuf>, emit_llvm: bool) -> PathBuf {
    if let Some(path) = output {
        return path;
    }

    // Generate default output path based on input
    let mut output_path = input.to_path_buf();

    if emit_llvm {
        output_path.set_extension("ll");
    } else {
        // For executable, remove extension
        output_path.set_extension("");
    }

    output_path
}

/// Resolve the input file from a path and optional module specifier
///
/// Supports:
///
/// - Direct file path: `file.ty` → `file.ty`
/// - Directory with `__main__.ty`: `dir/` → `dir/__main__.ty`
/// - Directory with `__init__.ty`: `dir/` → `dir/__init__.ty`
/// - Module path: `-m path.to.module` → `path/to/module.ty` or `path/to/module/__init__.ty`
fn resolve_input_file(path: &Path, module: Option<&str>) -> Result<PathBuf> {
    // If a module path is provided, convert it to a file path
    if let Some(mod_path) = module {
        let file_path = PathBuf::from(mod_path.replace('.', "/"));

        // Try module.ty first
        let ty_file = file_path.with_extension("ty");
        if ty_file.exists() && ty_file.is_file() {
            return Ok(ty_file);
        }

        // Try module/__init__.ty
        let init_file = file_path.join("__init__.ty");
        if init_file.exists() && init_file.is_file() {
            return Ok(init_file);
        }

        anyhow::bail!(
            "Module '{}' not found. Tried:\n  - {}\n  - {}",
            mod_path,
            ty_file.display(),
            init_file.display()
        );
    }

    // If it's a file, use it directly
    if path.is_file() {
        return Ok(path.to_path_buf());
    }

    // If it's a directory, look for __main__.ty or __init__.ty
    if path.is_dir() {
        let main_file = path.join("__main__.ty");
        if main_file.exists() && main_file.is_file() {
            return Ok(main_file);
        }

        let init_file = path.join("__init__.ty");
        if init_file.exists() && init_file.is_file() {
            return Ok(init_file);
        }

        anyhow::bail!("Directory '{}' does not contain __main__.ty or __init__.ty", path.display());
    }

    anyhow::bail!("Input path '{}' is not a valid file or directory", path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_output_path_llvm() {
        let input = PathBuf::from("test.ty");

        // Test explicit output
        let explicit = Some(PathBuf::from("custom.ll"));
        assert_eq!(determine_output_path(&input, explicit, true), PathBuf::from("custom.ll"));

        // Test default LLVM IR output
        assert_eq!(determine_output_path(&input, None, true), PathBuf::from("test.ll"));
    }

    #[test]
    fn test_determine_output_path_executable() {
        let input = PathBuf::from("test.ty");

        // Test default executable output
        assert_eq!(determine_output_path(&input, None, false), PathBuf::from("test"));
    }
}
