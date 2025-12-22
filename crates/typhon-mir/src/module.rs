//! Module Representation
//!
//! This module defines the MIR module structure, which represents a compilation unit
//! containing functions, global variables, and type definitions.

use crate::function::MIRFunction;
use crate::instr::MIRConst;
use crate::types::{MIRType, TypeID};

/// Type field
#[derive(Debug, Clone)]
pub struct MIRField {
    pub name: String,
    pub ty: MIRType,
    pub offset: usize,
}

/// Global variable
#[derive(Debug, Clone)]
pub struct MIRGlobal {
    pub name: String,
    pub ty: MIRType,
    pub initializer: Option<MIRConst>,
    pub mutable: bool,
}

/// MIR module (compilation unit)
#[derive(Debug, Clone)]
pub struct MIRModule {
    /// Module name
    pub name: String,
    /// Functions
    pub functions: Vec<MIRFunction>,
    /// Global variables
    pub globals: Vec<MIRGlobal>,
    /// Type definitions
    pub types: Vec<MIRTypeDef>,
}

/// Method information for classes
#[derive(Debug, Clone)]
pub struct MethodInfo {
    /// Method name (e.g. `method_name`)
    pub name: String,
    /// Mangled function name in the module (e.g. `ClassName__method_name`)
    pub function_name: String,
    /// Whether this is a static method
    pub is_static: bool,
    /// Whether this is a class method
    pub is_class_method: bool,
    /// Whether this is a private method (underscore-prefixed)
    pub is_private: bool,
    /// Slot index for vtable dispatch
    pub slot_index: usize,
}

/// Type definition (class)
#[derive(Debug, Clone)]
pub struct MIRTypeDef {
    /// Class name
    pub name: String,
    /// Unique type identifier
    pub type_id: TypeID,
    /// Instance fields
    pub fields: Vec<MIRField>,
    /// Methods (including special methods)
    pub methods: Vec<MethodInfo>,
    /// Base classes (in MRO order)
    pub base_classes: Vec<TypeID>,
    /// Whether this is an abstract class
    pub is_abstract: bool,
    /// Constructor method name (`__new__`)
    pub constructor: Option<String>,
    /// Initializer method name (`__init__`)
    pub initializer: Option<String>,
    /// Destructor method name (`__del__`)
    pub destructor: Option<String>,
}
