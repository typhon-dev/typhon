//! MIR instruction to LLVM IR translation.
//!
//! This module handles translating individual MIR instructions to LLVM IR instructions.
//! Each MIR instruction type is mapped to corresponding LLVM operations or runtime function calls.

use inkwell::AddressSpace;
use inkwell::values::{BasicValue, BasicValueEnum};
use typhon_mir::instr::{BinOpKind, MIRConst, MIRInstr, Terminator, ValueID};

use crate::context::CodegenContext;
use crate::error::{CodegenError, CodegenResult};

/// Translate a binary operation to LLVM IR.
///
/// This function handles binary operations by calling the appropriate runtime
/// functions. All operations work on boxed Typhon objects and return boxed results.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `op` - Binary operation kind
/// - `lhs` - Left-hand side value ID
/// - `rhs` - Right-hand side value ID
///
/// ## Returns
///
/// The LLVM value representing the result of the operation.
///
/// ## Errors
///
/// Returns an error if:
/// - Operand values are not found
/// - Runtime function is not declared
/// - LLVM call instruction building fails
///
/// ## Operation Mapping
///
/// - Arithmetic: `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Pow` → `typhon_<op>(lhs, rhs)`
/// - Comparison: `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge` → `typhon_<op>(lhs, rhs)` returns `i1`
/// - Other operations (logical, bitwise) are not yet implemented
fn translate_binop<'ctx>(
    ctx: &CodegenContext<'ctx>,
    op: BinOpKind,
    lhs: ValueID,
    rhs: ValueID,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let lhs_val = ctx.get_value(lhs)?;
    let rhs_val = ctx.get_value(rhs)?;

    let (fn_name, call_name) = match op {
        // Arithmetic operations
        BinOpKind::Add => ("typhon_add", "add"),
        BinOpKind::Sub => ("typhon_sub", "sub"),
        BinOpKind::Mul => ("typhon_mul", "mul"),
        BinOpKind::Div => ("typhon_div", "div"),
        BinOpKind::Mod => ("typhon_mod", "mod"),
        BinOpKind::Pow => ("typhon_pow", "pow"),

        // Comparison operations
        BinOpKind::Eq => ("typhon_eq", "eq"),
        BinOpKind::Ne => ("typhon_ne", "ne"),
        BinOpKind::Lt => ("typhon_lt", "lt"),
        BinOpKind::Le => ("typhon_le", "le"),
        BinOpKind::Gt => ("typhon_gt", "gt"),
        BinOpKind::Ge => ("typhon_ge", "ge"),

        // Not yet implemented
        _ => {
            return Err(CodegenError::InstructionTranslationError(format!(
                "Unsupported binary operation: {op:?}"
            )));
        }
    };

    let fn_val = ctx.get_runtime_function(fn_name)?;
    let call_site = ctx
        .builder
        .build_call(fn_val, &[lhs_val.into(), rhs_val.into()], call_name)
        .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

    call_site.try_as_basic_value().left().ok_or_else(|| {
        CodegenError::InstructionTranslationError(format!("{fn_name} call didn't return value"))
    })
}

