//! Type checking and inference visitor.
//!
//! This visitor performs the third pass of semantic analysis by:
//! - Inferring types for expressions bottom-up
//! - Checking type compatibility for statements and declarations
//! - Generating and solving type constraints
//! - Validating operators, function calls, and assignments
//! - Detecting type errors

use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    ASTNode,
    AssignmentStmt,
    AttributeExpr,
    BinaryOpExpr,
    BinaryOpKind,
    CallExpr,
    ForStmt,
    FunctionDecl,
    LiteralExpr,
    LiteralValue,
    NodeID,
    NodeKind,
    ReturnStmt,
    UnaryOpExpr,
    UnaryOpKind,
    VariableDecl,
    VariableExpr,
};
use typhon_ast::visitor::{MutVisitor, VisitorResult};

use crate::error::SemanticError;
use crate::symbol::SymbolTable;
use crate::types::{ConstraintSolver, Type, TypeEnvironment, TypeID};

/// Visitor that performs type checking and inference.
///
/// This visitor implements the third phase of semantic analysis by:
/// - Inferring expression types bottom-up
/// - Checking statement and declaration types
/// - Generating constraints for type variables
/// - Validating operators, calls, and assignments
#[derive(Debug)]
pub struct TypeCheckerVisitor<'ast> {
    /// Reference to the AST being analyzed
    ast: &'ast AST,
    /// The type environment for storing inferred types
    type_env: &'ast mut TypeEnvironment,
    /// The symbol table for looking up variable declarations
    symbol_table: &'ast mut SymbolTable,
    /// Constraint solver for type inference
    constraint_solver: ConstraintSolver,
    /// Collected errors during traversal
    errors: Vec<SemanticError>,
    /// Current function return type (for checking return statements)
    current_function_return_type: Option<TypeID>,
}

impl<'ast> TypeCheckerVisitor<'ast> {
    /// Creates a new type checker visitor.
    #[must_use]
    pub fn new(
        ast: &'ast AST,
        type_env: &'ast mut TypeEnvironment,
        symbol_table: &'ast mut SymbolTable,
    ) -> Self {
        Self {
            ast,
            type_env,
            symbol_table,
            constraint_solver: ConstraintSolver::new(),
            errors: Vec::new(),
            current_function_return_type: None,
        }
    }

    /// Checks types in a module, returning any errors found.
    ///
    /// ## Errors
    ///
    /// Returns collected semantic errors if any were found during type checking.
    pub fn check(mut self, module_id: NodeID) -> Result<(), Vec<SemanticError>> {
        // Visit the module to check all types
        drop(self.visit_module(module_id));

        // Solve collected constraints
        if let Err(constraint_errors) = self.constraint_solver.solve(self.type_env) {
            self.errors.extend(constraint_errors);
        }

        // Return errors if any were collected
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(())
    }

    /// Checks that an assignment is type-correct.
    fn check_assignment(
        &mut self,
        target_id: NodeID,
        value_id: NodeID,
    ) -> Result<(), SemanticError> {
        // Infer value type
        let value_type_id = self.infer_expr_type(value_id)?;

        // Check if target has a declared type
        if let Some(target_type_id) = self.type_env.get_node_type(target_id) {
            let target_type = self.type_env.get_type(target_type_id).cloned().unwrap_or(Type::Any);
            let value_type = self.type_env.get_type(value_type_id).cloned().unwrap_or(Type::Any);

            // Check compatibility
            if !value_type.is_compatible_with(&target_type) {
                let span = self
                    .ast
                    .get_node(target_id)
                    .map_or_else(|| typhon_source::types::Span::new(0, 0), |n| n.span);

                return Err(SemanticError::TypeMismatch {
                    expected: Box::new(target_type),
                    found: Box::new(value_type),
                    span,
                });
            }
        } else {
            // No declared type, infer from value
            self.type_env.set_node_type(target_id, value_type_id);
        }

        Ok(())
    }

