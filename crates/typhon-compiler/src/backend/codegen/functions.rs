//! This module handles function code generation.

use inkwell::types::{AnyType, AnyTypeEnum};
use inkwell::values::BasicValueEnum;

use super::context::CodeGenContext;

/// Build a store instruction.
pub fn build_store(context: &CodeGenContext, ptr: BasicValueEnum, value: BasicValueEnum) {
    // Check if the pointer is actually a pointer type
    let ptr_value = ptr.into_pointer_value();
    let builder = context.llvm_context.builder();
    builder.build_store(ptr_value, value).expect("Failed to build store instruction");
}

/// Build a load instruction.
pub fn build_load(context: &CodeGenContext, ptr: BasicValueEnum, name: &str) -> BasicValueEnum {
    // Check if the pointer is actually a pointer type
    let ptr_value = ptr.into_pointer_value();
    // Get the type of the pointed-to value
    let pointee_type = ptr_value.get_type().as_any_type_enum();

    // Build the load based on the pointee type
    let builder = context.llvm_context.builder();

    match pointee_type {
        AnyTypeEnum::IntType(_) => builder
            .build_load(pointee_type.into_int_type(), ptr_value, name)
            .expect("Failed to build int load instruction"),
        AnyTypeEnum::FloatType(_) => builder
            .build_load(pointee_type.into_float_type(), ptr_value, name)
            .expect("Failed to build float load instruction"),
        AnyTypeEnum::PointerType(_) => builder
            .build_load(pointee_type.into_pointer_type(), ptr_value, name)
            .expect("Failed to build pointer load instruction"),
        AnyTypeEnum::StructType(_) => builder
            .build_load(pointee_type.into_struct_type(), ptr_value, name)
            .expect("Failed to build struct load instruction"),
        AnyTypeEnum::ArrayType(_) => builder
            .build_load(pointee_type.into_array_type(), ptr_value, name)
            .expect("Failed to build array load instruction"),
        _ => panic!("Unsupported pointee type for load: {pointee_type:?}"),
    }
}
