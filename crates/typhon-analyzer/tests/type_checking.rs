//! Tests for type checking and inference functionality.

use typhon_analyzer::types::Type;

#[test]
fn test_type_is_numeric() {
    // Numeric types
    assert!(Type::Int.is_numeric());
    assert!(Type::Float.is_numeric());

    // Non-numeric types
    assert!(!Type::Str.is_numeric());
    assert!(!Type::Bool.is_numeric());
    assert!(!Type::None.is_numeric());
    assert!(!Type::Any.is_numeric());
}

#[test]
fn test_type_is_compatible_with_same_types() {
    // Same types are compatible
    assert!(Type::Int.is_compatible_with(&Type::Int));
    assert!(Type::Float.is_compatible_with(&Type::Float));
    assert!(Type::Str.is_compatible_with(&Type::Str));
    assert!(Type::Bool.is_compatible_with(&Type::Bool));
    assert!(Type::None.is_compatible_with(&Type::None));
}

#[test]
fn test_type_is_compatible_with_any() {
    // Any type is compatible with everything
    assert!(Type::Int.is_compatible_with(&Type::Any));
    assert!(Type::Float.is_compatible_with(&Type::Any));
    assert!(Type::Str.is_compatible_with(&Type::Any));
    assert!(Type::Bool.is_compatible_with(&Type::Any));

    // Everything is compatible with Any
    assert!(Type::Any.is_compatible_with(&Type::Int));
    assert!(Type::Any.is_compatible_with(&Type::Float));
    assert!(Type::Any.is_compatible_with(&Type::Str));
    assert!(Type::Any.is_compatible_with(&Type::Bool));
}

#[test]
fn test_type_is_compatible_with_incompatible() {
    // Different concrete types are incompatible
    assert!(!Type::Int.is_compatible_with(&Type::Float));
    assert!(!Type::Int.is_compatible_with(&Type::Str));
    assert!(!Type::Float.is_compatible_with(&Type::Int));
    assert!(!Type::Str.is_compatible_with(&Type::Int));
    assert!(!Type::Bool.is_compatible_with(&Type::Int));
}

#[test]
fn test_type_is_subtype_of_same() {
    // Types are subtypes of themselves
    assert!(Type::Int.is_subtype_of(&Type::Int));
    assert!(Type::Float.is_subtype_of(&Type::Float));
    assert!(Type::Str.is_subtype_of(&Type::Str));
}

#[test]
fn test_type_is_subtype_of_any() {
    // Everything is a subtype of Any
    assert!(Type::Int.is_subtype_of(&Type::Any));
    assert!(Type::Float.is_subtype_of(&Type::Any));
    assert!(Type::Str.is_subtype_of(&Type::Any));
    assert!(Type::Bool.is_subtype_of(&Type::Any));
    assert!(Type::None.is_subtype_of(&Type::Any));
}

#[test]
fn test_type_is_subtype_of_different() {
    // Different concrete types are not subtypes
    assert!(!Type::Int.is_subtype_of(&Type::Float));
    assert!(!Type::Float.is_subtype_of(&Type::Int));
    assert!(!Type::Str.is_subtype_of(&Type::Int));
}

#[test]
fn test_type_unify_same_types() {
    // Unifying same types returns that type
    assert_eq!(Type::Int.unify(&Type::Int), Some(Type::Int));
    assert_eq!(Type::Float.unify(&Type::Float), Some(Type::Float));
    assert_eq!(Type::Str.unify(&Type::Str), Some(Type::Str));
    assert_eq!(Type::Bool.unify(&Type::Bool), Some(Type::Bool));
}

#[test]
fn test_type_unify_with_any() {
    // Unifying with Any returns the more specific type
    assert_eq!(Type::Int.unify(&Type::Any), Some(Type::Int));
    assert_eq!(Type::Any.unify(&Type::Float), Some(Type::Float));
    assert_eq!(Type::Str.unify(&Type::Any), Some(Type::Str));
    assert_eq!(Type::Any.unify(&Type::Bool), Some(Type::Bool));
}

#[test]
fn test_type_unify_incompatible() {
    // Unifying incompatible concrete types creates a union
    // (based on actual implementation behavior)
    assert!(Type::Int.unify(&Type::Float).is_some());
    assert!(Type::Int.unify(&Type::Str).is_some());
    assert!(Type::Float.unify(&Type::Bool).is_some());
}

