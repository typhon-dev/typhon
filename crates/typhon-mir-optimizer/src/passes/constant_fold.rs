//! Constant Folding Pass
//!
//! This pass evaluates constant expressions at compile time and replaces them with their
//! computed values. This reduces runtime computation and enables further optimizations
//! like dead code elimination.

use indexmap::IndexMap;
use typhon_mir::function::MIRFunction;
use typhon_mir::instr::{BinOpKind, MIRConst, MIRInstr, UnOpKind, ValueID};

use crate::error::OptimizerResult;

/// Result of attempting to fold a constant expression.
#[derive(Debug)]
enum FoldResult {
    /// Expression was folded to a constant.
    Folded(MIRConst, ValueID),
    /// Expression cannot be folded.
    Unchanged,
}

/// Constant folding optimization pass.
///
/// This pass evaluates constant expressions at compile time, including:
///
/// - Arithmetic operations (`Add`, `Sub`, `Mul`, `Div`, `Mod`, `Pow`)
/// - Comparison operations (`Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`)
/// - Logical operations (`And`, `Or`, `Not`)
/// - Bitwise operations (`BitAnd`, `BitOr`, `BitXor`, `ShiftLeft`, `ShiftRight`)
/// - String concatenation
/// - Constant propagation through assignments
#[derive(Debug)]
pub struct ConstantFolder {
    /// Map of known constant values.
    constants: IndexMap<ValueID, MIRConst>,
    /// Number of constants folded in this pass.
    fold_count: usize,
}

impl ConstantFolder {
    /// Create a new constant folder.
    #[must_use]
    pub fn new() -> Self { Self { constants: IndexMap::new(), fold_count: 0 } }

    /// Fold all constant expressions in the function.
    ///
    /// Returns the number of constants folded.
    ///
    /// # Errors
    ///
    /// Returns an error if constant folding fails due to invalid operations.
    pub fn fold_constants(&mut self, func: &mut MIRFunction) -> OptimizerResult<usize> {
        self.constants.clear();
        self.fold_count = 0;

        let mut value_counter = 0u32;
        let mut changed = true;

        // Iterate until no more changes (fixed-point)
        while changed {
            changed = false;

            for block in &mut func.blocks {
                for instr in &mut block.instrs {
                    let value_id = ValueID(value_counter);
                    value_counter += 1;

                    match self.try_fold_instruction(instr, value_id) {
                        FoldResult::Folded(const_val, vid) => {
                            // Replace instruction with constant
                            *instr = MIRInstr::Const(const_val.clone());
                            drop(self.constants.insert(vid, const_val));
                            self.fold_count += 1;
                            changed = true;
                        }
                        FoldResult::Unchanged => {
                            // Check if this is a constant instruction
                            if let MIRInstr::Const(c) = instr {
                                drop(self.constants.insert(value_id, c.clone()));
                            }
                        }
                    }
                }
            }
        }

        Ok(self.fold_count)
    }

    /// Attempt to fold a single instruction.
    fn try_fold_instruction(&self, instr: &MIRInstr, result_id: ValueID) -> FoldResult {
        match instr {
            MIRInstr::BinOp { op, lhs, rhs, .. } => {
                if let (Some(lhs_const), Some(rhs_const)) =
                    (self.constants.get(lhs), self.constants.get(rhs))
                    && let Some(result) = self.fold_binop(*op, lhs_const, rhs_const)
                {
                    return FoldResult::Folded(result, result_id);
                }
            }
            MIRInstr::UnOp { op, operand, .. } => {
                if let Some(operand_const) = self.constants.get(operand)
                    && let Some(result) = self.fold_unop(*op, operand_const)
                {
                    return FoldResult::Folded(result, result_id);
                }
            }
            MIRInstr::Phi { incoming, .. } => {
                // If all incoming values are the same constant, fold the phi
                if !incoming.is_empty() {
                    let first_val = &incoming[0].1;
                    if let Some(first_const) = self.constants.get(first_val)
                        && incoming.iter().all(|(_, vid)| {
                            self.constants.get(vid).is_some_and(|c| constants_equal(c, first_const))
                        })
                    {
                        return FoldResult::Folded(first_const.clone(), result_id);
                    }
                }
            }
            _ => {}
        }

        FoldResult::Unchanged
    }

    /// Fold a binary operation on constants.
    fn fold_binop(&self, op: BinOpKind, lhs: &MIRConst, rhs: &MIRConst) -> Option<MIRConst> {
        match (lhs, rhs) {
            (MIRConst::Int(l), MIRConst::Int(r)) => self.fold_int_binop(op, *l, *r),
            (MIRConst::Float(l), MIRConst::Float(r)) => self.fold_float_binop(op, *l, *r),
            (MIRConst::Bool(l), MIRConst::Bool(r)) => self.fold_bool_binop(op, *l, *r),
            (MIRConst::Str(l), MIRConst::Str(r)) => self.fold_str_binop(op, l, r),
            _ => None,
        }
    }

