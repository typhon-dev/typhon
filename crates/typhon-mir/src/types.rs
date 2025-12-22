//! MIR Type System
//!
//! This module defines the type system for Typhon's Mid-level Intermediate Representation.
//! Types are used throughout MIR for validation and optimization.

use std::fmt;

/// Type ID from semantic analysis
pub type TypeID = usize;

/// Type representation in MIR
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MIRType {
    /// Boolean type
    Bool,
    /// Closure type (function + environment)
    Closure { params: Vec<Self>, return_type: Box<Self>, captured: Vec<Self> },
    /// Dict type
    Dict { key: Box<Self>, value: Box<Self> },
    /// Float type (f64)
    Float,
    /// Function type
    Function { params: Vec<Self>, return_type: Box<Self> },
    /// Integer type (arbitrary precision)
    Int,
    /// List type
    List(Box<Self>),
    /// None type
    None,
    /// Boxed Python object (runtime-typed)
    Object {
        /// Type ID from semantic analysis (None = unknown/dynamic)
        type_id: Option<TypeID>,
    },
    /// Reference to a value (for mutability)
    Ref(Box<Self>),
    /// String type (UTF-8)
    Str,
    /// Tuple type
    Tuple(Vec<Self>),
    /// Void type (no value)
    Void,
}

impl fmt::Display for MIRType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "Bool"),
            Self::Closure { params, return_type, captured } => {
                write!(f, "Closure<[")?;
                for (i, cap) in captured.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{cap}")?;
                }
                write!(f, "]>(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{param}")?;
                }

                write!(f, ") -> {return_type}>")
            }
            Self::Dict { key, value } => write!(f, "Dict<{key}, {value}>"),
            Self::Float => write!(f, "Float"),
            Self::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{param}")?;
                }

                write!(f, ") -> {return_type}")
            }
            Self::Int => write!(f, "Int"),
            Self::List(elem) => write!(f, "List<{elem}>"),
            Self::None => write!(f, "None"),
            Self::Object { type_id } => {
                if let Some(id) = type_id {
                    write!(f, "Object({id})")
                } else {
                    write!(f, "Object")
                }
            }
            Self::Ref(inner) => write!(f, "Ref<{inner}>"),
            Self::Str => write!(f, "Str"),
            Self::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{ty}")?;
                }

                write!(f, ")")
            }
            Self::Void => write!(f, "Void"),
        }
    }
}
