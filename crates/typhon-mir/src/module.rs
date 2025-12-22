//! Module Representation
//!
//! This module defines the MIR module structure, which represents a compilation unit
//! containing functions, global variables, and type definitions.

use rustc_hash::FxHashMap;

use crate::function::MIRFunction;
use crate::instr::{MIRConst, ValueID};
use crate::types::{MIRType, TypeID};

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
    /// Value-to-name mapping for function resolution during optimization
    pub value_names: FxHashMap<ValueID, String>,
}

impl MIRModule {
    /// Get the name associated with a value ID, if any
    #[must_use]
    pub fn get_value_name(&self, value_id: ValueID) -> Option<&str> {
        self.value_names.get(&value_id).map(String::as_str)
    }

    /// Set the name for a value ID
    pub fn set_value_name(&mut self, value_id: ValueID, name: String) {
        self.value_names.insert(value_id, name);
    }
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
