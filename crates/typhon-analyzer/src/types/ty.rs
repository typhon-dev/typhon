//! Type representation for the Typhon type system.

use std::fmt;

/// Unique identifier for a type.
///
/// `TypeID` is a newtype wrapper around `usize` that uniquely identifies
/// a type within a type environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeID(usize);

impl TypeID {
    /// Creates a new `TypeID` with the given value.
    #[must_use]
    pub const fn new(id: usize) -> Self { Self(id) }

    /// Returns the inner value of the `TypeID`.
    #[must_use]
    pub const fn value(self) -> usize { self.0 }
}

impl fmt::Display for TypeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "type:{}", self.0) }
}

/// Represents a type in the Typhon type system.
///
/// This enum covers all type forms including primitives, collections,
/// functions, classes, unions, and advanced types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// The `Any` type - top type that matches anything.
    Any,
    /// The `bool` type.
    Bool,
    /// The `bytes` type.
    Bytes,
    /// Class type with name and type parameters.
    Class {
        /// Name of the class
        name: String,
        /// Type parameter IDs
        type_params: Vec<TypeID>,
    },
    /// Dictionary type with key and value types.
    Dict(Box<Type>, Box<Type>),
    /// The `float` type.
    Float,
    /// Function type with parameter types and return type.
    Function {
        /// Parameter types
        params: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },
    /// The `int` type.
    Int,
    /// List type with element type.
    List(Box<Type>),
    /// The `Never` type - bottom type that never occurs.
    Never,
    /// The `None` type.
    None,
    /// Optional type (syntactic sugar for `Union[T, None]`).
    Optional(Box<Type>),
    /// Set type with element type.
    Set(Box<Type>),
    /// The `str` type.
    Str,
    /// Tuple type with element types.
    Tuple(Vec<Type>),
    /// Type variable for generics.
    TypeVar(String),
    /// Union type with multiple possible types.
    Union(Vec<Type>),
}

impl Type {
    /// Gets the attribute with the given name from this type.
    ///
    /// Returns the attribute's type if it exists.
    #[must_use]
    pub const fn get_attribute(&self, _name: &str) -> Option<Self> {
        match self {
            // TODO: Class types would look up attributes from their definitions
            Self::Class { .. }
            | Self::Any
            | Self::Bool
            | Self::Bytes
            | Self::Dict(_, _)
            | Self::Float
            | Self::Function { .. }
            | Self::Int
            | Self::List(_)
            | Self::Never
            | Self::None
            | Self::Optional(_)
            | Self::Set(_)
            | Self::Str
            | Self::Tuple(_)
            | Self::TypeVar(_)
            | Self::Union(_) => None,
        }
    }

    /// Gets the method with the given name from this type.
    ///
    /// Returns the method's function type if it exists.
    #[must_use]
    pub fn get_method(&self, name: &str) -> Option<Self> {
        match self {
            Self::List(elem_type) => Self::get_list_method(name, elem_type.as_ref()),
            Self::Dict(key_type, val_type) => {
                Self::get_dict_method(name, key_type.as_ref(), val_type.as_ref())
            }
            Self::Str => Self::get_str_method(name),
            Self::Set(elem_type) => Self::get_set_method(name, elem_type.as_ref()),
            // TODO: Class types would look up methods from their definitions
            Self::Class { .. }
            | Self::Any
            | Self::Bool
            | Self::Bytes
            | Self::Float
            | Self::Function { .. }
            | Self::Int
            | Self::Never
            | Self::None
            | Self::Optional(_)
            | Self::Tuple(_)
            | Self::TypeVar(_)
            | Self::Union(_) => None,
        }
    }

