//! Type translation tests for LLVM code generation.
//!
//! These tests verify that MIR types are correctly translated to their LLVM type
//! representations.

use inkwell::context::Context;
use typhon_codegen_llvm::context::CodegenContext;
use typhon_codegen_llvm::object_layout::declare_object_types;
use typhon_codegen_llvm::types::{translate_and_cache_type, translate_type};
use typhon_mir::types::MIRType;

/// Helper to create a test context with object types declared.
///
/// Returns a leaked context and a code generation context.
/// The context is leaked to avoid lifetime issues in tests.
fn create_test_context() -> &'static CodegenContext<'static> {
    // SAFETY: We're leaking the context to get a 'static lifetime for testing.
    // This is acceptable in tests as they run in isolated processes.
    let context = Box::leak(Box::new(Context::create()));
    let module = context.create_module("test");
    let builder = context.create_builder();
    let mut ctx = Box::new(CodegenContext::new(context, module, builder));

    declare_object_types(&mut ctx).expect("Failed to declare object types");

    Box::leak(ctx)
}

#[test]
fn test_translate_bool_type() {
    let ctx = create_test_context();
    let bool_type = translate_type(ctx, &MIRType::Bool).expect("Failed to translate Bool type");

    assert!(bool_type.is_int_type(), "Bool should translate to int type");

    let int_type = bool_type.into_int_type();

    assert_eq!(int_type.get_bit_width(), 1, "Bool should be i1 (1-bit integer)");
}

#[test]
fn test_translate_int_type() {
    let ctx = create_test_context();
    let int_type = translate_type(ctx, &MIRType::Int).expect("Failed to translate Int type");

    assert!(int_type.is_pointer_type(), "Int should translate to pointer type");
}

#[test]
fn test_translate_float_type() {
    let ctx = create_test_context();
    let float_type = translate_type(ctx, &MIRType::Float).expect("Failed to translate Float type");

    assert!(float_type.is_pointer_type(), "Float should translate to pointer type");
}

#[test]
fn test_translate_str_type() {
    let ctx = create_test_context();
    let str_type = translate_type(ctx, &MIRType::Str).expect("Failed to translate Str type");

    assert!(str_type.is_pointer_type(), "Str should translate to pointer type");
}

#[test]
fn test_translate_none_type() {
    let ctx = create_test_context();
    let none_type = translate_type(ctx, &MIRType::None).expect("Failed to translate None type");

    assert!(none_type.is_pointer_type(), "None should translate to pointer type");
}

#[test]
fn test_translate_list_type() {
    let ctx = create_test_context();
    let list_type = translate_type(ctx, &MIRType::List(Box::new(MIRType::Int)))
        .expect("Failed to translate List type");

    assert!(list_type.is_pointer_type(), "List should translate to pointer type");
}

#[test]
fn test_translate_dict_type() {
    let ctx = create_test_context();
    let dict_type = translate_type(
        ctx,
        &MIRType::Dict { key: Box::new(MIRType::Str), value: Box::new(MIRType::Int) },
    )
    .expect("Failed to translate Dict type");

    assert!(dict_type.is_pointer_type(), "Dict should translate to pointer type");
}

#[test]
fn test_translate_tuple_type() {
    let ctx = create_test_context();
    let tuple_type = translate_type(ctx, &MIRType::Tuple(vec![MIRType::Int, MIRType::Str]))
        .expect("Failed to translate Tuple type");

    assert!(tuple_type.is_pointer_type(), "Tuple should translate to pointer type");
}

#[test]
fn test_translate_function_type() {
    let ctx = create_test_context();
    let function_type = translate_type(
        ctx,
        &MIRType::Function {
            params: vec![MIRType::Int, MIRType::Int],
            return_type: Box::new(MIRType::Int),
        },
    )
    .expect("Failed to translate Function type");

    assert!(function_type.is_pointer_type(), "Function should translate to pointer type");
}

#[test]
fn test_translate_closure_type() {
    let ctx = create_test_context();
    let closure_type = translate_type(
        ctx,
        &MIRType::Closure {
            params: vec![MIRType::Int],
            return_type: Box::new(MIRType::Int),
            captured: vec![MIRType::Str],
        },
    )
    .expect("Failed to translate Closure type");

    assert!(closure_type.is_pointer_type(), "Closure should translate to pointer type");
}

#[test]
fn test_translate_ref_type() {
    let ctx = create_test_context();
    let ref_type = translate_type(ctx, &MIRType::Ref(Box::new(MIRType::Int)))
        .expect("Failed to translate Ref type");

    assert!(ref_type.is_pointer_type(), "Ref should translate to pointer type");
}

#[test]
fn test_translate_object_type() {
    let ctx = create_test_context();
    let object_type = translate_type(ctx, &MIRType::Object { type_id: Some(42) })
        .expect("Failed to translate Object type");

    assert!(object_type.is_pointer_type(), "Object should translate to pointer type");
}

#[test]
fn test_translate_void_type_error() {
    let ctx = create_test_context();
    let result = translate_type(ctx, &MIRType::Void);

    assert!(result.is_err(), "Void type should produce an error");

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Void"),
                "Error message should mention Void, got: {error_msg}"
            );
        }
        Ok(_) => panic!("Expected error for Void type"),
    }
}

#[test]
fn test_type_caching() {
    // Create a non-leaked context for mutability testing
    let context = Context::create();
    let module = context.create_module("test");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(&context, module, builder);

    declare_object_types(&mut ctx).expect("Failed to declare object types");

    let mir_type = MIRType::Int;
    let first_translation =
        translate_and_cache_type(&mut ctx, &mir_type).expect("First translation failed");
    let second_translation = translate_type(&ctx, &mir_type).expect("Second translation failed");

    assert_eq!(
        first_translation, second_translation,
        "Cached type should be identical to original"
    );

    // Verify caching by translating again - should hit cache
    let third_translation = translate_type(&ctx, &mir_type).expect("Third translation failed");

    assert_eq!(first_translation, third_translation, "Third translation should also match");
}

#[test]
fn test_nested_ref_type() {
    let ctx = create_test_context();
    let nested_ref_type =
        translate_type(ctx, &MIRType::Ref(Box::new(MIRType::Ref(Box::new(MIRType::Int)))))
            .expect("Failed to translate nested Ref type");

    assert!(nested_ref_type.is_pointer_type(), "Nested Ref should translate to pointer type");
}

#[test]
fn test_complex_list_type() {
    let ctx = create_test_context();
    let complex_list_type = translate_type(
        ctx,
        &MIRType::List(Box::new(MIRType::Tuple(vec![MIRType::Int, MIRType::Str]))),
    )
    .expect("Failed to translate complex List type");

    assert!(complex_list_type.is_pointer_type(), "Complex List should translate to pointer type");
}

#[test]
fn test_complex_dict_type() {
    let ctx = create_test_context();
    let complex_dict_type = translate_type(
        ctx,
        &MIRType::Dict {
            key: Box::new(MIRType::Tuple(vec![MIRType::Int, MIRType::Str])),
            value: Box::new(MIRType::List(Box::new(MIRType::Float))),
        },
    )
    .expect("Failed to translate complex Dict type");

    assert!(complex_dict_type.is_pointer_type(), "Complex Dict should translate to pointer type");
}
