//! Test-specific validation helpers.

use typhon_mir::function::MIRFunction;
use typhon_mir::instr::MIRInstr;

/// Helper to count `BinOp` instructions in a function.
pub fn count_binop_instrs(func: &MIRFunction) -> usize {
    func.blocks[0].instrs.iter().filter(|instr| matches!(instr, MIRInstr::BinOp { .. })).count()
}

/// Helper to count basic blocks in a function.
pub fn count_blocks(func: &MIRFunction) -> usize { func.blocks.len() }

/// Helper to count Const instructions in a function.
pub fn count_const_instrs(func: &MIRFunction) -> usize {
    func.blocks[0].instrs.iter().filter(|instr| matches!(instr, MIRInstr::Const(_))).count()
}

/// Helper to count instructions in a function.
pub fn count_instrs(func: &MIRFunction) -> usize {
    func.blocks.iter().map(|block| block.instrs.len()).sum()
}
