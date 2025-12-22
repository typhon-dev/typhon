//! Expression lowering
//!
//! This module handles lowering of AST expressions to MIR instructions.
//! Expressions are lowered bottom-up: operands first, then operations.

use typhon_ast::nodes::{
    AnyNode,
    AttributeExpr,
    BinaryOpExpr,
    BinaryOpKind as ASTBinOp,
    CallExpr,
    LiteralExpr,
    LiteralValue,
    NodeID,
    UnaryOpExpr,
    UnaryOpKind as ASTUnOp,
    VariableExpr,
};
use typhon_mir::instr::{BinOpKind, MIRConst, MIRInstr, UnOpKind, ValueID};
use typhon_mir::types::MIRType;

use crate::context::LoweringContext;
use crate::error::LoweringError;

impl LoweringContext<'_> {
    /// Lower an expression node to MIR
    ///
    /// Returns the `ValueID` of the resulting value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The node does not exist in the AST
    /// - The node is not a supported expression type
    /// - Lowering of a sub-expression fails
    pub fn lower_expr(&mut self, node_id: NodeID) -> Result<ValueID, LoweringError> {
        // Get the node from the AST
        let node = self.ast().get_node(node_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Node not found: {node_id}"),
            span: typhon_source::types::Span::new(0, 0),
        })?;

        match &node.data {
            AnyNode::LiteralExpr(lit) => self.lower_literal(lit),
            AnyNode::VariableExpr(var) => self.lower_variable(var),
            AnyNode::BinaryOpExpr(binop) => self.lower_binary_op(binop),
            AnyNode::UnaryOpExpr(unop) => self.lower_unary_op(unop),
            AnyNode::CallExpr(call) => self.lower_call(call),
            AnyNode::AttributeExpr(attr) => self.lower_attribute(attr),
            _ => Err(LoweringError::UnsupportedNode {
                node_kind: format!("{:?}", node.kind),
                span: node.span,
            }),
        }
    }

    /// Lower a literal expression
    fn lower_literal(&mut self, lit: &LiteralExpr) -> Result<ValueID, LoweringError> {
        let mir_const = match &lit.kind {
            LiteralValue::Bool(b) => MIRConst::Bool(*b),
            LiteralValue::Int(i) => MIRConst::Int(*i),
            LiteralValue::Float(f) => MIRConst::Float(*f),
            LiteralValue::String(s) => MIRConst::Str(s.clone()),
            LiteralValue::None => MIRConst::None,
            LiteralValue::Bytes(_) | LiteralValue::Ellipsis => {
                return Err(LoweringError::UnsupportedNode {
                    node_kind: format!("Unsupported literal type: {:?}", lit.kind),
                    span: lit.span,
                });
            }
        };

        Ok(self.emit(MIRInstr::Const(mir_const)))
    }

    /// Lower a variable reference
    ///
    /// Variables are resolved in the following order:
    ///
    /// 1. Local variables (function parameters and local assignments)
    /// 2. Captured variables (for closures)
    /// 3. Global variables (module-level definitions and builtins)
    ///
    /// ## Type Information
    ///
    /// Currently uses [`MIRType::Object`] with `type_id: None` as a default type.
    /// This is appropriate for the MIR lowering phase as detailed type information
    /// will be propagated from the semantic analysis phase in future integration.
    /// The dynamic type will be resolved at runtime.
    ///
    /// ## Global Variables
    ///
    /// Global variable access uses [`MIRInstr::LoadGlobal`] which handles:
    ///
    /// - Module-level function definitions
    /// - Module-level class definitions
    /// - Module-level variable assignments
    /// - Built-in functions and constants
    ///
    /// The runtime will resolve these names in the global namespace at execution time.
    ///
    /// TODO: Enhance global variable support once symbol table integration is completed
    fn lower_variable(&mut self, var: &VariableExpr) -> Result<ValueID, LoweringError> {
        let ty = MIRType::Object { type_id: None };

        // Try to look up the local ID for this variable by name
        if let Ok(local_id) = self.get_local(&var.name) {
            // It's a local variable - emit a load instruction
            Ok(self.emit(MIRInstr::Load { local: local_id, ty }))
        } else {
            // Not a local variable - treat as a global reference
            // This handles function names, class names, and module-level variables
            Ok(self.emit(MIRInstr::LoadGlobal { name: var.name.clone(), ty }))
        }
    }

    /// Lower a binary operation
    ///
    /// ## Type Information
    ///
    /// Uses [`MIRType::Object`] as the result type. In Python's dynamic type system,
    /// binary operations can return different types based on the operands (e.g., `+` can
    /// return int, float, str, list, etc.). The actual result type will be determined at
    /// runtime through Python's operator protocol (`__add__`, `__mul__`, etc.).
    ///
    /// Future integration with the type checker can provide more specific type information
    /// where available (e.g., from type annotations or inference), but the Object type
    /// remains a safe default for the MIR phase.
    ///
    /// TODO: Get actual type from type environment once type checker integration is completed
    fn lower_binary_op(&mut self, binop: &BinaryOpExpr) -> Result<ValueID, LoweringError> {
        // Lower operands first (bottom-up)
        let lhs = self.lower_expr(binop.left)?;
        let rhs = self.lower_expr(binop.right)?;

        // Map AST operator to MIR operator
        let op = map_binary_op(binop.op)?;

        // Use Object type - actual type determined at runtime via operator protocol
        let ty = MIRType::Object { type_id: None };

        // Emit binary operation
        Ok(self.emit(MIRInstr::BinOp { op, lhs, rhs, ty }))
    }

    /// Lower a unary operation
    ///
    /// ## Type Information
    ///
    /// Uses [`MIRType::Object`] as the result type. Unary operations in Python can return
    /// different types based on the operand (e.g., `-x` can return int, float, or any type
    /// with `__neg__` defined). The actual result type is determined at runtime through
    /// Python's unary operator protocol.
    ///
    /// TODO: Get actual type from type environment once type checker integration is completed
    fn lower_unary_op(&mut self, unop: &UnaryOpExpr) -> Result<ValueID, LoweringError> {
        // Lower operand first (bottom-up)
        let operand = self.lower_expr(unop.operand)?;

        // Map AST operator to MIR operator
        let op = map_unary_op(unop.op);

        // Use Object type - actual type determined at runtime via operator protocol
        let ty = MIRType::Object { type_id: None };

        // Emit unary operation
        Ok(self.emit(MIRInstr::UnOp { op, operand, ty }))
    }

    /// Lower a function call
    ///
    /// ## Type Information
    ///
    /// Uses [`MIRType::Object`] for the return type. In Python's dynamic type system,
    /// function return types are determined at runtime. Type annotations can be used
    /// in future integration with the type checker to provide more specific return types.
    ///
    /// TODO: Get actual return type from type environment once type checker integration is completed
    ///
    /// ## Keyword Arguments
    ///
    /// Keyword arguments are not yet supported in the MIR layer. Supporting them requires:
    /// - Dictionary unpacking for `**kwargs`
    /// - Argument name resolution and reordering
    /// - Default parameter value handling
    /// - Integration with function signature metadata
    ///
    /// Currently returns [`LoweringError::UnsupportedNode`] when keyword arguments are present.
    ///
    /// TODO: Handle keyword arguments once dictionary unpacking infrastructure is completed
    ///
    /// ## Future Work
    ///
    /// - Add keyword argument support through argument dictionary construction
    /// - Implement default parameter value handling
    /// - Support *args and **kwargs unpacking
    fn lower_call(&mut self, call: &CallExpr) -> Result<ValueID, LoweringError> {
        // Lower the callee
        let callee = self.lower_expr(call.func)?;

        // Lower positional arguments
        let mut args = Vec::with_capacity(call.args.len());
        for arg_id in &call.args {
            args.push(self.lower_expr(*arg_id)?);
        }

        // Keyword arguments not yet supported - requires dict unpacking infrastructure
        if !call.keywords.is_empty() {
            return Err(LoweringError::UnsupportedNode {
                node_kind: "Keyword arguments (requires dict unpacking)".to_string(),
                span: call.span,
            });
        }

        // Use Object type for return value - actual type determined at runtime
        let ty = MIRType::Object { type_id: None };

        // Emit call instruction
        Ok(self.emit(MIRInstr::Call { callee, args, ty }))
    }

    /// Lower an attribute access
    ///
    /// ## Type Information
    ///
    /// Uses [`MIRType::Object`] for the attribute type. In Python's dynamic type system,
    /// attribute types are determined at runtime through the attribute lookup protocol
    /// (`__getattribute__`, `__getattr__`). The actual type depends on the object's class
    /// definition and cannot always be determined statically.
    ///
    /// Future integration with the type checker can provide more specific attribute types
    /// where class definitions and type annotations are available.
    ///
    /// TODO: Get actual attribute type from type environment once type checker integration is completed
    fn lower_attribute(&mut self, attr: &AttributeExpr) -> Result<ValueID, LoweringError> {
        // Lower the object
        let object = self.lower_expr(attr.value)?;

        // Use Object type - actual attribute type determined at runtime
        let ty = MIRType::Object { type_id: None };

        // Emit GetAttr instruction
        Ok(self.emit(MIRInstr::GetAttr { object, attr: attr.name.clone(), ty }))
    }
}