    /// Checks a return statement against the current function's return type.
    fn check_return(&mut self, return_value_id: Option<NodeID>) -> Result<(), SemanticError> {
        // Get expected return type
        let Some(expected_type_id) = self.current_function_return_type else {
            // Not in a function, can't validate
            return Ok(());
        };

        let expected_type = self.type_env.get_type(expected_type_id).cloned().unwrap_or(Type::Any);

        // Infer actual return type (even if no return type annotation, we need to
        // validate the expression for attribute/method errors)
        let actual_type = if let Some(value_id) = return_value_id {
            let type_id = self.infer_expr_type(value_id)?;
            self.type_env.get_type(type_id).cloned().unwrap_or(Type::Any)
        } else {
            Type::None
        };

        // If expected type is None (no return type annotation), allow any return
        // but we've already validated the expression above
        if expected_type == Type::None {
            return Ok(());
        }

        // Check compatibility
        if !actual_type.is_compatible_with(&expected_type) {
            let span = if let Some(value_id) = return_value_id {
                self.ast
                    .get_node(value_id)
                    .map_or_else(|| typhon_source::types::Span::new(0, 0), |n| n.span)
            } else {
                typhon_source::types::Span::new(0, 0)
            };

            return Err(SemanticError::ReturnTypeMismatch {
                expected: Box::new(expected_type),
                found: Box::new(actual_type),
                span,
            });
        }

        Ok(())
    }

    /// Infers the type of an attribute access.
    fn infer_attribute_type(&mut self, attr: &AttributeExpr) -> Result<TypeID, SemanticError> {
        // Infer base type
        let base_type_id = self.infer_expr_type(attr.value)?;
        let base_type = self.type_env.get_type(base_type_id).cloned().unwrap_or(Type::Any);

        // Look up attribute type (try attribute first, then method)
        let attr_type =
            base_type.get_attribute(&attr.name).or_else(|| base_type.get_method(&attr.name));

        // If neither attribute nor method exists and type is not Any, error
        if attr_type.is_none() && !matches!(base_type, Type::Any) {
            return Err(SemanticError::AttributeError {
                type_name: format!("{base_type}"),
                attribute: attr.name.clone(),
                span: attr.span,
            });
        }

        Ok(self.type_env.add_type(attr_type.unwrap_or(Type::Any)))
    }

    /// Infers the type of a binary operation.
    fn infer_binary_op_type(&mut self, binary_op: &BinaryOpExpr) -> Result<TypeID, SemanticError> {
        // Infer operand types
        let left_type_id = self.infer_expr_type(binary_op.left)?;
        let right_type_id = self.infer_expr_type(binary_op.right)?;

        let left_type = self.type_env.get_type(left_type_id).cloned().unwrap_or(Type::Any);
        let right_type = self.type_env.get_type(right_type_id).cloned().unwrap_or(Type::Any);

        // Determine result type based on operator and operand types
        let result_type = match binary_op.op {
            // Arithmetic operators
            BinaryOpKind::Add
            | BinaryOpKind::Sub
            | BinaryOpKind::Mul
            | BinaryOpKind::Div
            | BinaryOpKind::FloorDiv
            | BinaryOpKind::Mod
            | BinaryOpKind::Pow => {
                Self::infer_arithmetic_type(&left_type, &right_type, binary_op.op)?
            }

            // Comparison and logical operators always return Bool
            BinaryOpKind::And
            | BinaryOpKind::Eq
            | BinaryOpKind::NotEq
            | BinaryOpKind::Lt
            | BinaryOpKind::LtEq
            | BinaryOpKind::Gt
            | BinaryOpKind::GtEq
            | BinaryOpKind::Is
            | BinaryOpKind::IsNot
            | BinaryOpKind::In
            | BinaryOpKind::NotIn
            | BinaryOpKind::Or => Type::Bool,

            // Bitwise operators return Int
            BinaryOpKind::BitAnd
            | BinaryOpKind::BitOr
            | BinaryOpKind::BitXor
            | BinaryOpKind::LShift
            | BinaryOpKind::RShift => {
                if !matches!(left_type, Type::Int) || !matches!(right_type, Type::Int) {
                    return Err(SemanticError::InvalidOperator {
                        operator: format!("{:?}", binary_op.op),
                        left_type: Box::new(left_type),
                        right_type: Box::new(right_type),
                        span: binary_op.span,
                    });
                }
                Type::Int
            }

            // Matrix multiplication
            BinaryOpKind::MatMul => {
                // For now, default to Any for matrix operations
                Type::Any
            }
        };

        Ok(self.type_env.add_type(result_type))
    }

