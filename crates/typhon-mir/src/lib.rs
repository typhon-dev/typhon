//! Mid-level Intermediate Representation (MIR) for the Typhon compiler.
//!
//! MIR is a typed, SSA-form intermediate representation that preserves Typhon semantics
//! while being low-level enough for optimization and efficient code generation.
//!
//! ## Overview
//!
//! The MIR consists of:
//!
//! - **Types** ([`MIRType`](types::MIRType)): A type system representing Typhon types
//! - **Instructions** ([`MIRInstr`](instr::MIRInstr)): Operations on values (arithmetic, memory, calls)
//! - **Basic Blocks** ([`BasicBlock`](block::BasicBlock)): Single-entry, single-exit code blocks
//! - **Functions** ([`MIRFunction`](function::MIRFunction)): Complete function definitions
//! - **Modules** ([`MIRModule`](module::MIRModule)): Compilation units
//!
//! ## SSA Form
//!
//! MIR uses Static Single Assignment (SSA) form where each value is assigned exactly once.
//! This enables powerful dataflow optimizations like constant folding, dead code elimination,
//! and escape analysis.
//!
//! ## Builder API
//!
//! The [`builder::FunctionBuilder`] provides an ergonomic API for constructing MIR:
//!
//! ```rust,ignore
//! use typhon_mir::*;
//!
//! let mut builder = FunctionBuilder::new(
//!     "add".to_string(),
//!     vec![
//!         ("a".to_string(), MIRType::Int),
//!         ("b".to_string(), MIRType::Int),
//!     ],
//!     MIRType::Int,
//! );
//!
//! let entry = builder.create_block();
//! builder.switch_to_block(entry);
//!
//! let r0 = builder.allocate_register(MIRType::Int);
//! builder.add_instruction(Instruction::Add {
//!     result: r0.clone(),
//!     left: Operand::Register(Register { id: 0, ty: MIRType::Int }),
//!     right: Operand::Register(Register { id: 1, ty: MIRType::Int }),
//! });
//!
//! builder.set_terminator(Terminator::Return {
//!     value: Some(Operand::Register(r0)),
//! });
//!
//! let function = builder.build();
//! ```

pub mod block;
pub mod builder;
pub mod function;
pub mod instr;
pub mod module;
pub mod pretty;
pub mod types;