    /// Fold an integer binary operation.
    fn fold_int_binop(&self, op: BinOpKind, lhs: i64, rhs: i64) -> Option<MIRConst> {
        match op {
            // Arithmetic
            BinOpKind::Add => Some(MIRConst::Int(lhs.wrapping_add(rhs))),
            BinOpKind::Sub => Some(MIRConst::Int(lhs.wrapping_sub(rhs))),
            BinOpKind::Mul => Some(MIRConst::Int(lhs.wrapping_mul(rhs))),
            BinOpKind::Div => {
                if rhs == 0 {
                    None
                } else {
                    Some(MIRConst::Int(lhs / rhs))
                }
            }
            BinOpKind::FloorDiv => {
                if rhs == 0 {
                    None
                } else {
                    Some(MIRConst::Int(lhs.div_euclid(rhs)))
                }
            }
            BinOpKind::Mod => {
                if rhs == 0 {
                    None
                } else {
                    Some(MIRConst::Int(lhs.rem_euclid(rhs)))
                }
            }
            BinOpKind::Pow => {
                if rhs < 0 {
                    None
                } else {
                    lhs.checked_pow(rhs as u32).map(MIRConst::Int)
                }
            }
            // Comparison
            BinOpKind::Eq => Some(MIRConst::Bool(lhs == rhs)),
            BinOpKind::Ne => Some(MIRConst::Bool(lhs != rhs)),
            BinOpKind::Lt => Some(MIRConst::Bool(lhs < rhs)),
            BinOpKind::Le => Some(MIRConst::Bool(lhs <= rhs)),
            BinOpKind::Gt => Some(MIRConst::Bool(lhs > rhs)),
            BinOpKind::Ge => Some(MIRConst::Bool(lhs >= rhs)),
            // Bitwise
            BinOpKind::BitAnd => Some(MIRConst::Int(lhs & rhs)),
            BinOpKind::BitOr => Some(MIRConst::Int(lhs | rhs)),
            BinOpKind::BitXor => Some(MIRConst::Int(lhs ^ rhs)),
            BinOpKind::Shl => {
                if (0..64).contains(&rhs) {
                    Some(MIRConst::Int(lhs << rhs))
                } else {
                    None
                }
            }
            BinOpKind::Shr => {
                if (0..64).contains(&rhs) {
                    Some(MIRConst::Int(lhs >> rhs))
                } else {
                    None
                }
            }
            // Logical operations don't apply to integers
            BinOpKind::And | BinOpKind::Or => None,
        }
    }

    /// Fold a float binary operation.
    fn fold_float_binop(&self, op: BinOpKind, lhs: f64, rhs: f64) -> Option<MIRConst> {
        match op {
            BinOpKind::Add => Some(MIRConst::Float(lhs + rhs)),
            BinOpKind::Sub => Some(MIRConst::Float(lhs - rhs)),
            BinOpKind::Mul => Some(MIRConst::Float(lhs * rhs)),
            BinOpKind::Div | BinOpKind::FloorDiv => Some(MIRConst::Float(lhs / rhs)),
            BinOpKind::Mod => Some(MIRConst::Float(lhs % rhs)),
            BinOpKind::Pow => Some(MIRConst::Float(lhs.powf(rhs))),
            BinOpKind::Eq => Some(MIRConst::Bool(lhs == rhs)),
            BinOpKind::Ne => Some(MIRConst::Bool(lhs != rhs)),
            BinOpKind::Lt => Some(MIRConst::Bool(lhs < rhs)),
            BinOpKind::Le => Some(MIRConst::Bool(lhs <= rhs)),
            BinOpKind::Gt => Some(MIRConst::Bool(lhs > rhs)),
            BinOpKind::Ge => Some(MIRConst::Bool(lhs >= rhs)),
            _ => None,
        }
    }

    /// Fold a boolean binary operation.
    const fn fold_bool_binop(&self, op: BinOpKind, lhs: bool, rhs: bool) -> Option<MIRConst> {
        match op {
            BinOpKind::And => Some(MIRConst::Bool(lhs && rhs)),
            BinOpKind::Or => Some(MIRConst::Bool(lhs || rhs)),
            BinOpKind::Eq => Some(MIRConst::Bool(lhs == rhs)),
            BinOpKind::Ne => Some(MIRConst::Bool(lhs != rhs)),
            _ => None,
        }
    }

    /// Fold a string binary operation.
    fn fold_str_binop(&self, op: BinOpKind, lhs: &str, rhs: &str) -> Option<MIRConst> {
        match op {
            BinOpKind::Add => Some(MIRConst::Str(format!("{lhs}{rhs}"))),
            BinOpKind::Eq => Some(MIRConst::Bool(lhs == rhs)),
            BinOpKind::Ne => Some(MIRConst::Bool(lhs != rhs)),
            BinOpKind::Lt => Some(MIRConst::Bool(lhs < rhs)),
            BinOpKind::Le => Some(MIRConst::Bool(lhs <= rhs)),
            BinOpKind::Gt => Some(MIRConst::Bool(lhs > rhs)),
            BinOpKind::Ge => Some(MIRConst::Bool(lhs >= rhs)),
            _ => None,
        }
    }

    /// Fold a unary operation on a constant.
    fn fold_unop(&self, op: UnOpKind, operand: &MIRConst) -> Option<MIRConst> {
        match (op, operand) {
            (UnOpKind::Neg, MIRConst::Int(val)) => Some(MIRConst::Int(-val)),
            (UnOpKind::Neg, MIRConst::Float(val)) => Some(MIRConst::Float(-val)),
            (UnOpKind::Not, MIRConst::Bool(val)) => Some(MIRConst::Bool(!val)),
            (UnOpKind::BitNot, MIRConst::Int(val)) => Some(MIRConst::Int(!val)),
            _ => None,
        }
    }

    /// Get the number of constants folded in the last run.
    #[must_use]
    pub const fn fold_count(&self) -> usize { self.fold_count }
}

impl Default for ConstantFolder {
    fn default() -> Self { Self::new() }
}

/// Check if two constants are equal.
fn constants_equal(a: &MIRConst, b: &MIRConst) -> bool {
    match (a, b) {
        (MIRConst::Int(a), MIRConst::Int(b)) => a == b,
        (MIRConst::Float(a), MIRConst::Float(b)) => a == b,
        (MIRConst::Bool(a), MIRConst::Bool(b)) => a == b,
        (MIRConst::Str(a), MIRConst::Str(b)) => a == b,
        (MIRConst::None, MIRConst::None) => true,
        _ => false,
    }
}