    /// Infers the type of a function call.
    fn infer_call_type(&mut self, call: &CallExpr) -> Result<TypeID, SemanticError> {
        // Check if this is a method call (AttributeExpr)
        if let Ok(attr_expr) = self.ast.get_as::<AttributeExpr>(call.func) {
            // Infer base type
            let base_type_id = self.infer_expr_type(attr_expr.value)?;
            let base_type = self.type_env.get_type(base_type_id).cloned().unwrap_or(Type::Any);

            // Check if method exists
            let method_exists = base_type.get_method(&attr_expr.name).is_some();

            if !method_exists && !matches!(base_type, Type::Any) {
                return Err(SemanticError::AttributeError {
                    type_name: format!("{base_type}"),
                    attribute: attr_expr.name.clone(),
                    span: attr_expr.span,
                });
            }
        }

        // Infer function type
        let func_type_id = self.infer_expr_type(call.func)?;
        let func_type = self.type_env.get_type(func_type_id).cloned().unwrap_or(Type::Any);

        // Infer argument types
        for &arg_id in &call.args {
            drop(self.infer_expr_type(arg_id));
        }

        // Extract return type if function type is known
        let return_type = match func_type {
            Type::Function { return_type, .. } => *return_type,
            // TODO: Report error for non-callable types (except Any)
            Type::Any
            | Type::Bool
            | Type::Bytes
            | Type::Class { .. }
            | Type::Dict(_, _)
            | Type::Float
            | Type::Int
            | Type::List(_)
            | Type::Never
            | Type::None
            | Type::Optional(_)
            | Type::Set(_)
            | Type::Str
            | Type::Tuple(_)
            | Type::TypeVar(_)
            | Type::Union(_) => Type::Any,
        };

        Ok(self.type_env.add_type(return_type))
    }

    /// Infers the type of an expression, returning its type ID.
    fn infer_expr_type(&mut self, expr_id: NodeID) -> Result<TypeID, SemanticError> {
        // Check if type is already inferred
        if let Some(type_id) = self.type_env.get_node_type(expr_id) {
            return Ok(type_id);
        }

        // Get the node
        let node = self.ast.get_node(expr_id).ok_or_else(|| SemanticError::InvalidScope {
            message: format!("Expression node {expr_id} not found"),
            span: typhon_source::types::Span::new(0, 0),
        })?;

        // Infer type based on expression kind
        let type_id = match node.kind {
            NodeKind::Expression => {
                // Try specific expression types
                if let Ok(literal) = self.ast.get_as::<LiteralExpr>(expr_id) {
                    self.infer_literal_type(literal)?
                } else if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(expr_id) {
                    self.infer_variable_type(var_expr)?
                } else if let Ok(binary_op) = self.ast.get_as::<BinaryOpExpr>(expr_id) {
                    self.infer_binary_op_type(binary_op)?
                } else if let Ok(unary_op) = self.ast.get_as::<UnaryOpExpr>(expr_id) {
                    self.infer_unary_op_type(unary_op)?
                } else if let Ok(call) = self.ast.get_as::<CallExpr>(expr_id) {
                    self.infer_call_type(call)?
                } else if let Ok(attr) = self.ast.get_as::<AttributeExpr>(expr_id) {
                    self.infer_attribute_type(attr)?
                } else {
                    // Default to Any for unknown expression types
                    self.type_env.add_type(Type::Any)
                }
            }
            _ => {
                // Non-expression nodes default to Any
                self.type_env.add_type(Type::Any)
            }
        };

        // Store the inferred type
        self.type_env.set_node_type(expr_id, type_id);

        Ok(type_id)
    }