/// Translate a constant value to LLVM IR.
///
/// This function handles constant value creation, either as direct LLVM constants
/// (for booleans) or by calling runtime functions to create boxed objects
/// (for integers, floats, strings, and None).
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `const_val` - MIR constant value
///
/// ## Returns
///
/// The LLVM value representing the constant.
///
/// ## Errors
///
/// Returns an error if:
/// - Runtime function call fails
/// - LLVM value creation fails
///
/// ## Constant Translation
///
/// - `Bool(b)` → Direct `i1` constant
/// - `Int(n)` → Call `typhon_int_new(i64)` to create boxed int
/// - `Float(f)` → Call `typhon_float_new(f64)` to create boxed float
/// - `Str(s)` → Call `typhon_str_new(ptr, len)` to create boxed string
/// - `None` → Call `typhon_none_get()` to get singleton None object
fn translate_const<'ctx>(
    ctx: &CodegenContext<'ctx>,
    const_val: &MIRConst,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    match const_val {
        MIRConst::Bool(b) => {
            let bool_val = ctx.context.bool_type().const_int(u64::from(*b), false);
            Ok(bool_val.as_basic_value_enum())
        }

        MIRConst::Int(n) => {
            let i64_val = ctx.context.i64_type().const_int((*n).cast_unsigned(), true);
            let fn_val = ctx.get_runtime_function("typhon_int_new")?;
            let call_site = ctx
                .builder
                .build_call(fn_val, &[i64_val.into()], "int_new")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError(
                    "typhon_int_new call didn't return value".to_string(),
                )
            })
        }

        MIRConst::Float(f) => {
            let f64_val = ctx.context.f64_type().const_float(*f);
            let fn_val = ctx.get_runtime_function("typhon_float_new")?;
            let call_site = ctx
                .builder
                .build_call(fn_val, &[f64_val.into()], "float_new")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError(
                    "typhon_float_new call didn't return value".to_string(),
                )
            })
        }

        MIRConst::Str(s) => {
            // Create a global string constant
            let string_val = ctx
                .builder
                .build_global_string_ptr(s, "str_const")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            let len_val = ctx.context.i64_type().const_int(s.len() as u64, false);
            let fn_val = ctx.get_runtime_function("typhon_str_new")?;
            let call_site = ctx
                .builder
                .build_call(
                    fn_val,
                    &[string_val.as_pointer_value().into(), len_val.into()],
                    "str_new",
                )
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError(
                    "typhon_str_new call didn't return value".to_string(),
                )
            })
        }

        MIRConst::None => {
            let fn_val = ctx.get_runtime_function("typhon_none_get")?;
            let call_site = ctx
                .builder
                .build_call(fn_val, &[], "none_get")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError(
                    "typhon_none_get call didn't return value".to_string(),
                )
            })
        }
    }
}

