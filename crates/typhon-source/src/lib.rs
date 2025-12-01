//! Source file handling and position tracking for the Typhon programming language.
//!
//! This crate provides the fundamental types and utilities for working with source code
//! in the Typhon compiler pipeline. It handles source file representation, content access,
//! and precise location tracking through spans and positions.
//!
//! The crate consists of two main modules:
//! - [`source`]: Manages source files, content storage, and line/column tracking
//! - [`span`]: Provides position and span tracking for accurate source locations
//!
//! Together, these modules form the foundation for source code representation used
//! throughout the compiler, enabling precise error reporting and diagnostics.

pub mod types;
