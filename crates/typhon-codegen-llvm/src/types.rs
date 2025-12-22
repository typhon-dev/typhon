//! MIR type to LLVM type translation.
//!
//! This module handles translating Typhon MIR types to their corresponding LLVM IR type
//! representations. All Typhon objects are represented as pointers to heap-allocated structs
//! in LLVM, enabling uniform object representation, reference counting, runtime type
//! information, and dynamic dispatch.
//!
//! ## Type Translation Strategy
//!
//! The translation follows these principles:
//!
//! - **Primitive types** like [`MIRType::Bool`](typhon_mir::types::MIRType::Bool) use native
//!   LLVM types (`i1` for booleans)
//! - **Object types** are represented as pointers to struct types defined in the object layout
//!   (e.g., `%TyphonInt*` for integers)
//! - **Container types** use specialized struct pointers (e.g., `%TyphonList*` for lists)
//! - **Function and closure types** are represented as pointers to callable objects
//! - **Reference types** are represented as pointer-to-pointer types
//!
//! ## Type Caching
//!
//! Type translations are cached to avoid redundant work. The cache is stored in the
//! [`CodegenContext`] and is consulted before performing
//! translation.
//!
//! ## Examples
//!
//! Translating a simple integer type:
//!
//! ```ignore
//! use typhon_mir::types::MIRType;
//!
//! let int_type = translate_type(&ctx, &MIRType::Int)?;
//! // Returns: %TyphonInt* (pointer to TyphonInt struct)
//! ```
//!
//! Translating a list type:
//!
//! ```ignore
//! let list_type = translate_type(&ctx, &MIRType::List(Box::new(MIRType::Int)))?;
//! // Returns: %TyphonList* (pointer to TyphonList struct)
//! ```

use inkwell::AddressSpace;
use inkwell::types::{BasicType, BasicTypeEnum};
use typhon_mir::types::MIRType;

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// Translates a MIR type to its corresponding LLVM type representation.
///
/// This function first checks the type cache to see if the type has already been translated.
/// If found in the cache, the cached type is returned immediately. Otherwise, it delegates
/// to `translate_type_uncached` to perform the actual translation.
///
/// All Typhon objects are represented as pointers to heap-allocated structs. Primitive types
/// like `Bool` use native LLVM integer types.
///
/// ## Arguments
///
/// - `ctx` - Code generation context containing type cache and struct declarations
/// - `ty` - MIR type to translate
///
/// ## Returns
///
/// The corresponding LLVM basic type, typically a pointer type for objects or a primitive
/// type for booleans.
///
/// ## Errors
///
/// Returns [`CodegenError::VoidTypeNotAllowed`] if attempting to translate [`MIRType::Void`].
/// Returns [`CodegenError::StructTypeNotFound`] if a required struct type was not declared.
///
/// ## Examples
///
/// ```ignore
/// let int_type = translate_type(&ctx, &MIRType::Int)?;
/// assert!(int_type.is_pointer_type());
/// ```
pub fn translate_type<'ctx>(
    ctx: &CodegenContext<'ctx>,
    ty: &MIRType,
) -> CodegenResult<BasicTypeEnum<'ctx>> {
    if let Some(cached) = ctx.type_cache.get(ty) {
        return Ok(*cached);
    }

    translate_type_uncached(ctx, ty)
}

/// Translates and caches a MIR type to LLVM type.
///
/// This variant requires mutable access to the context to enable caching. Use this when
/// you have mutable context access and want to populate the cache for future lookups.
///
/// ## Arguments
///
/// - `ctx` - Mutable code generation context
/// - `ty` - MIR type to translate
///
/// ## Returns
///
/// The corresponding LLVM basic type.
///
/// ## Errors
///
/// Returns [`CodegenError::VoidTypeNotAllowed`] if attempting to translate [`MIRType::Void`].
/// Returns [`CodegenError::StructTypeNotFound`] if a required struct type was not declared.
///
/// ## Examples
///
/// ```ignore
/// let float_type = translate_and_cache_type(&mut ctx, &MIRType::Float)?;
/// // Type is now cached for future lookups
/// ```
pub fn translate_and_cache_type<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    ty: &MIRType,
) -> CodegenResult<BasicTypeEnum<'ctx>> {
    if let Some(cached) = ctx.type_cache.get(ty) {
        return Ok(*cached);
    }

    let llvm_type = translate_type_uncached(ctx, ty)?;
    ctx.cache_type(ty.clone(), llvm_type);
    Ok(llvm_type)
}

/// Performs the actual type translation without consulting the cache.
///
/// This function handles the translation of all MIR type variants to their LLVM
/// representations. It should not be called directly; use [`translate_type`] or
/// [`translate_and_cache_type`] instead.
///
/// ## Type Mappings
///
/// - `Void` → Error (not allowed as value type)
/// - `Bool` → `i1` (1-bit integer)
/// - `Int` → `%TyphonInt*` (pointer to integer object)
/// - `Float` → `%TyphonFloat*` (pointer to float object)
/// - `Str` → `%TyphonStr*` (pointer to string object)
/// - `None` → `%TyphonNone*` (pointer to None singleton)
/// - `Object{type_id}` → `%TyphonObject*` (base object pointer)
/// - `List(elem)` → `%TyphonList*` (pointer to list object)
/// - `Dict{key, value}` → `%TyphonDict*` (pointer to dict object)
/// - `Tuple(types)` → `%TyphonTuple*` (pointer to tuple object)
/// - `Function{params, ret}` → `%TyphonFunction*` (pointer to function object)
/// - `Closure{params, ret, cap}` → `%TyphonClosure*` (pointer to closure object)
/// - `Ref(inner)` → `inner*` (pointer to inner type)
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `ty` - MIR type to translate
///
/// ## Returns
///
/// The corresponding LLVM basic type.
///
/// ## Errors
///
/// Returns [`CodegenError::VoidTypeNotAllowed`] if `ty` is [`MIRType::Void`].
/// Returns [`CodegenError::StructTypeNotFound`] if a required struct type was not declared.
fn translate_type_uncached<'ctx>(
    ctx: &CodegenContext<'ctx>,
    ty: &MIRType,
) -> CodegenResult<BasicTypeEnum<'ctx>> {
    match ty {
        MIRType::Void => Err(CodegenError::VoidTypeNotAllowed),
        MIRType::Bool => Ok(ctx.context.bool_type().as_basic_type_enum()),
        MIRType::Closure { .. } => {
            ctx.get_struct_type("TyphonClosure")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Dict { .. } => {
            ctx.get_struct_type("TyphonDict")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Float => {
            ctx.get_struct_type("TyphonFloat")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Function { .. } => {
            ctx.get_struct_type("TyphonFunction")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Int => {
            ctx.get_struct_type("TyphonInt")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::List(_) => {
            ctx.get_struct_type("TyphonList")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::None => {
            ctx.get_struct_type("TyphonNone")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Object { .. } => {
            ctx.get_struct_type("TyphonObject")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Ref(inner) => {
            translate_type(ctx, inner)?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Str => {
            ctx.get_struct_type("TyphonStr")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
        MIRType::Tuple(_) => {
            ctx.get_struct_type("TyphonTuple")?;

            Ok(ctx.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
        }
    }
}
