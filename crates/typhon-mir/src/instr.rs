//! MIR Instructions
//!
//! This module defines the instruction set for Typhon's Mid-level Intermediate Representation.
//! Instructions operate on SSA values and represent all operations in the MIR.

use crate::types::{MIRType, TypeID};

/// Unique identifier for a value in SSA form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueID(pub u32);

/// Unique identifier for a local variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalID(pub u32);

/// Unique identifier for a basic block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockID(pub u32);

/// MIR instruction
#[derive(Debug, Clone)]
pub enum MIRInstr {
    // ===== Memory Management =====
    /// Decrement reference count
    DecRef(ValueID),
    /// Increment reference count
    IncRef(ValueID),

    // ===== Object Operations =====
    /// Allocate new object
    AllocObject { type_id: TypeID, size: usize },
    /// Get item from container (`obj[key]`)
    GetItem { object: ValueID, key: ValueID, ty: MIRType },
    /// Set item in container (`obj[key] = value`)
    SetItem { object: ValueID, key: ValueID, value: ValueID },

    // ===== Attribute Operations =====
    /// Get object attribute
    GetAttr { object: ValueID, attr: String, ty: MIRType },
    /// Set object attribute
    SetAttr { object: ValueID, attr: String, value: ValueID },

    // ===== Function Calls =====
    /// Call function
    Call { callee: ValueID, args: Vec<ValueID>, ty: MIRType },
    /// Call method
    MethodCall { object: ValueID, method: String, args: Vec<ValueID>, ty: MIRType },

    // ===== Constants and Loads =====
    /// Constant value
    Const(MIRConst),
    /// Load from local variable
    Load { local: LocalID, ty: MIRType },
    /// Load from global variable
    LoadGlobal { name: String, ty: MIRType },
    /// Store to local variable
    Store { local: LocalID, value: ValueID },
    /// Store to global variable
    StoreGlobal { name: String, value: ValueID },

    // ===== Arithmetic Operations =====
    /// Binary operation
    BinOp { op: BinOpKind, lhs: ValueID, rhs: ValueID, ty: MIRType },
    /// Unary operation
    UnOp { op: UnOpKind, operand: ValueID, ty: MIRType },

    // ===== Type Operations =====
    /// Cast to type (runtime check)
    Cast { value: ValueID, target_ty: MIRType },
    /// Check if object is instance of type
    InstanceOf { object: ValueID, type_id: TypeID },

    // ===== Class Operations =====
    /// Get class attribute (for static methods/fields)
    GetClassAttr { type_id: TypeID, attr: String, ty: MIRType },

    // ===== Closure Operations =====
    /// Create closure object
    CreateClosure { function_name: String, captures: Vec<ValueID>, ty: MIRType },
    /// Get captured variable from closure environment
    GetCapture { closure: ValueID, index: usize, ty: MIRType },
    /// Set captured variable in closure environment (for mutable captures)
    SetCapture { closure: ValueID, index: usize, value: ValueID },

    // ===== Phi Node (SSA) =====
    /// Phi node for SSA form
    Phi { incoming: Vec<(BasicBlockID, ValueID)>, ty: MIRType },
}

/// Terminator instruction (ends a basic block)
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional branch
    Branch(BasicBlockID),
    /// Conditional branch
    CondBranch { condition: ValueID, then_block: BasicBlockID, else_block: BasicBlockID },
    /// Invoke function with exception handling
    Invoke {
        callee: ValueID,
        args: Vec<ValueID>,
        normal: BasicBlockID,
        unwind: BasicBlockID,
        ty: MIRType,
    },
    /// Raise exception
    Raise(ValueID),
    /// Return from function
    Return(Option<ValueID>),
    /// Unreachable code
    Unreachable,
}

/// Binary operation kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    // Arithmetic
    Add,
    Div,
    FloorDiv,
    Mod,
    Mul,
    Pow,
    Sub,

    // Comparison
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
    Ne,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operation kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOpKind {
    /// Bitwise not (~x)
    BitNot,
    /// Negation (-x)
    Neg,
    /// Logical not (not x)
    Not,
}

/// Constant value
#[derive(Debug, Clone)]
pub enum MIRConst {
    Bool(bool),
    Float(f64),
    Int(i64),
    None,
    Str(String),
}
