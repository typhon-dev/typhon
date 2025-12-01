use inkwell::builder::BuilderError;
use inkwell::values::{BasicValue, BasicValueEnum};
use inkwell::{FloatPredicate, IntPredicate};

use super::generator::CodeGenerator;
use crate::backend::CodeGenError;
use crate::backend::error::CodeGenResult;
use crate::frontend::ast::{BinaryOperator, UnaryOperator};

/// Extension trait for binary and unary operations on CodeGenerator
pub trait CodeGenOperations {
    /// Build a binary operation.
    fn build_binary_op(
        &self,
        op: BinaryOperator,
        left: BasicValueEnum,
        right: BasicValueEnum,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum>;

    /// Build a unary operation.
    fn build_unary_op(
        &self,
        op: UnaryOperator,
        operand: BasicValueEnum,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum>;
}

impl CodeGenOperations for CodeGenerator {
    fn build_binary_op(
        &self,
        op: BinaryOperator,
        left: BasicValueEnum,
        right: BasicValueEnum,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum> {
        // Ensure both operands are of the same type
        if left.get_type() != right.get_type() {
            return Err(crate::backend::error::CodeGenError::code_gen_error(
                format!(
                    "Mismatched operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ));
        }

        // Get builder directly from our context
        let builder = self.context.llvm_context.builder();

        // Handle integer operations
        if left.is_int_value() && right.is_int_value() {
            let (left_int, right_int) = (left.into_int_value(), right.into_int_value());

            match op {
                BinaryOperator::Add => op_result(
                    builder.build_int_add(left_int, right_int, name),
                    "Failed to build integer addition",
                ),
                BinaryOperator::Sub => op_result(
                    builder.build_int_sub(left_int, right_int, name),
                    "Failed to build integer subtraction",
                ),
                BinaryOperator::Mul => op_result(
                    builder.build_int_mul(left_int, right_int, name),
                    "Failed to build integer multiplication",
                ),
                BinaryOperator::Div => op_result(
                    builder.build_int_signed_div(left_int, right_int, name),
                    "Failed to build integer division",
                ),
                BinaryOperator::Mod => op_result(
                    builder.build_int_signed_rem(left_int, right_int, name),
                    "Failed to build integer remainder",
                ),
                BinaryOperator::Eq => op_result(
                    builder.build_int_compare(IntPredicate::EQ, left_int, right_int, name),
                    "Failed to build integer equality comparison",
                ),
                BinaryOperator::NotEq => op_result(
                    builder.build_int_compare(IntPredicate::NE, left_int, right_int, name),
                    "Failed to build integer inequality comparison",
                ),
                BinaryOperator::Lt => op_result(
                    builder.build_int_compare(IntPredicate::SLT, left_int, right_int, name),
                    "Failed to build integer less-than comparison",
                ),
                BinaryOperator::LtE => op_result(
                    builder.build_int_compare(IntPredicate::SLE, left_int, right_int, name),
                    "Failed to build integer less-than-or-equal comparison",
                ),
                BinaryOperator::Gt => op_result(
                    builder.build_int_compare(IntPredicate::SGT, left_int, right_int, name),
                    "Failed to build integer greater-than comparison",
                ),
                BinaryOperator::GtE => op_result(
                    builder.build_int_compare(IntPredicate::SGE, left_int, right_int, name),
                    "Failed to build integer greater-than-or-equal comparison",
                ),
                BinaryOperator::BitAnd => op_result(
                    builder.build_and(left_int, right_int, name),
                    "Failed to build bitwise AND operation",
                ),
                BinaryOperator::BitOr => op_result(
                    builder.build_or(left_int, right_int, name),
                    "Failed to build bitwise OR operation",
                ),
                BinaryOperator::BitXor => op_result(
                    builder.build_xor(left_int, right_int, name),
                    "Failed to build bitwise XOR operation",
                ),
                BinaryOperator::LShift => op_result(
                    builder.build_left_shift(left_int, right_int, name),
                    "Failed to build left shift operation",
                ),
                BinaryOperator::RShift => op_result(
                    builder.build_right_shift(left_int, right_int, true, name),
                    "Failed to build right shift operation",
                ),
                _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation: {op:?}"),
                    None,
                )),
            }
        }
        // Handle float operations
        else if left.is_float_value() && right.is_float_value() {
            let (left_float, right_float) = (left.into_float_value(), right.into_float_value());

            match op {
                BinaryOperator::Add => op_result(
                    builder.build_float_add(left_float, right_float, name),
                    "Failed to build float addition",
                ),
                BinaryOperator::Sub => op_result(
                    builder.build_float_sub(left_float, right_float, name),
                    "Failed to build float subtraction",
                ),
                BinaryOperator::Mul => op_result(
                    builder.build_float_mul(left_float, right_float, name),
                    "Failed to build float multiplication",
                ),
                BinaryOperator::Div => op_result(
                    builder.build_float_div(left_float, right_float, name),
                    "Failed to build float division",
                ),
                BinaryOperator::Eq => op_result(
                    builder.build_float_compare(FloatPredicate::OEQ, left_float, right_float, name),
                    "Failed to build float equality comparison",
                ),
                BinaryOperator::NotEq => op_result(
                    builder.build_float_compare(FloatPredicate::ONE, left_float, right_float, name),
                    "Failed to build float inequality comparison",
                ),
                BinaryOperator::Lt => op_result(
                    builder.build_float_compare(FloatPredicate::OLT, left_float, right_float, name),
                    "Failed to build float less-than comparison",
                ),
                BinaryOperator::LtE => op_result(
                    builder.build_float_compare(FloatPredicate::OLE, left_float, right_float, name),
                    "Failed to build float less-than-or-equal comparison",
                ),
                BinaryOperator::Gt => op_result(
                    builder.build_float_compare(FloatPredicate::OGT, left_float, right_float, name),
                    "Failed to build float greater-than comparison",
                ),
                BinaryOperator::GtE => op_result(
                    builder.build_float_compare(FloatPredicate::OGE, left_float, right_float, name),
                    "Failed to build float greater-than-or-equal comparison",
                ),

                _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation for float: {op:?}"),
                    None,
                )),
            }
        } else {
            Err(crate::backend::error::CodeGenError::unsupported_feature(
                format!(
                    "Unsupported operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ))
        }
    }

    fn build_unary_op(
        &self,
        op: UnaryOperator,
        operand: BasicValueEnum,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum> {
        let builder = self.context.llvm_context.builder();

        // Handle integer operations
        if operand.is_int_value() {
            let int_operand = operand.into_int_value();

            match op {
                UnaryOperator::Pos => {
                    // For integers, positive is the value itself
                    Ok(int_operand.into())
                }
                UnaryOperator::Neg => {
                    // For integers, negation is 0 - operand
                    let zero = int_operand.get_type().const_int(0, false);
                    op_result(
                        builder.build_int_sub(zero, int_operand, name),
                        "Failed to build integer negation",
                    )
                }
                UnaryOperator::Not => {
                    // For integers, logical not is comparison with zero
                    let zero = int_operand.get_type().const_int(0, false);
                    op_result(
                        builder.build_int_compare(IntPredicate::EQ, int_operand, zero, name),
                        "Failed to build integer comparison for logical not",
                    )
                }
                UnaryOperator::Invert => {
                    // Bitwise not is XOR with all ones
                    let all_ones = int_operand.get_type().const_all_ones();
                    op_result(
                        builder.build_xor(int_operand, all_ones, name),
                        "Failed to build bitwise NOT operation",
                    )
                }
            }
        }
        // Handle float operations
        else if operand.is_float_value() {
            let float_operand = operand.into_float_value();
            match op {
                UnaryOperator::Pos => Ok(float_operand.into()),
                UnaryOperator::Neg => op_result(
                    builder.build_float_neg(float_operand, name),
                    "Failed to build float negation",
                ),
                UnaryOperator::Not => {
                    // For floats, logical not is comparison with zero
                    let zero = float_operand.get_type().const_float(0.0);
                    op_result(
                        builder.build_float_compare(FloatPredicate::OEQ, float_operand, zero, name),
                        "Failed to build float comparison for logical not",
                    )
                }
                _ => Err(crate::backend::error::CodeGenError::unsupported_feature(
                    format!("Unsupported unary operation for float: {op:?}"),
                    None,
                )),
            }
        } else {
            Err(crate::backend::error::CodeGenError::unsupported_feature(
                format!("Unsupported operand type for unary operation: {:?}", operand.get_type()),
                None,
            ))
        }
    }
}

fn op_result(
    result: Result<BasicValue, BuilderError>,
    message: &str,
) -> CodeGenResult<BasicValueEnum> {
    match result {
        Ok(result) => Ok(result.as_basic_value_enum()),
        Err(_) => Err(CodeGenError::code_gen_error(message, None)),
    }
}