    /// Infers the type of a literal expression.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`. The `Result` type is maintained for consistency
    /// with other type inference methods.
    #[allow(clippy::unnecessary_wraps)]
    fn infer_literal_type(&mut self, literal: &LiteralExpr) -> Result<TypeID, SemanticError> {
        let ty = match &literal.kind {
            LiteralValue::Bool(_) => Type::Bool,
            LiteralValue::Bytes(_) => Type::Bytes,
            LiteralValue::Ellipsis => Type::Any, // Ellipsis is special
            LiteralValue::Float(_) => Type::Float,
            LiteralValue::Int(_) => Type::Int,
            LiteralValue::None => Type::None,
            LiteralValue::String(_) => Type::Str,
        };

        Ok(self.type_env.add_type(ty))
    }

    /// Infers the type of a unary operation.
    fn infer_unary_op_type(&mut self, unary_op: &UnaryOpExpr) -> Result<TypeID, SemanticError> {
        // Infer operand type
        let operand_type_id = self.infer_expr_type(unary_op.operand)?;
        let operand_type = self.type_env.get_type(operand_type_id).cloned().unwrap_or(Type::Any);

        // Determine result type based on operator
        let result_type = match unary_op.op {
            UnaryOpKind::Pos | UnaryOpKind::Neg => {
                // +x or -x: must be numeric, returns same type
                if operand_type.is_numeric() { operand_type } else { Type::Any }
            }
            UnaryOpKind::Not => {
                // not x: always returns bool
                Type::Bool
            }
            UnaryOpKind::BitNot => {
                // ~x: must be Int, returns Int
                if matches!(operand_type, Type::Int) { Type::Int } else { Type::Any }
            }
        };

        Ok(self.type_env.add_type(result_type))
    }

    /// Infers the type of a variable expression by looking it up in the symbol table and type environment.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`. The `Result` type is maintained for future
    /// error reporting of undefined variables.
    #[allow(clippy::unnecessary_wraps)]
    fn infer_variable_type(&mut self, var_expr: &VariableExpr) -> Result<TypeID, SemanticError> {
        // Look up the variable's symbol in the symbol table
        if let Some(symbol) = self.symbol_table.lookup_in_scope_chain(&var_expr.name) {
            // Get the symbol's definition node
            let def_node_id = symbol.definition_node;

            // Look up the type for that declaration node
            if let Some(type_id) = self.type_env.get_node_type(def_node_id) {
                return Ok(type_id);
            }
        }

        // Default to Any if no type information found
        Ok(self.type_env.add_type(Type::Any))
    }