#[test]
fn test_type_unify_list_types() {
    // Unifying list types with same element type
    let list_int = Type::List(Box::new(Type::Int));
    let list_int2 = Type::List(Box::new(Type::Int));
    // Based on actual implementation, unify creates a union of the two list types
    let result = list_int.unify(&list_int2);
    assert!(result.is_some());

    // Unifying list with Any element
    let list_any = Type::List(Box::new(Type::Any));
    let result = list_int.unify(&list_any);
    assert!(result.is_some());
}

#[test]
fn test_type_substitute_simple() {
    use std::collections::HashMap;

    // No substitution for concrete types
    let mut subst = HashMap::new();
    drop(subst.insert("T".to_string(), Type::Int));

    assert_eq!(Type::Int.substitute(&subst), Type::Int);
    assert_eq!(Type::Str.substitute(&subst), Type::Str);
}

#[test]
fn test_type_substitute_list() {
    use std::collections::HashMap;

    // Substitute element type in List
    let mut subst = HashMap::new();
    drop(subst.insert("T".to_string(), Type::Int));

    let list_any = Type::List(Box::new(Type::Any));
    // Substitution recurses into the list element type
    let result = list_any.substitute(&subst);
    assert!(matches!(result, Type::List(_)));
}

#[test]
fn test_type_get_attribute_stub() {
    // get_attribute returns None for now (stub implementation)
    assert_eq!(Type::Int.get_attribute("value"), None);
    assert_eq!(Type::Str.get_attribute("length"), None);
}

#[test]
fn test_type_get_method_stub() {
    // get_method is a stub that returns None for now
    assert_eq!(Type::Int.get_method("to_string"), None);
    assert_eq!(Type::Str.get_method("upper"), None);
}

#[test]
fn test_list_type_compatibility() {
    let list_int = Type::List(Box::new(Type::Int));
    let list_int2 = Type::List(Box::new(Type::Int));
    let list_str = Type::List(Box::new(Type::Str));

    // Same list types are compatible
    assert!(list_int.is_compatible_with(&list_int2));

    // Different element types are incompatible
    assert!(!list_int.is_compatible_with(&list_str));

    // List is compatible with Any
    assert!(list_int.is_compatible_with(&Type::Any));
}

#[test]
fn test_function_type_compatibility() {
    let func1 =
        Type::Function { params: vec![Type::Int, Type::Str], return_type: Box::new(Type::Bool) };

    let func2 =
        Type::Function { params: vec![Type::Int, Type::Str], return_type: Box::new(Type::Bool) };

    let func3 = Type::Function { params: vec![Type::Int], return_type: Box::new(Type::Bool) };

    // Same function signatures are compatible
    assert!(func1.is_compatible_with(&func2));

    // Different signatures are incompatible
    assert!(!func1.is_compatible_with(&func3));
}

#[test]
fn test_tuple_type_compatibility() {
    let tuple1 = Type::Tuple(vec![Type::Int, Type::Str]);
    let tuple2 = Type::Tuple(vec![Type::Int, Type::Str]);
    let tuple3 = Type::Tuple(vec![Type::Int, Type::Bool]);

    // Same tuple types are compatible
    assert!(tuple1.is_compatible_with(&tuple2));

    // Different tuple types are incompatible
    assert!(!tuple1.is_compatible_with(&tuple3));
}

#[test]
fn test_dict_type_compatibility() {
    let dict1 = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));

    let dict2 = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));

    let dict3 = Type::Dict(Box::new(Type::Str), Box::new(Type::Bool));

    // Same dict types are compatible
    assert!(dict1.is_compatible_with(&dict2));

    // Different value types are incompatible
    assert!(!dict1.is_compatible_with(&dict3));
}

#[test]
fn test_union_type_is_numeric() {
    // Union types are not considered numeric in the current implementation
    // because is_numeric() only checks for Int and Float directly
    let union = Type::Union(vec![Type::Int, Type::Float]);
    assert!(!union.is_numeric()); // Union itself is not numeric

    // Union with non-numeric is also not numeric
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert!(!union.is_numeric());
}
