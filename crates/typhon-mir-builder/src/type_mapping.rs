//! Type mapping between analyzer types and MIR types.
//!
//! This module handles conversion from [`Type`] to [`MIRType`].

use typhon_analyzer::types::Type;
use typhon_mir::types::MIRType;

/// Context for type conversion operations.
///
/// Provides additional information needed during type mapping.
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeConversionContext;

impl TypeConversionContext {
    /// Creates a new type conversion context.
    #[must_use]
    pub const fn new() -> Self { Self }
}

/// Trait for converting analyzer types to MIR types.
///
/// This trait is similar to `From` but is defined locally to satisfy
/// Rust's orphan rules, since both `Type` and `MIRType` are defined
/// in external crates.
pub trait ToMIRType {
    /// Converts this type to a MIR type.
    fn to_mir_type(&self) -> MIRType;
}

impl ToMIRType for Type {
    fn to_mir_type(&self) -> MIRType {
        match self {
            Self::Int => MIRType::Int,
            Self::Float => MIRType::Float,
            Self::Bool => MIRType::Bool,
            Self::Str => MIRType::Str,
            Self::None => MIRType::None,

            Self::Bytes | Self::Any | Self::Never | Self::TypeVar(_) => {
                MIRType::Object { type_id: None }
            }

            Self::List(elem_ty) => {
                let elem = elem_ty.to_mir_type();
                MIRType::List(Box::new(elem))
            }

            Self::Dict(key_ty, val_ty) => {
                let key = key_ty.to_mir_type();
                let val = val_ty.to_mir_type();
                MIRType::Dict { key: Box::new(key), value: Box::new(val) }
            }

            Self::Tuple(elem_tys) => {
                let elems = elem_tys.iter().map(ToMIRType::to_mir_type).collect();
                MIRType::Tuple(elems)
            }

            Self::Set(elem_ty) => {
                // MIR doesn't have a Set type yet, so we map to Object
                // Will be enhanced when MIR adds Set support
                let _ = elem_ty;
                MIRType::Object { type_id: None }
            }

            Self::Function { params, return_type } => {
                let param_types = params.iter().map(ToMIRType::to_mir_type).collect();
                let ret = Box::new(return_type.to_mir_type());
                MIRType::Function { params: param_types, return_type: ret }
            }

            Self::Class { name, type_params } => {
                // Map class types to Object for now
                // Will be refined when we add proper class type support to MIR
                let _ = (name, type_params);
                MIRType::Object { type_id: None }
            }

            Self::Union(_types) => {
                // Unions mapped to Object - MIR doesn't have union types yet
                MIRType::Object { type_id: None }
            }

            Self::Optional(inner) => {
                // Optional[T] becomes T | None in MIR
                // For now map to Object, will enhance when MIR supports unions
                let _ = inner;
                MIRType::Object { type_id: None }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_conversion() {
        assert_eq!(Type::Int.to_mir_type(), MIRType::Int);
        assert_eq!(Type::Float.to_mir_type(), MIRType::Float);
        assert_eq!(Type::Bool.to_mir_type(), MIRType::Bool);
        assert_eq!(Type::Str.to_mir_type(), MIRType::Str);
        assert_eq!(Type::None.to_mir_type(), MIRType::None);
    }

    #[test]
    fn test_list_type_conversion() {
        let list_int = Type::List(Box::new(Type::Int));
        let mir_type = list_int.to_mir_type();

        match mir_type {
            MIRType::List(elem) => assert_eq!(*elem, MIRType::Int),
            _ => panic!("Expected List type"),
        }
    }

    #[test]
    fn test_dict_type_conversion() {
        let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        let mir_type = dict.to_mir_type();

        match mir_type {
            MIRType::Dict { key, value } => {
                assert_eq!(*key, MIRType::Str);
                assert_eq!(*value, MIRType::Int);
            }
            _ => panic!("Expected Dict type"),
        }
    }

    #[test]
    fn test_tuple_type_conversion() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]);
        let mir_type = &tuple.to_mir_type();

        match mir_type {
            MIRType::Tuple(elems) => {
                assert_eq!(elems.len(), 3);
                assert_eq!(elems[0], MIRType::Int);
                assert_eq!(elems[1], MIRType::Str);
                assert_eq!(elems[2], MIRType::Bool);
            }
            _ => panic!("Expected Tuple type"),
        }
    }

    #[test]
    fn test_function_type_conversion() {
        let func = Type::Function {
            params: vec![Type::Int, Type::Str],
            return_type: Box::new(Type::Bool),
        };
        let mir_type = func.to_mir_type();

        match mir_type {
            MIRType::Function { params, return_type } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], MIRType::Int);
                assert_eq!(params[1], MIRType::Str);
                assert_eq!(*return_type, MIRType::Bool);
            }
            _ => panic!("Expected Function type"),
        }
    }

    #[test]
    fn test_class_type_conversion() {
        let class = Type::Class { name: "MyClass".to_string(), type_params: vec![] };
        let mir_type = class.to_mir_type();

        assert_eq!(mir_type, MIRType::Object { type_id: None });
    }

    #[test]
    fn test_unknown_type_conversion() {
        assert_eq!(Type::Any.to_mir_type(), MIRType::Object { type_id: None });
        assert_eq!(Type::Never.to_mir_type(), MIRType::Object { type_id: None });
    }

    #[test]
    fn test_union_type_conversion() {
        let union = Type::Union(vec![Type::Int, Type::Str]);
        let mir_type = union.to_mir_type();

        assert_eq!(mir_type, MIRType::Object { type_id: None });
    }

    #[test]
    fn test_optional_type_conversion() {
        let optional = Type::Optional(Box::new(Type::Int));
        let mir_type = optional.to_mir_type();

        assert_eq!(mir_type, MIRType::Object { type_id: None });
    }

    #[test]
    fn test_nested_type_conversion() {
        // List[Dict[Str, Int]]
        let nested = Type::List(Box::new(Type::Dict(Box::new(Type::Str), Box::new(Type::Int))));
        let mir_type = nested.to_mir_type();

        match mir_type {
            MIRType::List(elem) => match *elem {
                MIRType::Dict { key, value } => {
                    assert_eq!(*key, MIRType::Str);
                    assert_eq!(*value, MIRType::Int);
                }
                _ => panic!("Expected Dict inside List"),
            },
            _ => panic!("Expected List type"),
        }
    }
}
