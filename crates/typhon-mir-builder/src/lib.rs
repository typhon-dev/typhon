//! AST to MIR lowering for the Typhon compiler.
//!
//! This crate transforms the high-level, typed Abstract Syntax Tree (AST) from semantic
//! analysis into Typhon's Mid-level Intermediate Representation (MIR) in SSA form.
//!
//! ## Architecture
//!
//! The lowering process follows a single-pass strategy with post-processing for SSA:
//!
//! 1. **Expression lowering** ([`expr`]) - Bottom-up lowering of expressions
//! 2. **Statement lowering** ([`stmt`]) - Top-down lowering of statements with control flow
//! 3. **Function lowering** ([`function`]) - Complete function lowering
//! 4. **Class lowering** ([`class`]) - Class definitions with method tables
//! 5. **Closure lowering** ([`closure`]) - Nested functions with captured variables
//! 6. **SSA construction** - Post-processing to insert phi nodes (future work)
//! 7. **Optimization** - Basic optimizations like constant folding (see `typhon-mir-optimizer`)
//!
//! ## Semantic Analysis Integration
//!
//! This crate integrates with `typhon-analyzer` to leverage type information and symbol
//! resolution during lowering. When semantic context is available:
//!
//! - **Type queries**: Access precise type information for expressions and names
//! - **Name classification**: Determine whether names are local, global, captured, or builtin
//! - **Symbol resolution**: Look up symbols in the symbol table for accurate code generation
//!
//! See [`context::LoweringContext`] for the integration API and the [`symbol_resolution`]
//! module for name classification logic.
//!
//! ## Examples
//!
//! ### Basic Usage Without Semantic Analysis
//!
//! ```rust,ignore
//! use typhon_ast::ast::AST;
//! use typhon_mir_builder::context::LoweringContext;
//!
//! let ast = AST::new();
//! let mut ctx = LoweringContext::new(&ast, "my_module".to_string());
//!
//! // Lower functions from the module
//! for func_id in module.functions {
//!     ctx.lower_function(func_id)?;
//! }
//!
//! // Get the resulting MIR module
//! let mir_module = ctx.build();
//! ```
//!
//! ### With Semantic Analysis (Recommended)
//!
//! ```rust,ignore
//! use typhon_analyzer::Analyzer;
//! use typhon_ast::ast::AST;
//! use typhon_mir_builder::context::LoweringContext;
//!
//! // Parse and analyze the source
//! let ast = AST::new();
//! let mut analyzer = Analyzer::new(&ast);
//! analyzer.analyze_module(module_id)?;
//! let analysis_ctx = analyzer.context();
//!
//! // Create lowering context with semantic information
//! let mut ctx = LoweringContext::new_with_semantics(
//!     &ast,
//!     "my_module".to_string(),
//!     analysis_ctx,
//! );
//!
//! // Lower with type information and symbol resolution
//! for func_id in module.functions {
//!     ctx.lower_function(func_id)?;
//! }
//!
//! let mir_module = ctx.build();
//! ```

pub mod class;
pub mod closure;
pub mod context;
pub mod error;
pub mod expr;
pub mod function;
pub mod stmt;
pub mod symbol_resolution;
pub mod type_mapping;
