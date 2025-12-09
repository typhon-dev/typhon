//! Symbol table management for semantic analysis.
//!
//! This module provides the infrastructure for managing symbols, scopes, and name resolution
//! during semantic analysis. It includes:
//!
//! - [`Scope`]: Represents a lexical scope in the program
//! - [`Symbol`]: Represents a declared symbol (variable, function, class, etc.)
//! - [`SymbolTable`]: The main symbol table managing all scopes and symbols

mod scope;
mod table;
mod types;

pub use scope::*;
pub use table::*;
pub use types::*;

/// Common Python builtins that are always available.
pub const BUILTINS: &[&str] = &[
    "abs",
    "all",
    "any",
    "bin",
    "bool",
    "chr",
    "dict",
    "dir",
    "divmod",
    "enumerate",
    "filter",
    "float",
    "hex",
    "id",
    "input",
    "int",
    "isinstance",
    "issubclass",
    "iter",
    "len",
    "list",
    "map",
    "max",
    "min",
    "next",
    "oct",
    "open",
    "ord",
    "pow",
    "print",
    "range",
    "repr",
    "reversed",
    "round",
    "set",
    "sorted",
    "str",
    "sum",
    "tuple",
    "type",
    "zip",
];