    /// Returns true if this type is compatible with the other type.
    ///
    /// Two types are compatible if either is a subtype of the other.
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.is_subtype_of(other) || other.is_subtype_of(self)
    }

    /// Returns true if this is a numeric type (int or float).
    #[must_use]
    pub const fn is_numeric(&self) -> bool { matches!(self, Self::Int | Self::Float) }

    /// Returns true if this is a subtype of the other type.
    ///
    /// This implements basic subtyping rules:
    ///
    /// - Reflexivity: T <: T
    /// - None <: Optional[T]
    /// - T <: Optional[T]
    /// - T <: Union[...] if T is one of the union members
    #[must_use]
    pub fn is_subtype_of(&self, other: &Self) -> bool {
        // Reflexivity: T <: T
        if self == other {
            return true;
        }

        match (self, other) {
            // None is a subtype of Optional[T]
            // Any is the top type - everything is a subtype of Any
            (Self::None, Self::Optional(_)) | (_, Self::Any) => true,

            // T <: Optional[T]
            (t, Self::Optional(opt_t)) if t == opt_t.as_ref() => true,

            // Union subtyping: T <: Union[...] if T is one of the union members
            (t, Self::Union(members)) => members.iter().any(|m| t.is_subtype_of(m)),

            // Never is the bottom type - it's a subtype of everything
            (Self::Never, _) => true,

            _ => false,
        }
    }

    /// Substitutes type variables in this type with their concrete types.
    ///
    /// This is used during type inference to replace type variables with
    /// their inferred concrete types.
    #[must_use]
    pub fn substitute(&self, substitutions: &std::collections::HashMap<String, Self>) -> Self {
        match self {
            Self::TypeVar(name) => {
                // Look up the substitution for this type variable
                substitutions.get(name).cloned().unwrap_or_else(|| self.clone())
            }
            Self::List(elem) => Self::List(Box::new(elem.substitute(substitutions))),
            Self::Set(elem) => Self::Set(Box::new(elem.substitute(substitutions))),
            Self::Dict(key, val) => Self::Dict(
                Box::new(key.substitute(substitutions)),
                Box::new(val.substitute(substitutions)),
            ),
            Self::Optional(inner) => Self::Optional(Box::new(inner.substitute(substitutions))),
            Self::Union(types) => {
                Self::Union(types.iter().map(|t| t.substitute(substitutions)).collect())
            }
            Self::Tuple(types) => {
                Self::Tuple(types.iter().map(|t| t.substitute(substitutions)).collect())
            }
            Self::Function { params, return_type } => Self::Function {
                params: params.iter().map(|t| t.substitute(substitutions)).collect(),
                return_type: Box::new(return_type.substitute(substitutions)),
            },
            // Other types don't contain type variables, return as-is
            _ => self.clone(),
        }
    }

    /// Attempts to unify this type with another, returning the most specific common type.
    ///
    /// Unification finds a type that both types can be considered instances of.
    /// Returns None if the types cannot be unified.
    #[must_use]
    pub fn unify(&self, other: &Self) -> Option<Self> {
        // If types are equal, return one of them
        if self == other {
            return Some(self.clone());
        }

        // Handle Any as the top type
        if matches!(self, Self::Any) {
            return Some(other.clone());
        }

        if matches!(other, Self::Any) {
            return Some(self.clone());
        }

        // Handle Never as the bottom type
        if matches!(self, Self::Never) {
            return Some(other.clone());
        }

        if matches!(other, Self::Never) {
            return Some(self.clone());
        }

        // Numeric type unification: int + float = float
        match (self, other) {
            (Self::Int, Self::Float) | (Self::Float, Self::Int) => return Some(Self::Float),
            _ => {}
        }

        // Try subtyping relationships
        if self.is_subtype_of(other) {
            return Some(other.clone());
        }

        if other.is_subtype_of(self) {
            return Some(self.clone());
        }

        // Create union type as a fallback
        Some(Self::Union(vec![self.clone(), other.clone()]))
    }

    /// Gets a dict method by name.
    fn get_dict_method(name: &str, key_type: &Self, val_type: &Self) -> Option<Self> {
        match name {
            "clear" => Some(Self::Function { params: vec![], return_type: Box::new(Self::None) }),
            "copy" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::Dict(
                    Box::new(key_type.clone()),
                    Box::new(val_type.clone()),
                )),
            }),
            "get" => Some(Self::Function {
                params: vec![key_type.clone()],
                return_type: Box::new(Self::Optional(Box::new(val_type.clone()))),
            }),
            "items" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::List(Box::new(Self::Tuple(vec![
                    key_type.clone(),
                    val_type.clone(),
                ])))),
            }),
            "keys" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::List(Box::new(key_type.clone()))),
            }),
            "pop" => Some(Self::Function {
                params: vec![key_type.clone()],
                return_type: Box::new(val_type.clone()),
            }),
            "popitem" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::Tuple(vec![key_type.clone(), val_type.clone()])),
            }),
            "setdefault" => Some(Self::Function {
                params: vec![key_type.clone(), val_type.clone()],
                return_type: Box::new(val_type.clone()),
            }),
            "update" => Some(Self::Function {
                params: vec![Self::Dict(Box::new(key_type.clone()), Box::new(val_type.clone()))],
                return_type: Box::new(Self::None),
            }),
            "values" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::List(Box::new(val_type.clone()))),
            }),
            _ => None,
        }
    }

    /// Gets a list method by name.
    fn get_list_method(name: &str, elem_type: &Self) -> Option<Self> {
        match name {
            "append" | "remove" => Some(Self::Function {
                params: vec![elem_type.clone()],
                return_type: Box::new(Self::None),
            }),
            "clear" | "reverse" | "sort" => {
                Some(Self::Function { params: vec![], return_type: Box::new(Self::None) })
            }
            "copy" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::List(Box::new(elem_type.clone()))),
            }),
            "count" | "index" => Some(Self::Function {
                params: vec![elem_type.clone()],
                return_type: Box::new(Self::Int),
            }),
            "extend" => Some(Self::Function {
                params: vec![Self::List(Box::new(elem_type.clone()))],
                return_type: Box::new(Self::None),
            }),
            "insert" => Some(Self::Function {
                params: vec![Self::Int, elem_type.clone()],
                return_type: Box::new(Self::None),
            }),
            "pop" => {
                Some(Self::Function { params: vec![], return_type: Box::new(elem_type.clone()) })
            }
            _ => None,
        }
    }

    /// Gets a set method by name.
    fn get_set_method(name: &str, elem_type: &Self) -> Option<Self> {
        match name {
            "add" | "discard" | "remove" => Some(Self::Function {
                params: vec![elem_type.clone()],
                return_type: Box::new(Self::None),
            }),
            "clear" => Some(Self::Function { params: vec![], return_type: Box::new(Self::None) }),
            "copy" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::Set(Box::new(elem_type.clone()))),
            }),
            "difference" | "intersection" | "symmetric_difference" | "union" => {
                Some(Self::Function {
                    params: vec![Self::Set(Box::new(elem_type.clone()))],
                    return_type: Box::new(Self::Set(Box::new(elem_type.clone()))),
                })
            }
            "difference_update"
            | "intersection_update"
            | "symmetric_difference_update"
            | "update" => Some(Self::Function {
                params: vec![Self::Set(Box::new(elem_type.clone()))],
                return_type: Box::new(Self::None),
            }),
            "isdisjoint" | "issubset" | "issuperset" => Some(Self::Function {
                params: vec![Self::Set(Box::new(elem_type.clone()))],
                return_type: Box::new(Self::Bool),
            }),
            "pop" => {
                Some(Self::Function { params: vec![], return_type: Box::new(elem_type.clone()) })
            }
            _ => None,
        }
    }

    /// Gets a str method by name.
    fn get_str_method(name: &str) -> Option<Self> {
        match name {
            "capitalize" | "casefold" | "lower" | "lstrip" | "rstrip" | "strip" | "swapcase"
            | "title" | "upper" => {
                Some(Self::Function { params: vec![], return_type: Box::new(Self::Str) })
            }
            "center" | "ljust" | "rjust" => {
                Some(Self::Function { params: vec![Self::Int], return_type: Box::new(Self::Str) })
            }
            "count" | "find" | "index" | "rfind" | "rindex" => {
                Some(Self::Function { params: vec![Self::Str], return_type: Box::new(Self::Int) })
            }
            "endswith" | "startswith" => {
                Some(Self::Function { params: vec![Self::Str], return_type: Box::new(Self::Bool) })
            }
            "isalnum" | "isalpha" | "isascii" | "isdecimal" | "isdigit" | "islower"
            | "isnumeric" | "isspace" | "istitle" | "isupper" => {
                Some(Self::Function { params: vec![], return_type: Box::new(Self::Bool) })
            }
            "join" => Some(Self::Function {
                params: vec![Self::List(Box::new(Self::Str))],
                return_type: Box::new(Self::Str),
            }),
            "replace" => Some(Self::Function {
                params: vec![Self::Str, Self::Str],
                return_type: Box::new(Self::Str),
            }),
            "split" | "rsplit" | "splitlines" => Some(Self::Function {
                params: vec![],
                return_type: Box::new(Self::List(Box::new(Self::Str))),
            }),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => write!(f, "Any"),
            Self::Bool => write!(f, "bool"),
            Self::Bytes => write!(f, "bytes"),
            Self::Class { name, type_params } => {
                if type_params.is_empty() {
                    write!(f, "{name}")
                } else {
                    write!(
                        f,
                        "{name}[{}]",
                        type_params.iter().map(|id| format!("{id}")).collect::<Vec<_>>().join(", ")
                    )
                }
            }
            Self::Dict(k, v) => write!(f, "dict[{k}, {v}]"),
            Self::Float => write!(f, "float"),
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
            Self::Int => write!(f, "int"),
            Self::List(elem) => write!(f, "list[{elem}]"),
            Self::Never => write!(f, "Never"),
            Self::None => write!(f, "None"),
            Self::Optional(inner) => write!(f, "{inner} | None"),
            Self::Set(elem) => write!(f, "set[{elem}]"),
            Self::Str => write!(f, "str"),
            Self::Tuple(elems) => {
                write!(f, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{elem}")?;
                }
                write!(f, ")")
            }
            Self::TypeVar(name) => write!(f, "{name}"),
            Self::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{ty}")?;
                }
                Ok(())
            }
        }
    }
}
