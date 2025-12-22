//! Runtime integration tests for LLVM code generation.
//!
//! Tests runtime function calls for object operations, type operations,
//! and collection building.

use inkwell::context::Context;
use inkwell::values::BasicValue;
use typhon_codegen_llvm::context::CodegenContext;
use typhon_codegen_llvm::instructions::translate_instruction;
use typhon_codegen_llvm::runtime::declare_runtime_functions;
use typhon_mir::instr::{MIRConst, MIRInstr, ValueID};
use typhon_mir::types::{MIRType, TypeID};

/// Helper to create a test context with runtime functions declared.
fn setup_test_context(context: &Context) -> CodegenContext<'_> {
    let module = context.create_module("test");
    let builder = context.create_builder();
    let mut ctx = CodegenContext::new(context, module, builder);

    declare_runtime_functions(&mut ctx).expect("Failed to declare runtime functions");

    // Create a dummy function and basic block for the builder to work in
    let fn_type = context.void_type().fn_type(&[], false);
    let function = ctx.module.add_function("test_fn", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    ctx.builder.position_at_end(basic_block);
    ctx.current_function = Some(function);

    ctx
}

/// Helper to create a mock object value.
fn create_mock_object(ctx: &mut CodegenContext<'_>, value_id: ValueID) {
    let ptr_type = ctx.context.ptr_type(inkwell::AddressSpace::default());
    let null_ptr = ptr_type.const_null();
    ctx.set_value(value_id, null_ptr.as_basic_value_enum());
}

// ===== Attribute Operations Tests =====

#[test]
fn test_getattr_generates_runtime_call() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create mock object value
    let obj_id = ValueID(1);
    create_mock_object(&mut ctx, obj_id);

    // Create GetAttr instruction
    let instr = MIRInstr::GetAttr {
        object: obj_id,
        attr: "test_attr".to_string(),
        ty: MIRType::Object { type_id: None },
    };

    // Translate instruction
    let result = translate_instruction(&mut ctx, &instr);
    assert!(
        result.is_ok(),
        "GetAttr instruction should translate successfully: {:?}",
        result.as_ref().err()
    );
    assert!(result.unwrap().is_some(), "GetAttr should return a value");

    // Verify LLVM IR contains call to typhon_getattr
    let module_str = ctx.module.print_to_string().to_string();
    assert!(
        module_str.contains("typhon_getattr"),
        "Generated IR should contain call to typhon_getattr"
    );
}

#[test]
fn test_setattr_generates_runtime_call() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create mock object and value
    let obj_id = ValueID(1);
    let val_id = ValueID(2);
    create_mock_object(&mut ctx, obj_id);
    create_mock_object(&mut ctx, val_id);

    // Create SetAttr instruction
    let instr = MIRInstr::SetAttr { object: obj_id, attr: "test_attr".to_string(), value: val_id };

    // Translate instruction
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_ok(), "SetAttr instruction should translate successfully");
    assert!(result.unwrap().is_none(), "SetAttr should not return a value");

    // Verify LLVM IR contains call to typhon_setattr
    let module_str = ctx.module.print_to_string().to_string();
    assert!(
        module_str.contains("typhon_setattr"),
        "Generated IR should contain call to typhon_setattr"
    );
}

#[test]
fn test_getattr_with_different_attribute_names() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    let obj_id = ValueID(1);
    create_mock_object(&mut ctx, obj_id);

    // Test multiple attribute names
    for attr_name in &["foo", "bar", "baz"] {
        let instr = MIRInstr::GetAttr {
            object: obj_id,
            attr: (*attr_name).to_string(),
            ty: MIRType::Object { type_id: None },
        };

        let result = translate_instruction(&mut ctx, &instr);
        assert!(
            result.is_ok(),
            "GetAttr with attribute '{attr_name}' should translate successfully"
        );
    }
}

// ===== Item Operations Tests =====

