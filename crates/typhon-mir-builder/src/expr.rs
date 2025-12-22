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
use crate::symbol_resolution::NameClassification;

impl LoweringContext<'_, '_> {
    /// Lower an expression node to MIR
    ///
    /// Returns the `ValueID` of the resulting value.
    ///
    /// ## Errors
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
    /// ## Symbol Resolution
    ///
    /// When semantic context is available, uses the symbol table to classify the name
    /// and determine the correct access pattern. Falls back to local/global distinction
    /// when semantic context is unavailable.
    ///
    /// ## Type Information
    ///
    /// Queries type from the semantic analysis context when available. Falls back to
    /// [`MIRType::Object`] with `type_id: None` when type information is unavailable.
    fn lower_variable(&mut self, var: &VariableExpr) -> Result<ValueID, LoweringError> {
        // Query type from semantic context or use default
        let ty = self.query_name_type(&var.name).unwrap_or(MIRType::Object { type_id: None });

        // Classify the name using symbol table if available
        if let Some(classification) = self.classify_name(&var.name) {
            match classification {
                NameClassification::Local => {
                    // Local variable - emit Load instruction
                    let local_id = self.get_local(&var.name)?;

                    Ok(self.emit(MIRInstr::Load { local: local_id, ty }))
                }
                NameClassification::Captured => {
                    // Captured variable - check if we have a local binding first (parameter)
                    // Otherwise treat as nonlocal access (LoadGlobal for now)
                    // Full closure support with GetCapture requires tracking the closure object
                    if let Ok(local_id) = self.get_local(&var.name) {
                        Ok(self.emit(MIRInstr::Load { local: local_id, ty }))
                    } else {
                        // TODO: Treat as nonlocal for now - full closure infrastructure pending
                        Ok(self.emit(MIRInstr::LoadGlobal { name: var.name.clone(), ty }))
                    }
                }
                NameClassification::Global | NameClassification::Builtin => {
                    // Global or builtin - emit LoadGlobal instruction
                    Ok(self.emit(MIRInstr::LoadGlobal { name: var.name.clone(), ty }))
                }
            }
        } else {
            // Fallback when semantic context is unavailable
            // Try local first, then treat as global
            if let Ok(local_id) = self.get_local(&var.name) {
                Ok(self.emit(MIRInstr::Load { local: local_id, ty }))
            } else {
                Ok(self.emit(MIRInstr::LoadGlobal { name: var.name.clone(), ty }))
            }
        }
    }

    /// Lower a binary operation
    ///
    /// ## Type Information
    ///
    /// Queries the type from the semantic analysis context if available. Binary operations
    /// can return different types based on the operands (e.g., `+` can return int, float,
    /// str, list, etc.). When type information is unavailable, falls back to [`MIRType::Object`],
    /// and the actual result type will be determined at runtime through the operator protocol
    /// (`__add__`, `__mul__`, etc.).
    fn lower_binary_op(&mut self, binop: &BinaryOpExpr) -> Result<ValueID, LoweringError> {
        // Lower operands first (bottom-up)
        let lhs = self.lower_expr(binop.left)?;
        let rhs = self.lower_expr(binop.right)?;

        // Map AST operator to MIR operator
        let op = map_binary_op(binop.op)?;

        // Query type from semantic context, or use Object as fallback
        let ty = self.query_expr_type_or_default(binop.id);

        // Emit binary operation
        Ok(self.emit(MIRInstr::BinOp { op, lhs, rhs, ty }))
    }

    /// Lower a unary operation
    ///
    /// ## Type Information
    ///
    /// Queries the type from the semantic analysis context if available. Unary operations
    /// can return different types based on the operand (e.g., `-x` can return int, float,
    /// or any type with `__neg__` defined). When type information is unavailable, falls back
    /// to [`MIRType::Object`], and the actual result type is determined at runtime through
    /// the unary operator protocol.
    fn lower_unary_op(&mut self, unop: &UnaryOpExpr) -> Result<ValueID, LoweringError> {
        // Lower operand first (bottom-up)
        let operand = self.lower_expr(unop.operand)?;

        // Map AST operator to MIR operator
        let op = map_unary_op(unop.op);

        // Query type from semantic context, or use Object as fallback
        let ty = self.query_expr_type_or_default(unop.id);

        // Emit unary operation
        Ok(self.emit(MIRInstr::UnOp { op, operand, ty }))
    }

    /// Lower a function call
    ///
    /// ## Type Information
    ///
    /// Queries the return type from the semantic analysis context if available. Function
    /// return types can be inferred from type annotations and inference. Falls back to
    /// [`MIRType::Object`] when type information is unavailable, with the actual return
    /// type determined at runtime.
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
        // Try to extract the function name if it's a direct call to a named function
        let func_name = if let Some(func_node) = self.ast().get_node(call.func)
            && let AnyNode::VariableExpr(var) = &func_node.data
        {
            Some(var.name.clone())
        } else {
            None
        };

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

        // Query return type from semantic context, or use Object as fallback
        let ty = self.query_expr_type_or_default(call.id);

        // Emit call instruction
        let call_result = self.emit(MIRInstr::Call { callee, args, ty });

        // Track function name for this call result if we have it
        if let Some(name) = func_name {
            self.module_mut().set_value_name(call_result, name);
        }

        Ok(call_result)
    }

    /// Lower an attribute access
    ///
    /// ## Type Information
    ///
    /// Queries the attribute type from the semantic analysis context if available. Attribute
    /// types are determined at runtime through the attribute lookup protocol (`__getattribute__`,
    /// `__getattr__`), but class definitions and type annotations can provide more specific types.
    /// Falls back to [`MIRType::Object`] when type information is unavailable.
    fn lower_attribute(&mut self, attr: &AttributeExpr) -> Result<ValueID, LoweringError> {
        // Lower the object
        let object = self.lower_expr(attr.value)?;

        // Query type from semantic context, or use Object as fallback
        let ty = self.query_expr_type_or_default(attr.id);

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
