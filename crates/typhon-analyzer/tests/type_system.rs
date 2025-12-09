//! Tests for type system functionality.

use typhon_analyzer::context::SemanticContext;
use typhon_analyzer::types::{Type, TypeEnvironment, TypeID};
use typhon_ast::nodes::NodeID;

#[test]
fn test_type_environment_creation() {
    let env = TypeEnvironment::new();
    // Environment starts empty
    assert!(env.get_type(TypeID::new(0)).is_none());
}

#[test]
fn test_add_and_get_type() {
    let mut env = TypeEnvironment::new();

    let type_id = env.add_type(Type::Int);
    assert_eq!(type_id, TypeID::new(0));

    let retrieved = env.get_type(type_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &Type::Int);
}

#[test]
fn test_multiple_types() {
    let mut env = TypeEnvironment::new();

    let int_id = env.add_type(Type::Int);
    let str_id = env.add_type(Type::Str);
    let bool_id = env.add_type(Type::Bool);

    assert_eq!(env.get_type(int_id), Some(&Type::Int));
    assert_eq!(env.get_type(str_id), Some(&Type::Str));
    assert_eq!(env.get_type(bool_id), Some(&Type::Bool));
}

#[test]
fn test_node_type_association() {
    let mut env = TypeEnvironment::new();
    let type_id = env.add_type(Type::Int);
    let node_id = NodeID::new(42, 0);

    env.set_node_type(node_id, type_id);

    let retrieved = env.get_node_type(node_id);
    assert_eq!(retrieved, Some(type_id));
}

#[test]
fn test_is_numeric() {
    assert!(Type::Int.is_numeric());
    assert!(Type::Float.is_numeric());
    assert!(!Type::Str.is_numeric());
    assert!(!Type::Bool.is_numeric());
}

#[test]
fn test_subtyping_reflexivity() {
    assert!(Type::Int.is_subtype_of(&Type::Int));
    assert!(Type::Str.is_subtype_of(&Type::Str));
    assert!(Type::Bool.is_subtype_of(&Type::Bool));
}

#[test]
fn test_subtyping_none_optional() {
    let opt_int = Type::Optional(Box::new(Type::Int));
    assert!(Type::None.is_subtype_of(&opt_int));
}

#[test]
fn test_subtyping_value_optional() {
    let opt_int = Type::Optional(Box::new(Type::Int));
    assert!(Type::Int.is_subtype_of(&opt_int));
}

#[test]
fn test_subtyping_union() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert!(Type::Int.is_subtype_of(&union));
    assert!(Type::Str.is_subtype_of(&union));
    assert!(!Type::Bool.is_subtype_of(&union));
}

#[test]
fn test_subtyping_any() {
    // Everything is a subtype of Any
    assert!(Type::Int.is_subtype_of(&Type::Any));
    assert!(Type::Str.is_subtype_of(&Type::Any));
    assert!(Type::None.is_subtype_of(&Type::Any));
}

#[test]
fn test_subtyping_never() {
    // Never is a subtype of everything
    assert!(Type::Never.is_subtype_of(&Type::Int));
    assert!(Type::Never.is_subtype_of(&Type::Str));
    assert!(Type::Never.is_subtype_of(&Type::Any));
}

#[test]
fn test_type_compatibility() {
    // Same types are compatible
    assert!(Type::Int.is_compatible_with(&Type::Int));

    // Different types are not compatible
    assert!(!Type::Int.is_compatible_with(&Type::Str));

    // Any is compatible with everything
    assert!(Type::Int.is_compatible_with(&Type::Any));
    assert!(Type::Any.is_compatible_with(&Type::Str));
}

#[test]
fn test_collection_types() {
    let list_int = Type::List(Box::new(Type::Int));
    let set_str = Type::Set(Box::new(Type::Str));
    let dict_str_int = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));

    assert_eq!(list_int, Type::List(Box::new(Type::Int)));
    assert_eq!(set_str, Type::Set(Box::new(Type::Str)));
    assert_eq!(dict_str_int, Type::Dict(Box::new(Type::Str), Box::new(Type::Int)));
}

#[test]
fn test_tuple_type() {
    let tuple = Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]);
    assert_eq!(tuple, Type::Tuple(vec![Type::Int, Type::Str, Type::Bool]));
}

#[test]
fn test_function_type() {
    let func =
        Type::Function { params: vec![Type::Int, Type::Str], return_type: Box::new(Type::Bool) };

    match func {
        Type::Function { params, return_type } => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], Type::Int);
            assert_eq!(params[1], Type::Str);
            assert_eq!(*return_type, Type::Bool);
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_class_type() {
    let mut env = TypeEnvironment::new();
    let type_param_id = env.add_type(Type::Int);

    let class = Type::Class { name: "MyClass".to_string(), type_params: vec![type_param_id] };

    match class {
        Type::Class { name, type_params } => {
            assert_eq!(name, "MyClass");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0], type_param_id);
        }
        _ => panic!("Expected class type"),
    }
}

#[test]
fn test_type_var() {
    let type_var = Type::TypeVar("T".to_string());
    match type_var {
        Type::TypeVar(name) => assert_eq!(name, "T"),
        _ => panic!("Expected type variable"),
    }
}

#[test]
fn test_type_display() {
    assert_eq!(format!("{}", Type::Int), "int");
    assert_eq!(format!("{}", Type::Str), "str");
    assert_eq!(format!("{}", Type::Bool), "bool");
    assert_eq!(format!("{}", Type::Float), "float");
    assert_eq!(format!("{}", Type::None), "None");
    assert_eq!(format!("{}", Type::Any), "Any");
    assert_eq!(format!("{}", Type::Never), "Never");

    let list_int = Type::List(Box::new(Type::Int));
    assert_eq!(format!("{list_int}"), "list[int]");

    let opt_str = Type::Optional(Box::new(Type::Str));
    assert_eq!(format!("{opt_str}"), "str | None");

    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(format!("{union}"), "int | str");
}

#[test]
fn test_semantic_context_type_integration() {
    let mut context = SemanticContext::new();

    // Add a type through context
    let type_id = context.type_environment_mut().add_type(Type::Int);

    // Retrieve it
    let retrieved = context.type_environment().get_type(type_id);
    assert_eq!(retrieved, Some(&Type::Int));

    // Associate with node
    let node_id = NodeID::new(1, 0);
    context.type_environment_mut().set_node_type(node_id, type_id);

    let node_type = context.type_environment().get_node_type(node_id);
    assert_eq!(node_type, Some(type_id));
}
