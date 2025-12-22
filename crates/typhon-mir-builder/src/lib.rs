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
//! 4. **SSA construction** ([`ssa`]) - Post-processing to insert phi nodes
//! 5. **Optimization** ([`optimize`]) - Basic optimizations like constant folding
//!
//! ## Example
//!
//! ```rust,ignore
//! use typhon_ast::ast::AST;
//! use typhon_mir_builder::context::LoweringContext;
//!
//! let ast = AST::new();
//! let mut ctx = LoweringContext::new(&ast, "my_module".to_string());
//!
//! // Lower the AST to MIR
//! ctx.lower_module(module_node)?;
//!
//! // Get the resulting MIR module
//! let mir_module = ctx.build();
//! ```

pub mod class;
pub mod closure;
pub mod context;
pub mod error;
pub mod expr;
pub mod function;
pub mod stmt;
