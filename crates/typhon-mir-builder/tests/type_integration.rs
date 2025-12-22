//! Tests for type integration between analyzer and MIR builder

use typhon_analyzer::types::Type;
use typhon_mir::types::MIRType;
use typhon_mir_builder::type_mapping::ToMIRType;

#[test]
fn test_primitive_type_mapping() {
    assert_eq!(Type::Int.to_mir_type(), MIRType::Int);
    assert_eq!(Type::Float.to_mir_type(), MIRType::Float);
    assert_eq!(Type::Bool.to_mir_type(), MIRType::Bool);
    assert_eq!(Type::Str.to_mir_type(), MIRType::Str);
    assert_eq!(Type::None.to_mir_type(), MIRType::None);
}

#[test]
fn test_list_type_mapping() {
    let list_int = Type::List(Box::new(Type::Int));
    let mir_type = list_int.to_mir_type();

    match mir_type {
        MIRType::List(elem) => assert_eq!(*elem, MIRType::Int),
        _ => panic!("Expected List type, got {mir_type:?}"),
    }
}

#[test]
fn test_dict_type_mapping() {
    let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let mir_type = dict.to_mir_type();

    match mir_type {
        MIRType::Dict { key, value } => {
            assert_eq!(*key, MIRType::Str);
            assert_eq!(*value, MIRType::Int);
        }
        _ => panic!("Expected Dict type, got {mir_type:?}"),
    }
}

#[test]
fn test_tuple_type_mapping() {
    let tuple = Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]);
    let mir_type = tuple.to_mir_type();

    match mir_type {
        MIRType::Tuple(elems) => {
            assert_eq!(elems.len(), 3);
            assert_eq!(elems[0], MIRType::Int);
            assert_eq!(elems[1], MIRType::Str);
            assert_eq!(elems[2], MIRType::Bool);
        }
        _ => panic!("Expected Tuple type, got {mir_type:?}"),
    }
}

#[test]
fn test_function_type_mapping() {
    let func =
        Type::Function { params: vec![Type::Int, Type::Str], return_type: Box::new(Type::Bool) };
    let mir_type = func.to_mir_type();

    match mir_type {
        MIRType::Function { params, return_type } => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], MIRType::Int);
            assert_eq!(params[1], MIRType::Str);
            assert_eq!(*return_type, MIRType::Bool);
        }
        _ => panic!("Expected Function type, got {mir_type:?}"),
    }
}

#[test]
fn test_class_type_mapping() {
    let class = Type::Class { name: "MyClass".to_string(), type_params: vec![] };
    let mir_type = class.to_mir_type();

    assert_eq!(mir_type, MIRType::Object { type_id: None });
}

#[test]
fn test_any_type_mapping() {
    assert_eq!(Type::Any.to_mir_type(), MIRType::Object { type_id: None });
}

#[test]
fn test_union_type_mapping() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    let mir_type = union.to_mir_type();

    assert_eq!(mir_type, MIRType::Object { type_id: None });
}

#[test]
fn test_optional_type_mapping() {
    let optional = Type::Optional(Box::new(Type::Int));
    let mir_type = optional.to_mir_type();

    assert_eq!(mir_type, MIRType::Object { type_id: None });
}

#[test]
fn test_set_type_mapping() {
    let set = Type::Set(Box::new(Type::Int));
    let mir_type = set.to_mir_type();

    assert_eq!(mir_type, MIRType::Object { type_id: None });
}

#[test]
fn test_bytes_type_mapping() {
    assert_eq!(Type::Bytes.to_mir_type(), MIRType::Object { type_id: None });
}

#[test]
fn test_never_type_mapping() {
    assert_eq!(Type::Never.to_mir_type(), MIRType::Object { type_id: None });
}

#[test]
fn test_typevar_type_mapping() {
    let typevar = Type::TypeVar("T".to_string());
    let mir_type = typevar.to_mir_type();

    assert_eq!(mir_type, MIRType::Object { type_id: None });
}

#[test]
fn test_nested_type_mapping() {
    // List[Dict[Str, Int]]
    let nested = Type::List(Box::new(Type::Dict(Box::new(Type::Str), Box::new(Type::Int))));
    let mir_type = nested.to_mir_type();

    match mir_type {
        MIRType::List(elem) => match *elem {
            MIRType::Dict { key, value } => {
                assert_eq!(*key, MIRType::Str);
                assert_eq!(*value, MIRType::Int);
            }
            _ => panic!("Expected Dict inside List, got {elem:?}"),
        },
        _ => panic!("Expected List type, got {mir_type:?}"),
    }
}

#[test]
fn test_complex_nested_types() {
    // Tuple[List[Int], Dict[Str, Bool], Function[[Float], Str]]
    let complex = Type::Tuple(vec![
        Type::List(Box::new(Type::Int)),
        Type::Dict(Box::new(Type::Str), Box::new(Type::Bool)),
        Type::Function { params: vec![Type::Float], return_type: Box::new(Type::Str) },
    ]);
    let mir_type = complex.to_mir_type();

    match mir_type {
        MIRType::Tuple(elems) => {
            assert_eq!(elems.len(), 3);

            // First element: List[Int]
            match &elems[0] {
                MIRType::List(elem) => assert_eq!(**elem, MIRType::Int),
                _ => panic!("Expected List in first tuple element"),
            }

            // Second element: Dict[Str, Bool]
            match &elems[1] {
                MIRType::Dict { key, value } => {
                    assert_eq!(**key, MIRType::Str);
                    assert_eq!(**value, MIRType::Bool);
                }
                _ => panic!("Expected Dict in second tuple element"),
            }

            // Third element: Function[[Float], Str]
            match &elems[2] {
                MIRType::Function { params, return_type } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0], MIRType::Float);
                    assert_eq!(**return_type, MIRType::Str);
                }
                _ => panic!("Expected Function in third tuple element"),
            }
        }
        _ => panic!("Expected Tuple type, got {mir_type:?}"),
    }
}

// Integration tests with LoweringContext would require setting up:
// - An AST with actual nodes
// - An AnalysisContext with populated type environment
// - Symbol table with type information
// These are better suited for end-to-end integration tests
// For now, we focus on the type mapping layer which is pure and testable

#[test]
fn test_empty_tuple_mapping() {
    let empty_tuple = Type::Tuple(vec![]);
    let mir_type = empty_tuple.to_mir_type();

    match mir_type {
        MIRType::Tuple(elems) => assert_eq!(elems.len(), 0),
        _ => panic!("Expected empty Tuple type"),
    }
}

#[test]
fn test_function_no_params_mapping() {
    let func = Type::Function { params: vec![], return_type: Box::new(Type::None) };
    let mir_type = func.to_mir_type();

    match mir_type {
        MIRType::Function { params, return_type } => {
            assert_eq!(params.len(), 0);
            assert_eq!(*return_type, MIRType::None);
        }
        _ => panic!("Expected Function type"),
    }
}

#[test]
fn test_deeply_nested_lists() {
    // List[List[List[Int]]]
    let deep = Type::List(Box::new(Type::List(Box::new(Type::List(Box::new(Type::Int))))));
    let mir_type = deep.to_mir_type();

    match mir_type {
        MIRType::List(l1) => match *l1 {
            MIRType::List(l2) => match *l2 {
                MIRType::List(l3) => assert_eq!(*l3, MIRType::Int),
                _ => panic!("Expected List[Int] at third level"),
            },
            _ => panic!("Expected List[List[Int]] at second level"),
        },
        _ => panic!("Expected List at top level"),
    }
}