/// Map AST binary operator to MIR binary operator
fn map_binary_op(op: ASTBinOp) -> Result<BinOpKind, LoweringError> {
    match op {
        // Arithmetic
        ASTBinOp::Add => Ok(BinOpKind::Add),
        ASTBinOp::Sub => Ok(BinOpKind::Sub),
        ASTBinOp::Mul => Ok(BinOpKind::Mul),
        ASTBinOp::Div => Ok(BinOpKind::Div),
        ASTBinOp::FloorDiv => Ok(BinOpKind::FloorDiv),
        ASTBinOp::Mod => Ok(BinOpKind::Mod),
        ASTBinOp::Pow => Ok(BinOpKind::Pow),

        // Comparison
        ASTBinOp::Eq => Ok(BinOpKind::Eq),
        ASTBinOp::NotEq => Ok(BinOpKind::Ne),
        ASTBinOp::Lt => Ok(BinOpKind::Lt),
        ASTBinOp::LtEq => Ok(BinOpKind::Le),
        ASTBinOp::Gt => Ok(BinOpKind::Gt),
        ASTBinOp::GtEq => Ok(BinOpKind::Ge),

        // Logical
        ASTBinOp::And => Ok(BinOpKind::And),
        ASTBinOp::Or => Ok(BinOpKind::Or),

        // Bitwise
        ASTBinOp::BitAnd => Ok(BinOpKind::BitAnd),
        ASTBinOp::BitOr => Ok(BinOpKind::BitOr),
        ASTBinOp::BitXor => Ok(BinOpKind::BitXor),
        ASTBinOp::LShift => Ok(BinOpKind::Shl),
        ASTBinOp::RShift => Ok(BinOpKind::Shr),

        // Unsupported operators
        ASTBinOp::Is | ASTBinOp::IsNot | ASTBinOp::In | ASTBinOp::NotIn | ASTBinOp::MatMul => {
            Err(LoweringError::UnsupportedNode {
                node_kind: format!("Binary operator: {op:?}"),
                span: typhon_source::types::Span::new(0, 0),
            })
        }
    }
}

/// Map AST unary operator to MIR unary operator
const fn map_unary_op(op: ASTUnOp) -> UnOpKind {
    match op {
        ASTUnOp::Not => UnOpKind::Not,
        ASTUnOp::BitNot => UnOpKind::BitNot,
        ASTUnOp::Neg | ASTUnOp::Pos => UnOpKind::Neg, // +x is just x, but we'll use Neg for now
    }
}
