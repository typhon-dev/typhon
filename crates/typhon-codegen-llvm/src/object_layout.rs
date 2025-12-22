//! Typhon object memory layout definitions.
//!
//! This module declares all Typhon object struct types in LLVM IR.
//! All objects share a common header with reference count and type ID.

use inkwell::AddressSpace;

use crate::context::CodegenContext;
use crate::error::CodegenResult;

/// Declare all Typhon object types in the LLVM module.
///
/// This function creates opaque struct types for all Typhon runtime objects:
///
/// - `TyphonObject` - Base object with header (`ref_count`, `type_id`)
/// - `TyphonInt` - Integer object
/// - `TyphonFloat` - Float object
/// - `TyphonStr` - String object
/// - `TyphonBool` - Boolean object (currently unused, uses i1 directly)
/// - `TyphonNone` - None singleton
/// - `TyphonList` - List object
/// - `TyphonDict` - Dictionary object
/// - `TyphonTuple` - Tuple object
/// - `TyphonFunction` - Function object
/// - `TyphonClosure` - Closure object
///
/// All struct types are cached in the context for later use.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
///
/// ## Returns
///
/// `Ok(())` if all types were declared successfully, or an error.
///
/// ## Errors
///
/// This function currently does not return errors, but the signature allows
/// for future error handling if struct declaration fails.
#[allow(clippy::too_many_lines)]
pub fn declare_object_types(ctx: &mut CodegenContext<'_>) -> CodegenResult<()> {
    let context = ctx.context;
    let i64_type = context.i64_type();
    let i1_type = context.bool_type();
    let double_type = context.f64_type();
    let i8_ptr = context.ptr_type(AddressSpace::default());

    // Base object type (all objects start with this header)
    let typhon_object = context.opaque_struct_type("TyphonObject");

    typhon_object.set_body(
        &[
            i64_type.into(), // ref_count
            i64_type.into(), // type_id
        ],
        false,
    );

    ctx.struct_types.insert("TyphonObject".to_string(), typhon_object);

    // Integer object
    let typhon_int = context.opaque_struct_type("TyphonInt");

    typhon_int.set_body(
        &[
            i64_type.into(), // ref_count
            i64_type.into(), // type_id
            i64_type.into(), // value
        ],
        false,
    );

    ctx.struct_types.insert("TyphonInt".to_string(), typhon_int);

    // Float object
    let typhon_float = context.opaque_struct_type("TyphonFloat");

    typhon_float.set_body(
        &[
            i64_type.into(),    // ref_count
            i64_type.into(),    // type_id
            double_type.into(), // value (f64)
        ],
        false,
    );

    ctx.struct_types.insert("TyphonFloat".to_string(), typhon_float);

    // Boolean object
    let typhon_bool = context.opaque_struct_type("TyphonBool");

    typhon_bool.set_body(
        &[
            i64_type.into(), // ref_count
            i64_type.into(), // type_id
            i1_type.into(),  // value (bool)
        ],
        false,
    );

    ctx.struct_types.insert("TyphonBool".to_string(), typhon_bool);

    // String object
    let typhon_str = context.opaque_struct_type("TyphonStr");

    typhon_str.set_body(
        &[
            i64_type.into(), // ref_count
            i64_type.into(), // type_id
            i8_ptr.into(),   // data (UTF-8 bytes)
            i64_type.into(), // length
        ],
        false,
    );

    ctx.struct_types.insert("TyphonStr".to_string(), typhon_str);

    // None singleton
    let typhon_none = context.opaque_struct_type("TyphonNone");

    typhon_none.set_body(
        &[
            i64_type.into(), // ref_count (always 1, never freed)
            i64_type.into(), // type_id
        ],
        false,
    );

    ctx.struct_types.insert("TyphonNone".to_string(), typhon_none);

    // List object
    let typhon_list = context.opaque_struct_type("TyphonList");
    let obj_ptr_ptr = context.ptr_type(AddressSpace::default());

    typhon_list.set_body(
        &[
            i64_type.into(),    // ref_count
            i64_type.into(),    // type_id
            obj_ptr_ptr.into(), // items (array of object pointers)
            i64_type.into(),    // length
            i64_type.into(),    // capacity
        ],
        false,
    );

    ctx.struct_types.insert("TyphonList".to_string(), typhon_list);

    // Dictionary object
    let typhon_dict = context.opaque_struct_type("TyphonDict");

    typhon_dict.set_body(
        &[
            i64_type.into(), // ref_count
            i64_type.into(), // type_id
            i8_ptr.into(),   // internal data structure (hashmap)
            i64_type.into(), // size
        ],
        false,
    );

    ctx.struct_types.insert("TyphonDict".to_string(), typhon_dict);

    // Tuple object
    let typhon_tuple = context.opaque_struct_type("TyphonTuple");

    typhon_tuple.set_body(
        &[
            i64_type.into(),    // ref_count
            i64_type.into(),    // type_id
            obj_ptr_ptr.into(), // items (array of object pointers)
            i64_type.into(),    // length (fixed at creation)
        ],
        false,
    );

    ctx.struct_types.insert("TyphonTuple".to_string(), typhon_tuple);

    // Function object
    let typhon_function = context.opaque_struct_type("TyphonFunction");

    typhon_function.set_body(
        &[
            i64_type.into(), // ref_count
            i64_type.into(), // type_id
            i8_ptr.into(),   // function pointer (void*)
        ],
        false,
    );

    ctx.struct_types.insert("TyphonFunction".to_string(), typhon_function);

    // Closure object (function + captured environment)
    let typhon_closure = context.opaque_struct_type("TyphonClosure");

    typhon_closure.set_body(
        &[
            i64_type.into(),    // ref_count
            i64_type.into(),    // type_id
            i8_ptr.into(),      // function pointer (void*)
            obj_ptr_ptr.into(), // captured variables (array)
            i64_type.into(),    // num_captures
        ],
        false,
    );

    ctx.struct_types.insert("TyphonClosure".to_string(), typhon_closure);

    Ok(())
}

#[cfg(test)]
mod tests {
    use inkwell::context::Context;

    use super::*;
    use crate::error::CodegenError;

    #[test]
    fn test_declare_object_types() {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();
        let mut ctx = CodegenContext::new(&context, module, builder);
        let result = declare_object_types(&mut ctx);

        assert!(result.is_ok());

        // Verify all struct types were declared
        assert!(ctx.get_struct_type("TyphonObject").is_ok());
        assert!(ctx.get_struct_type("TyphonInt").is_ok());
        assert!(ctx.get_struct_type("TyphonFloat").is_ok());
        assert!(ctx.get_struct_type("TyphonBool").is_ok());
        assert!(ctx.get_struct_type("TyphonStr").is_ok());
        assert!(ctx.get_struct_type("TyphonNone").is_ok());
        assert!(ctx.get_struct_type("TyphonList").is_ok());
        assert!(ctx.get_struct_type("TyphonDict").is_ok());
        assert!(ctx.get_struct_type("TyphonTuple").is_ok());
        assert!(ctx.get_struct_type("TyphonFunction").is_ok());
        assert!(ctx.get_struct_type("TyphonClosure").is_ok());
    }

    #[test]
    fn test_struct_type_not_found() {
        let context = Context::create();
        let module = context.create_module("test");
        let builder = context.create_builder();
        let ctx = CodegenContext::new(&context, module, builder);
        let result = ctx.get_struct_type("NonExistent");

        assert!(result.is_err());
        assert!(matches!(result, Err(CodegenError::StructTypeNotFound(_))));
    }
}