/// Translate a MIR instruction to LLVM IR.
///
/// This function converts a single MIR instruction into LLVM IR instructions.
/// Instructions that produce values return `Some(value)`, while instructions
/// that only have side effects (like Store, `IncRef`, `DecRef`) return `None`.
///
/// ## Supported Instructions
///
/// - **Constants**: Bool, Int, Float, Str, None
/// - **Memory Operations**: Load, Store
/// - **Arithmetic**: Add, Sub, Mul, Div, Mod, Pow
/// - **Comparison**: Eq, Ne, Lt, Le, Gt, Ge
/// - **Memory Management**: `IncRef`, `DecRef`
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `instr` - MIR instruction to translate
///
/// ## Returns
///
/// - `Ok(Some(value))` if the instruction produces a value
/// - `Ok(None)` if the instruction is a statement with no value
///
/// ## Errors
///
/// Returns an error if:
/// - Referenced values or locals are not found
/// - LLVM instruction building fails
/// - Runtime function is not declared
/// - Instruction type is not yet supported
///
/// ## Example
///
/// ```rust,ignore
/// let value = translate_instruction(&mut ctx, &instr)?;
/// if let Some(llvm_value) = value {
///     ctx.set_value(value_id, llvm_value);
/// }
/// ```
pub fn translate_instruction<'ctx>(
    ctx: &mut CodegenContext<'ctx>,
    instr: &MIRInstr,
) -> CodegenResult<Option<BasicValueEnum<'ctx>>> {
    match instr {
        MIRInstr::Const(const_val) => {
            let value = translate_const(ctx, const_val)?;

            Ok(Some(value))
        }
        MIRInstr::Load { local, ty: _ } => {
            let alloca = ctx.get_local(*local)?;
            // Load pointer type - all Typhon values are represented as pointers
            let ptr_type = ctx.context.ptr_type(AddressSpace::default());
            let loaded = ctx
                .builder
                .build_load(ptr_type, alloca, "load")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(Some(loaded))
        }
        MIRInstr::Store { local, value } => {
            let alloca = ctx.get_local(*local)?;
            let val = ctx.get_value(*value)?;
            ctx.builder
                .build_store(alloca, val)
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(None)
        }
        MIRInstr::BinOp { op, lhs, rhs, ty: _ } => {
            let value = translate_binop(ctx, *op, *lhs, *rhs)?;

            Ok(Some(value))
        }
        MIRInstr::IncRef(value_id) => {
            let val = ctx.get_value(*value_id)?;
            let fn_val = ctx.get_runtime_function("typhon_incref")?;
            ctx.builder
                .build_call(fn_val, &[val.into()], "incref")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(None)
        }
        MIRInstr::DecRef(value_id) => {
            let val = ctx.get_value(*value_id)?;
            let fn_val = ctx.get_runtime_function("typhon_decref")?;
            ctx.builder
                .build_call(fn_val, &[val.into()], "decref")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(None)
        }
        MIRInstr::Phi { incoming, ty } => {
            let phi_type = crate::types::translate_type(ctx, ty)?;
            let phi = ctx
                .builder
                .build_phi(phi_type, "phi")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            for (block_id, value_id) in incoming {
                let block = ctx.get_block(*block_id)?;
                let value = ctx.get_value(*value_id)?;
                phi.add_incoming(&[(&value, block)]);
            }

            Ok(Some(phi.as_basic_value()))
        }
        MIRInstr::Call { callee, args, ty: _ } => {
            let callee_val = ctx.get_value(*callee)?;

            // Build args array for runtime call
            let ptr_type = ctx.context.ptr_type(AddressSpace::default());
            let args_array_type = ptr_type.array_type(args.len() as u32);
            let args_alloca = ctx
                .builder
                .build_alloca(args_array_type, "args_array")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            // Store each argument into the array
            for (i, arg_id) in args.iter().enumerate() {
                let arg_val = ctx.get_value(*arg_id)?;
                let i32_const = ctx.context.i32_type().const_int(i as u64, false);
                // SAFETY: GEP indices are valid - zero for array, i for element
                let gep = unsafe {
                    ctx.builder.build_in_bounds_gep(
                        args_array_type,
                        args_alloca,
                        &[ctx.context.i32_type().const_zero(), i32_const],
                        &format!("arg_{i}"),
                    )
                }
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

                ctx.builder
                    .build_store(gep, arg_val)
                    .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            }

            // Call typhon_call runtime function
            let num_args = ctx.context.i64_type().const_int(args.len() as u64, false);
            let fn_val = ctx.get_runtime_function("typhon_call")?;
            let call_site = ctx
                .builder
                .build_call(
                    fn_val,
                    &[callee_val.into(), args_alloca.into(), num_args.into()],
                    "call",
                )
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            let result = call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError(
                    "typhon_call didn't return value".to_string(),
                )
            })?;

            Ok(Some(result))
        }
        MIRInstr::GetAttr { object, attr, ty: _ } => {
            // Get object value
            let obj_val = ctx.get_value(*object)?;

            // Create string constant for attribute name
            let name_str = ctx
                .builder
                .build_global_string_ptr(attr, "attr_name")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            // Call runtime function
            let fn_val = ctx.get_runtime_function("typhon_getattr")?;
            let call_site = ctx
                .builder
                .build_call(
                    fn_val,
                    &[obj_val.into(), name_str.as_pointer_value().into()],
                    "getattr",
                )
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(Some(call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError("GetAttr didn't return value".to_string())
            })?))
        }
        MIRInstr::SetAttr { object, attr, value } => {
            let obj_val = ctx.get_value(*object)?;
            let value_val = ctx.get_value(*value)?;
            let name_str = ctx
                .builder
                .build_global_string_ptr(attr, "attr_name")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            let fn_val = ctx.get_runtime_function("typhon_setattr")?;
            ctx.builder
                .build_call(
                    fn_val,
                    &[obj_val.into(), name_str.as_pointer_value().into(), value_val.into()],
                    "setattr",
                )
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(None) // SetAttr is void
        }
        MIRInstr::GetItem { object, key, ty: _ } => {
            let obj_val = ctx.get_value(*object)?;
            let key_val = ctx.get_value(*key)?;

            let fn_val = ctx.get_runtime_function("typhon_getitem")?;
            let call_site = ctx
                .builder
                .build_call(fn_val, &[obj_val.into(), key_val.into()], "getitem")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(Some(call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError("GetItem didn't return value".to_string())
            })?))
        }
        MIRInstr::SetItem { object, key, value } => {
            let obj_val = ctx.get_value(*object)?;
            let key_val = ctx.get_value(*key)?;
            let value_val = ctx.get_value(*value)?;

            let fn_val = ctx.get_runtime_function("typhon_setitem")?;
            ctx.builder
                .build_call(fn_val, &[obj_val.into(), key_val.into(), value_val.into()], "setitem")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(None) // SetItem is void
        }
        MIRInstr::InstanceOf { object, type_id } => {
            let obj_val = ctx.get_value(*object)?;
            // For now, treat TypeID as an integer that can be converted to a pointer
            // In a full implementation, this would retrieve the actual type object
            let type_val = ctx
                .context
                .i64_type()
                .const_int(*type_id as u64, false)
                .const_to_pointer(ctx.context.ptr_type(AddressSpace::default()));

            let fn_val = ctx.get_runtime_function("typhon_isinstance")?;
            let call_site = ctx
                .builder
                .build_call(fn_val, &[obj_val.into(), type_val.into()], "isinstance")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(Some(call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError(
                    "InstanceOf didn't return value".to_string(),
                )
            })?))
        }
        MIRInstr::Cast { value, target_ty: _ } => {
            let obj_val = ctx.get_value(*value)?;
            // For cast, we need the target type as a runtime object
            // For now, use a null pointer as placeholder - full implementation
            // would retrieve the actual type object
            let type_val = ctx.context.ptr_type(AddressSpace::default()).const_null();

            let fn_val = ctx.get_runtime_function("typhon_cast")?;
            let call_site = ctx
                .builder
                .build_call(fn_val, &[obj_val.into(), type_val.into()], "cast")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;

            Ok(Some(call_site.try_as_basic_value().left().ok_or_else(|| {
                CodegenError::InstructionTranslationError("Cast didn't return value".to_string())
            })?))
        }
        _ => Err(CodegenError::InstructionTranslationError(format!(
            "Unsupported instruction: {instr:?}"
        ))),
    }
}

