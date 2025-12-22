//! Pretty Printing for MIR
//!
//! This module provides human-readable formatting for MIR structures,
//! making it easier to debug and understand generated IR.

use std::fmt;

use crate::block::BasicBlock;
use crate::function::MIRFunction;
use crate::instr::{BinOpKind, LocalID, MIRConst, MIRInstr, Terminator, UnOpKind, ValueID};
use crate::module::MIRModule;

impl fmt::Display for MIRModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "module {}", self.name)?;
        writeln!(f)?;

        for func in &self.functions {
            writeln!(f, "{func}")?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl fmt::Display for MIRFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func @{}(", self.name)?;

        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "%{}: {}", param.name, param.ty)?;
        }

        writeln!(f, ") -> {} {{", self.return_type)?;

        for block in &self.blocks {
            write!(f, "{block}")?;
        }

        writeln!(f, "}}")
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  bb{}:", self.id.0)?;

        for (i, instr) in self.instrs.iter().enumerate() {
            writeln!(f, "    %{i} = {instr}")?;
        }

        writeln!(f, "    {}", self.terminator)?;

        Ok(())
    }
}

impl fmt::Display for MIRInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocObject { type_id, size } => {
                write!(f, "alloc_object type={type_id}, size={size}")
            }
            Self::BinOp { op, lhs, rhs, ty } => {
                write!(f, "binop {op:?}, {lhs}, {rhs} : {ty}")
            }
            Self::Call { callee, args, ty } => {
                write!(f, "call {callee}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{arg}")?;
                }

                write!(f, ") : {ty}")
            }
            Self::Cast { value, target_ty } => {
                write!(f, "cast {value} to {target_ty}")
            }
            Self::Const(c) => write!(f, "const {c}"),
            Self::CreateClosure { function_name, captures, ty } => {
                write!(f, "create_closure @{function_name}, [")?;
                for (i, cap) in captures.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{cap}")?;
                }

                write!(f, "] : {ty}")
            }
            Self::DecRef(val) => write!(f, "decref {val}"),
            Self::GetAttr { object, attr, ty } => {
                write!(f, "getattr {object}.{attr} : {ty}")
            }
            Self::GetCapture { closure, index, ty } => {
                write!(f, "get_capture {closure}, {index} : {ty}")
            }
            Self::GetClassAttr { type_id, attr, ty } => {
                write!(f, "get_class_attr type={type_id}, {attr} : {ty}")
            }
            Self::GetItem { object, key, ty } => {
                write!(f, "getitem {object}[{key}] : {ty}")
            }
            Self::IncRef(val) => write!(f, "incref {val}"),
            Self::InstanceOf { object, type_id } => {
                write!(f, "instanceof {object}, type={type_id}")
            }
            Self::Load { local, ty } => write!(f, "load {local} : {ty}"),
            Self::LoadGlobal { name, ty } => write!(f, "load_global @{name} : {ty}"),
            Self::MethodCall { object, method, args, ty } => {
                write!(f, "method_call {object}.{method}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{arg}")?;
                }

                write!(f, ") : {ty}")
            }
            Self::Phi { incoming, ty } => {
                write!(f, "phi [")?;
                for (i, (block, val)) in incoming.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "bb{}: {val}", block.0)?;
                }

                write!(f, "] : {ty}")
            }
            Self::SetAttr { object, attr, value } => {
                write!(f, "setattr {object}.{attr} = {value}")
            }
            Self::SetCapture { closure, index, value } => {
                write!(f, "set_capture {closure}, {index} = {value}")
            }
            Self::SetItem { object, key, value } => {
                write!(f, "setitem {object}[{key}] = {value}")
            }
            Self::Store { local, value } => write!(f, "store {local}, {value}"),
            Self::StoreGlobal { name, value } => write!(f, "store_global @{name}, {value}"),
            Self::UnOp { op, operand, ty } => {
                write!(f, "unop {op:?}, {operand} : {ty}")
            }
        }
    }
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Branch(target) => write!(f, "br bb{}", target.0),
            Self::CondBranch { condition, then_block, else_block } => {
                write!(f, "cond_br {condition}, bb{}, bb{}", then_block.0, else_block.0)
            }
            Self::Invoke { callee, args, normal, unwind, ty } => {
                write!(f, "invoke {callee}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{arg}")?;
                }

                write!(f, ") normal bb{}, unwind bb{} : {ty}", normal.0, unwind.0)
            }
            Self::Raise(val) => write!(f, "raise {val}"),
            Self::Return(Some(val)) => write!(f, "ret {val}"),
            Self::Return(None) => write!(f, "ret"),
            Self::Unreachable => write!(f, "unreachable"),
        }
    }
}

impl fmt::Display for MIRConst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{b} : Bool"),
            Self::Float(fl) => write!(f, "{fl} : Float"),
            Self::Int(i) => write!(f, "{i} : Int"),
            Self::None => write!(f, "none : None"),
            Self::Str(s) => write!(f, "\"{s}\" : Str"),
        }
    }
}

impl fmt::Display for ValueID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "%{}", self.0) }
}

impl fmt::Display for LocalID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "%{}", self.0) }
}

impl fmt::Display for BinOpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "Add"),
            Self::And => write!(f, "And"),
            Self::BitAnd => write!(f, "BitAnd"),
            Self::BitOr => write!(f, "BitOr"),
            Self::BitXor => write!(f, "BitXor"),
            Self::Div => write!(f, "Div"),
            Self::Eq => write!(f, "Eq"),
            Self::FloorDiv => write!(f, "FloorDiv"),
            Self::Ge => write!(f, "Ge"),
            Self::Gt => write!(f, "Gt"),
            Self::Le => write!(f, "Le"),
            Self::Lt => write!(f, "Lt"),
            Self::Mod => write!(f, "Mod"),
            Self::Mul => write!(f, "Mul"),
            Self::Ne => write!(f, "Ne"),
            Self::Or => write!(f, "Or"),
            Self::Pow => write!(f, "Pow"),
            Self::Shl => write!(f, "Shl"),
            Self::Shr => write!(f, "Shr"),
            Self::Sub => write!(f, "Sub"),
        }
    }
}

impl fmt::Display for UnOpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BitNot => write!(f, "BitNot"),
            Self::Neg => write!(f, "Neg"),
            Self::Not => write!(f, "Not"),
        }
    }
}