#[test]
fn test_getitem_generates_runtime_call() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create mock object and key
    let obj_id = ValueID(1);
    let key_id = ValueID(2);
    create_mock_object(&mut ctx, obj_id);
    create_mock_object(&mut ctx, key_id);

    // Create GetItem instruction
    let instr =
        MIRInstr::GetItem { object: obj_id, key: key_id, ty: MIRType::Object { type_id: None } };

    // Translate instruction
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_ok(), "GetItem instruction should translate successfully");
    assert!(result.unwrap().is_some(), "GetItem should return a value");

    // Verify LLVM IR contains call to typhon_getitem
    let module_str = ctx.module.print_to_string().to_string();
    assert!(
        module_str.contains("typhon_getitem"),
        "Generated IR should contain call to typhon_getitem"
    );
}

#[test]
fn test_setitem_generates_runtime_call() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create mock object, key, and value
    let obj_id = ValueID(1);
    let key_id = ValueID(2);
    let val_id = ValueID(3);
    create_mock_object(&mut ctx, obj_id);
    create_mock_object(&mut ctx, key_id);
    create_mock_object(&mut ctx, val_id);

    // Create SetItem instruction
    let instr = MIRInstr::SetItem { object: obj_id, key: key_id, value: val_id };

    // Translate instruction
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_ok(), "SetItem instruction should translate successfully");
    assert!(result.unwrap().is_none(), "SetItem should not return a value");

    // Verify LLVM IR contains call to typhon_setitem
    let module_str = ctx.module.print_to_string().to_string();
    assert!(
        module_str.contains("typhon_setitem"),
        "Generated IR should contain call to typhon_setitem"
    );
}

#[test]
fn test_getitem_with_integer_key() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    let obj_id = ValueID(1);
    create_mock_object(&mut ctx, obj_id);

    // Create integer constant as key
    let key_instr = MIRInstr::Const(MIRConst::Int(42));
    let key_val = translate_instruction(&mut ctx, &key_instr).unwrap().unwrap();
    let key_id = ValueID(2);
    ctx.set_value(key_id, key_val);

    // Create GetItem instruction
    let instr =
        MIRInstr::GetItem { object: obj_id, key: key_id, ty: MIRType::Object { type_id: None } };

    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_ok(), "GetItem with integer key should translate successfully");
}

// ===== Type Operations Tests =====

#[test]
fn test_instanceof_generates_runtime_call() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create mock object
    let obj_id = ValueID(1);
    create_mock_object(&mut ctx, obj_id);

    // Create InstanceOf instruction
    let type_id: TypeID = 123;
    let instr = MIRInstr::InstanceOf { object: obj_id, type_id };

    // Translate instruction
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_ok(), "InstanceOf instruction should translate successfully");
    assert!(result.unwrap().is_some(), "InstanceOf should return a value");

    // Verify LLVM IR contains call to typhon_isinstance
    let module_str = ctx.module.print_to_string().to_string();
    assert!(
        module_str.contains("typhon_isinstance"),
        "Generated IR should contain call to typhon_isinstance"
    );
}

#[test]
fn test_cast_generates_runtime_call() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create mock object
    let obj_id = ValueID(1);
    create_mock_object(&mut ctx, obj_id);

    // Create Cast instruction
    let instr = MIRInstr::Cast { value: obj_id, target_ty: MIRType::Int };

    // Translate instruction
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_ok(), "Cast instruction should translate successfully");
    assert!(result.unwrap().is_some(), "Cast should return a value");

    // Verify LLVM IR contains call to typhon_cast
    let module_str = ctx.module.print_to_string().to_string();
    assert!(module_str.contains("typhon_cast"), "Generated IR should contain call to typhon_cast");
}

#[test]
fn test_instanceof_with_different_type_ids() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    let obj_id = ValueID(1);
    create_mock_object(&mut ctx, obj_id);

    // Test multiple type IDs
    for type_id in [1, 42, 100, 999] {
        let instr = MIRInstr::InstanceOf { object: obj_id, type_id };

        let result = translate_instruction(&mut ctx, &instr);
        assert!(result.is_ok(), "InstanceOf with type_id {type_id} should translate successfully");
    }
}

