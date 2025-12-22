//! Smoke tests for LLVM code generation infrastructure.
//!
//! These tests verify basic functionality of the LLVM codegen crate.

use inkwell::context::Context;
use typhon_codegen_llvm::context::CodegenContext;
use typhon_codegen_llvm::object_layout::declare_object_types;

#[test]
fn test_create_context_and_module() {
    // Test that we can create an LLVM context and module
    let context = Context::create();
    let module = context.create_module("test_module");
    let builder = context.create_builder();
    let _ctx = CodegenContext::new(&context, module, builder);

    // If we get here without panicking, test passes
}

#[test]
fn test_declare_all_object_types() {
    // Test that we can declare all Typhon object struct types
    let context = Context::create();
    let module = context.create_module("test_module");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(&context, module, builder);

    // This should declare all 11 struct types
    let result = declare_object_types(&mut ctx);

    assert!(result.is_ok(), "Failed to declare object types: {result:?}");

    // Verify all expected types are present
    let expected_types = vec![
        "TyphonObject",
        "TyphonInt",
        "TyphonFloat",
        "TyphonBool",
        "TyphonStr",
        "TyphonNone",
        "TyphonList",
        "TyphonDict",
        "TyphonTuple",
        "TyphonFunction",
        "TyphonClosure",
    ];

    for type_name in expected_types {
        assert!(ctx.get_struct_type(type_name).is_ok(), "Missing struct type: {type_name}");
    }
}

#[test]
fn test_struct_types_have_correct_structure() {
    // Test that struct types have the expected field counts
    let context = Context::create();
    let module = context.create_module("test_module");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(&context, module, builder);

    declare_object_types(&mut ctx).unwrap();

    // TyphonObject has 2 fields (ref_count, type_id)
    let typhon_object = ctx.get_struct_type("TyphonObject").unwrap();

    assert_eq!(typhon_object.count_fields(), 2, "TyphonObject should have 2 fields");

    // TyphonInt has 3 fields (ref_count, type_id, value)
    let typhon_int = ctx.get_struct_type("TyphonInt").unwrap();

    assert_eq!(typhon_int.count_fields(), 3, "TyphonInt should have 3 fields");

    // TyphonStr has 4 fields (ref_count, type_id, data, length)
    let typhon_str = ctx.get_struct_type("TyphonStr").unwrap();

    assert_eq!(typhon_str.count_fields(), 4, "TyphonStr should have 4 fields");

    // TyphonList has 5 fields (ref_count, type_id, items, length, capacity)
    let typhon_list = ctx.get_struct_type("TyphonList").unwrap();

    assert_eq!(typhon_list.count_fields(), 5, "TyphonList should have 5 fields");

    // TyphonClosure has 5 fields (ref_count, type_id, fn_ptr, captures, num_captures)
    let typhon_closure = ctx.get_struct_type("TyphonClosure").unwrap();

    assert_eq!(typhon_closure.count_fields(), 5, "TyphonClosure should have 5 fields");
}
