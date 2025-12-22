//! Runtime function declarations for LLVM code generation.
//!
//! This module declares the signatures of Typhon runtime functions that are
//! called from generated LLVM IR code. These functions handle operations like
//! object creation, memory management, and arithmetic operations that require
//! runtime type information.

use inkwell::AddressSpace;
use inkwell::types::BasicMetadataTypeEnum;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// Declare a single runtime function with the given signature.
///
/// This helper function creates an LLVM function declaration with C ABI
/// and stores it in the context's runtime function cache.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `name` - Function name (e.g., "`typhon_add`")
/// - `param_types` - Array of parameter types
/// - `return_type` - Return type
///
/// ## Errors
///
/// Returns an error if function declaration fails.
fn declare_fn<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    name: &str,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
    return_type: BasicMetadataTypeEnum<'ctx>,
) -> CodegenResult<()> {
    let fn_type = match return_type {
        BasicMetadataTypeEnum::IntType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::FloatType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::PointerType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::ArrayType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::StructType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::VectorType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::ScalableVectorType(t) => t.fn_type(param_types, false),
        BasicMetadataTypeEnum::MetadataType(_) => {
            return Err(CodegenError::InstructionTranslationError(
                "Metadata type cannot be used as function return type".to_string(),
            ));
        }
    };

    let fn_value = ctx.module.add_function(name, fn_type, None);
    ctx.runtime_functions.insert(name.to_string(), fn_value);

    Ok(())
}

/// Declare a runtime function with void return type that never returns (noreturn).
///
/// This is used for functions like `typhon_raise` that always throw exceptions
/// and never return to the caller.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `name` - Function name (e.g., "`typhon_raise`")
/// - `param_types` - Array of parameter types
fn declare_noreturn_fn<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    name: &str,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) {
    let void_type = ctx.context.void_type();
    let fn_type = void_type.fn_type(param_types, false);
    let fn_value = ctx.module.add_function(name, fn_type, None);

    ctx.runtime_functions.insert(name.to_string(), fn_value);
}

/// Declare all Typhon runtime functions in the LLVM module.
///
/// This function declares the C ABI signatures for runtime functions that
/// implement Typhon's object model and operations. The runtime functions
/// handle:
///
/// - Object creation (int, float, string, none)
/// - Memory management (reference counting)
/// - Arithmetic operations (add, sub, mul, div, mod, pow)
/// - Comparison operations (eq, ne, lt, le, gt, ge)
///
/// ## Arguments
///
/// - `ctx` - Code generation context
///
/// ## Errors
///
/// Returns an error if function declaration fails.
///
/// ## Example
///
/// ```rust,ignore
/// let mut ctx = CodegenContext::new(&context, module, builder);
/// declare_runtime_functions(&mut ctx)?;
/// let add_fn = ctx.get_runtime_function("typhon_add")?;
/// ```
pub fn declare_runtime_functions(ctx: &mut CodegenContext<'_>) -> CodegenResult<()> {
    let _i8_type = ctx.context.i8_type();
    let i64_type = ctx.context.i64_type();
    let f64_type = ctx.context.f64_type();
    let i1_type = ctx.context.bool_type();
    let ptr_type = ctx.context.ptr_type(AddressSpace::default());

    // Object creation functions
    declare_fn(ctx, "typhon_int_new", &[i64_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_float_new", &[f64_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_str_new", &[ptr_type.into(), i64_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_none_get", &[], ptr_type.into())?;

    // Memory management functions (void return type)
    declare_void_fn(ctx, "typhon_incref", &[ptr_type.into()]);
    declare_void_fn(ctx, "typhon_decref", &[ptr_type.into()]);

    // Arithmetic operations
    declare_fn(ctx, "typhon_add", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_sub", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_mul", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_div", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_mod", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_pow", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;

    // Comparison operations
    declare_fn(ctx, "typhon_eq", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_fn(ctx, "typhon_ge", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_fn(ctx, "typhon_gt", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_fn(ctx, "typhon_le", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_fn(ctx, "typhon_lt", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_fn(ctx, "typhon_ne", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;

    // Type conversion operations
    declare_fn(ctx, "typhon_is_truthy", &[ptr_type.into()], i1_type.into())?;

    // Function calls
    declare_fn(
        ctx,
        "typhon_call",
        &[ptr_type.into(), ptr_type.into(), i64_type.into()],
        ptr_type.into(),
    )?;

    // Exception handling
    declare_noreturn_fn(ctx, "typhon_raise", &[ptr_type.into()]);

    // Object attribute operations
    declare_fn(ctx, "typhon_getattr", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_void_fn(ctx, "typhon_setattr", &[ptr_type.into(), ptr_type.into(), ptr_type.into()]);
    declare_fn(ctx, "typhon_hasattr", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_void_fn(ctx, "typhon_delattr", &[ptr_type.into(), ptr_type.into()]);

    // Object item operations
    declare_fn(ctx, "typhon_getitem", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_void_fn(ctx, "typhon_setitem", &[ptr_type.into(), ptr_type.into(), ptr_type.into()]);
    declare_void_fn(ctx, "typhon_delitem", &[ptr_type.into(), ptr_type.into()]);

    // Type operations
    declare_fn(ctx, "typhon_isinstance", &[ptr_type.into(), ptr_type.into()], i1_type.into())?;
    declare_fn(ctx, "typhon_type_of", &[ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_cast", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;

    // Collection operations
    declare_fn(ctx, "typhon_list_new", &[i64_type.into()], ptr_type.into())?;
    declare_void_fn(ctx, "typhon_list_append", &[ptr_type.into(), ptr_type.into()]);
    declare_fn(ctx, "typhon_dict_new", &[], ptr_type.into())?;
    declare_void_fn(ctx, "typhon_dict_set", &[ptr_type.into(), ptr_type.into(), ptr_type.into()]);
    declare_fn(ctx, "typhon_tuple_new", &[i64_type.into()], ptr_type.into())?;
    declare_void_fn(ctx, "typhon_tuple_set", &[ptr_type.into(), i64_type.into(), ptr_type.into()]);

    // String operations
    declare_fn(ctx, "typhon_str_concat", &[ptr_type.into(), ptr_type.into()], ptr_type.into())?;
    declare_fn(
        ctx,
        "typhon_str_format",
        &[ptr_type.into(), ptr_type.into(), i64_type.into()],
        ptr_type.into(),
    )?;

    // Iteration operations
    declare_fn(ctx, "typhon_iter", &[ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_iter_next", &[ptr_type.into()], ptr_type.into())?;
    declare_fn(ctx, "typhon_iter_has_next", &[ptr_type.into()], i1_type.into())?;

    Ok(())
}

/// Declare a runtime function with void return type.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `name` - Function name (e.g., "`typhon_incref`")
/// - `param_types` - Array of parameter types
fn declare_void_fn<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    name: &str,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
) {
    let void_type = ctx.context.void_type();
    let fn_type = void_type.fn_type(param_types, false);
    let fn_value = ctx.module.add_function(name, fn_type, None);

    ctx.runtime_functions.insert(name.to_string(), fn_value);
}