// ===== Runtime Function Declaration Tests =====

#[test]
fn test_all_attribute_functions_declared() {
    let context = Context::create();
    let ctx = setup_test_context(&context);

    // Verify attribute operation functions are declared
    assert!(
        ctx.get_runtime_function("typhon_getattr").is_ok(),
        "typhon_getattr should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_setattr").is_ok(),
        "typhon_setattr should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_hasattr").is_ok(),
        "typhon_hasattr should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_delattr").is_ok(),
        "typhon_delattr should be declared"
    );
}

#[test]
fn test_all_item_functions_declared() {
    let context = Context::create();
    let ctx = setup_test_context(&context);

    // Verify item operation functions are declared
    assert!(
        ctx.get_runtime_function("typhon_getitem").is_ok(),
        "typhon_getitem should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_setitem").is_ok(),
        "typhon_setitem should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_delitem").is_ok(),
        "typhon_delitem should be declared"
    );
}

#[test]
fn test_all_type_functions_declared() {
    let context = Context::create();
    let ctx = setup_test_context(&context);

    // Verify type operation functions are declared
    assert!(
        ctx.get_runtime_function("typhon_isinstance").is_ok(),
        "typhon_isinstance should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_type_of").is_ok(),
        "typhon_type_of should be declared"
    );
    assert!(ctx.get_runtime_function("typhon_cast").is_ok(), "typhon_cast should be declared");
}

#[test]
fn test_all_collection_functions_declared() {
    let context = Context::create();
    let ctx = setup_test_context(&context);

    // Verify collection operation functions are declared
    assert!(
        ctx.get_runtime_function("typhon_list_new").is_ok(),
        "typhon_list_new should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_list_append").is_ok(),
        "typhon_list_append should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_dict_new").is_ok(),
        "typhon_dict_new should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_dict_set").is_ok(),
        "typhon_dict_set should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_tuple_new").is_ok(),
        "typhon_tuple_new should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_tuple_set").is_ok(),
        "typhon_tuple_set should be declared"
    );
}

#[test]
fn test_all_string_functions_declared() {
    let context = Context::create();
    let ctx = setup_test_context(&context);

    // Verify string operation functions are declared
    assert!(
        ctx.get_runtime_function("typhon_str_concat").is_ok(),
        "typhon_str_concat should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_str_format").is_ok(),
        "typhon_str_format should be declared"
    );
}

#[test]
fn test_all_iteration_functions_declared() {
    let context = Context::create();
    let ctx = setup_test_context(&context);

    // Verify iteration operation functions are declared
    assert!(ctx.get_runtime_function("typhon_iter").is_ok(), "typhon_iter should be declared");
    assert!(
        ctx.get_runtime_function("typhon_iter_next").is_ok(),
        "typhon_iter_next should be declared"
    );
    assert!(
        ctx.get_runtime_function("typhon_iter_has_next").is_ok(),
        "typhon_iter_has_next should be declared"
    );
}

// ===== Error Cases Tests =====

#[test]
fn test_getattr_with_missing_object() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create GetAttr instruction with non-existent object
    let obj_id = ValueID(999); // Not created
    let instr = MIRInstr::GetAttr {
        object: obj_id,
        attr: "test_attr".to_string(),
        ty: MIRType::Object { type_id: None },
    };

    // Should fail because object value doesn't exist
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_err(), "GetAttr with missing object should fail");
}

#[test]
fn test_setitem_with_missing_value() {
    let context = Context::create();
    let mut ctx = setup_test_context(&context);

    // Create object and key but not value
    let obj_id = ValueID(1);
    let key_id = ValueID(2);
    let val_id = ValueID(999); // Not created
    create_mock_object(&mut ctx, obj_id);
    create_mock_object(&mut ctx, key_id);

    let instr = MIRInstr::SetItem { object: obj_id, key: key_id, value: val_id };

    // Should fail because value doesn't exist
    let result = translate_instruction(&mut ctx, &instr);
    assert!(result.is_err(), "SetItem with missing value should fail");
}