/// Translate a MIR terminator to LLVM IR.
///
/// Terminators control the flow between basic blocks and include returns,
/// branches, and exception handling. Each terminator ends a basic block and
/// determines where control flow continues.
///
/// ## Arguments
///
/// - `ctx` - Code generation context
/// - `terminator` - MIR terminator to translate
///
/// ## Returns
///
/// `Ok(())` if the terminator was translated successfully.
///
/// ## Errors
///
/// Returns an error if:
/// - Referenced values or blocks are not found
/// - LLVM instruction building fails
/// - Runtime function is not declared
/// - Terminator type is not yet supported
///
/// ## Supported Terminators
///
/// - `Return(Some(value))` - Return a value from the function
/// - `Return(None)` - Return void from the function
/// - `Branch(target)` - Unconditional branch to target block
/// - `CondBranch { condition, then_block, else_block }` - Conditional branch based on condition
/// - `Unreachable` - Marks unreachable code
/// - `Raise(exception)` - Raise an exception and mark code as unreachable
/// - `Invoke` - Not yet implemented, returns error
pub fn translate_terminator(
    ctx: &mut CodegenContext<'_>,
    terminator: &Terminator,
) -> CodegenResult<()> {
    match terminator {
        Terminator::Return(Some(value_id)) => {
            let return_val = ctx.get_value(*value_id)?;
            ctx.builder
                .build_return(Some(&return_val))
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
        }
        Terminator::Return(None) => {
            ctx.builder
                .build_return(None)
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
        }
        Terminator::Branch(target_block) => {
            let target = ctx.get_block(*target_block)?;
            ctx.builder
                .build_unconditional_branch(target)
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
        }
        Terminator::CondBranch { condition, then_block, else_block } => {
            let cond_val = ctx.get_value(*condition)?;
            let then_bb = ctx.get_block(*then_block)?;
            let else_bb = ctx.get_block(*else_block)?;

            // Convert to i1 if needed
            let cond_i1 = if cond_val.get_type() == ctx.context.bool_type().into() {
                cond_val.into_int_value()
            } else {
                // Call runtime to convert object to bool
                let fn_val = ctx.get_runtime_function("typhon_is_truthy")?;
                let call_site = ctx
                    .builder
                    .build_call(fn_val, &[cond_val.into()], "is_truthy")
                    .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| {
                        CodegenError::InstructionTranslationError(
                            "typhon_is_truthy didn't return value".to_string(),
                        )
                    })?
                    .into_int_value()
            };

            ctx.builder
                .build_conditional_branch(cond_i1, then_bb, else_bb)
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
        }
        Terminator::Unreachable => {
            ctx.builder
                .build_unreachable()
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
        }
        Terminator::Raise(exception) => {
            let exc_val = ctx.get_value(*exception)?;
            let fn_val = ctx.get_runtime_function("typhon_raise")?;
            ctx.builder
                .build_call(fn_val, &[exc_val.into()], "raise")
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
            ctx.builder
                .build_unreachable()
                .map_err(|e| CodegenError::InstructionTranslationError(e.to_string()))?;
        }
        Terminator::Invoke { .. } => {
            return Err(CodegenError::InstructionTranslationError(
                "Invoke terminator not yet implemented".to_string(),
            ));
        }
    }

    Ok(())
}