    /// Infers the result type of an arithmetic operation.
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`. The `Result` type is maintained for future
    /// error reporting of invalid type combinations.
    #[allow(clippy::unnecessary_wraps)]
    fn infer_arithmetic_type(
        left: &Type,
        right: &Type,
        op: BinaryOpKind,
    ) -> Result<Type, SemanticError> {
        match (left, right) {
            // Int + Int -> Int
            (Type::Int, Type::Int) => Ok(Type::Int),

            // Float + Float -> Float or Int + Float or Float + Int -> Float
            (Type::Float | Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),

            // String + String -> String (only for Add)
            (Type::Str, Type::Str) if matches!(op, BinaryOpKind::Add) => Ok(Type::Str),

            // List + List -> List (only for Add)
            (Type::List(left_elem), Type::List(right_elem)) if matches!(op, BinaryOpKind::Add) => {
                // Unify element types
                let unified = left_elem.unify(right_elem).unwrap_or(Type::Any);
                Ok(Type::List(Box::new(unified)))
            }

            // Any type propagates through operations (dynamic typing)
            // TODO: Properly validate and report errors for unsupported type combinations
            // For now, be permissive with other combinations
            (
                Type::Any
                | Type::Bool
                | Type::Bytes
                | Type::Class { .. }
                | Type::Dict(_, _)
                | Type::Float
                | Type::Function { .. }
                | Type::Int
                | Type::List(_)
                | Type::Never
                | Type::None
                | Type::Optional(_)
                | Type::Set(_)
                | Type::Str
                | Type::Tuple(_)
                | Type::TypeVar(_)
                | Type::Union(_),
                Type::Any
                | Type::Bool
                | Type::Bytes
                | Type::Class { .. }
                | Type::Dict(_, _)
                | Type::Float
                | Type::Function { .. }
                | Type::Int
                | Type::List(_)
                | Type::Never
                | Type::None
                | Type::Optional(_)
                | Type::Set(_)
                | Type::Str
                | Type::Tuple(_)
                | Type::TypeVar(_)
                | Type::Union(_),
            ) => Ok(Type::Any),
        }
    }
}

impl MutVisitor<()> for TypeCheckerVisitor<'_> {
    fn visit(&mut self, node_id: NodeID) -> Option<()> {
        // Get node and dispatch based on kind
        let node = self.ast.get_node(node_id)?;

        match node.kind {
            NodeKind::Module => self.visit_module(node_id).ok(),
            NodeKind::Expression => {
                // Infer expression type
                if let Err(err) = self.infer_expr_type(node_id) {
                    self.errors.push(err);
                }

                Some(())
            }
            NodeKind::Statement => {
                // Try specific statement types
                if self.visit_assignment_stmt(node_id).is_ok()
                    || self.visit_for_stmt(node_id).is_ok()
                    || self.visit_return_stmt(node_id).is_ok()
                {
                    return Some(());
                }

                // For other statements, visit children
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }

                Some(())
            }
            NodeKind::Declaration => {
                // Try specific declaration types
                if self.visit_function_decl(node_id).is_ok()
                    || self.visit_variable_decl(node_id).is_ok()
                {
                    return Some(());
                }

                // For other declarations, visit children
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }

                Some(())
            }
            _ => {
                // Visit all children for other node types
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }

                Some(())
            }
        }
    }

    fn visit_assignment_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let assign = self.ast.get_as::<AssignmentStmt>(node_id)?;

        // Check the assignment
        if let Err(err) = self.check_assignment(assign.target, assign.value) {
            self.errors.push(err);
        }

        Ok(())
    }

    fn visit_for_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let for_stmt = self.ast.get_as::<ForStmt>(node_id)?;

        // Infer iterable type
        let _iter_type_id = self.infer_expr_type(for_stmt.iter);

        // Visit loop body
        for &stmt_id in &for_stmt.body {
            let _ = self.visit(stmt_id);
        }

        // Visit else body if present
        if let Some(else_body) = &for_stmt.else_body {
            for &stmt_id in else_body {
                let _ = self.visit(stmt_id);
            }
        }

        Ok(())
    }

    fn visit_function_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let func = self.ast.get_as::<FunctionDecl>(node_id)?;

        // Get return type if annotated
        let return_type_id = if let Some(return_type_node) = func.return_type {
            self.type_env.get_node_type(return_type_node)
        } else {
            Some(self.type_env.add_type(Type::None))
        };

        // Save previous return type and set current
        let prev_return_type = self.current_function_return_type;
        self.current_function_return_type = return_type_id;

        // Enter the function's scope so variable lookups work correctly
        if let Some(scope_id) = self.symbol_table.get_node_scope(node_id) {
            self.symbol_table.enter_scope(scope_id);
        }

        // Visit function body
        for &stmt_id in &func.body {
            let _ = self.visit(stmt_id);
        }

        // Exit the function's scope
        if self.symbol_table.get_node_scope(node_id).is_some() {
            let _ = self.symbol_table.exit_scope();
        }

        // Restore previous return type
        self.current_function_return_type = prev_return_type;

        Ok(())
    }

    fn visit_module(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let module = self.ast.get_as::<typhon_ast::nodes::Module>(node_id)?;

        // Process all statements
        for &stmt_id in &module.statements {
            let _ = self.visit(stmt_id);
        }

        Ok(())
    }

    fn visit_return_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let return_stmt = self.ast.get_as::<ReturnStmt>(node_id)?;

        // Check return type
        if let Err(err) = self.check_return(return_stmt.value) {
            self.errors.push(err);
        }

        Ok(())
    }

    fn visit_variable_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let var_decl = self.ast.get_as::<VariableDecl>(node_id)?;

        // If there's a value, check the assignment
        if let Some(value_id) = var_decl.value
            && let Err(err) = self.check_assignment(node_id, value_id)
        {
            self.errors.push(err);
        }

        Ok(())
    }
}
