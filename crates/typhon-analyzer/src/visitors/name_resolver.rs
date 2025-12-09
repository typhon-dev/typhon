//! Name resolution visitor for resolving identifier references.
//!
//! This visitor performs the second pass of semantic analysis by:
//! - Resolving all identifier references to their symbol definitions
//! - Performing scope chain lookup to find definitions
//! - Tracking variable captures for closures
//! - Resolving type annotations to Type instances
//! - Detecting undefined name errors
//! - Marking symbols as used when referenced

use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    ASTNode,
    AssignmentStmt,
    AttributeExpr,
    BasicIdent,
    CallExpr,
    CallableType,
    ClassDecl,
    ForStmt,
    FunctionDecl,
    GenericType,
    GlobalStmt,
    LambdaExpr,
    NodeID,
    NodeKind,
    NonlocalStmt,
    ReturnStmt,
    SubscriptionExpr,
    UnionType,
    VariableDecl,
    VariableExpr,
    WithStmt,
};
use typhon_ast::visitor::{MutVisitor, VisitorResult};

use crate::error::SemanticError;
use crate::symbol::{ScopeID, ScopeKind, SymbolTable};
use crate::types::{Type, TypeEnvironment};

/// Visitor that resolves name references and performs closure analysis.
///
/// This visitor implements the second phase of semantic analysis by:
/// - Resolving identifier references to their definitions
/// - Tracking variable captures for closures
/// - Resolving type annotations
/// - Detecting undefined names
#[derive(Debug)]
pub struct NameResolverVisitor<'ast> {
    /// Reference to the AST being analyzed
    ast: &'ast AST,
    /// The symbol table
    symbol_table: &'ast mut SymbolTable,
    /// The type environment for storing resolved types
    type_env: &'ast mut TypeEnvironment,
    /// Collected errors during traversal
    errors: Vec<SemanticError>,
    /// Current function scope (for closure analysis)
    current_function: Option<ScopeID>,
}

impl<'ast> NameResolverVisitor<'ast> {
    /// Creates a new name resolver visitor.
    pub const fn new(
        ast: &'ast AST,
        symbol_table: &'ast mut SymbolTable,
        type_env: &'ast mut TypeEnvironment,
    ) -> Self {
        Self { ast, symbol_table, type_env, errors: Vec::new(), current_function: None }
    }

    /// Resolves names in a module, returning any errors found.
    ///
    /// ## Errors
    ///
    /// Returns collected semantic errors if any were found during resolution.
    pub fn resolve(mut self, module_id: NodeID) -> Result<(), Vec<SemanticError>> {
        // Visit the module to resolve all names
        drop(self.visit_module(module_id));

        // Return errors if any were collected
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(())
    }

    /// Resolves a name reference to its symbol definition.
    ///
    /// This performs scope chain lookup following Python's LEGB rule:
    /// Local -> Enclosing -> Global -> Built-in
    fn resolve_name(&mut self, name: &str, node_id: NodeID) -> Result<(), SemanticError> {
        // Look up the symbol in the scope chain
        if let Some(symbol) = self.symbol_table.lookup_in_scope_chain(name) {
            let symbol_scope = symbol.scope_id;
            let symbol_name = symbol.name.clone();

            // Add reference to the symbol
            let _ = self.modify_symbol(&symbol_name, |sym| {
                sym.add_reference(node_id);
                sym.mark_used();
            });

            // Check if this is a closure capture
            self.check_closure_capture(symbol_scope, &symbol_name);

            Ok(())
        } else {
            // Symbol not found - undefined name error
            let span = self
                .ast
                .get_node(node_id)
                .map_or_else(|| typhon_source::types::Span::new(0, 0), |n| n.span);

            Err(SemanticError::UndefinedName { name: name.to_string(), span })
        }
    }

    /// Checks if a variable reference is a closure capture.
    ///
    /// A variable is captured if:
    /// 1. It's defined in an outer function scope (not module/class)
    /// 2. It's referenced from a nested function scope
    fn check_closure_capture(&mut self, symbol_scope: ScopeID, symbol_name: &str) {
        // Get current scope from the scope stack
        let Some(current_scope_id) = self.symbol_table.current_scope() else {
            return;
        };

        // If we're in a function and the symbol is from a different scope
        if let Some(current_func) = self.current_function
            && symbol_scope != current_scope_id
            && symbol_scope != current_func
        {
            // Check if the symbol's scope is a function scope
            if let Some(symbol_scope_ref) = self.symbol_table.get_scope(symbol_scope)
                && matches!(symbol_scope_ref.kind, ScopeKind::Function | ScopeKind::Lambda)
            {
                // This is a closure capture!
                let _ = self.modify_symbol(symbol_name, |sym| {
                    sym.add_capture(current_func);
                });
            }
        }
    }

    /// Looks up a symbol by name and modifies it.
    ///
    /// This is a helper method that finds a symbol in the scope chain and applies
    /// a modification function to it. This avoids borrow checker issues with
    /// returning mutable references from different scopes.
    fn modify_symbol<F>(&mut self, name: &str, f: F) -> bool
    where F: FnOnce(&mut crate::symbol::Symbol) {
        // Get the current scope
        let Some(scope_id) = self.symbol_table.current_scope() else {
            return false;
        };

        // Collect the scope chain first
        let mut scope_chain = Vec::new();
        let mut current = Some(scope_id);
        while let Some(sid) = current {
            scope_chain.push(sid);
            if let Some(scope) = self.symbol_table.get_scope(sid) {
                current = scope.parent;
            } else {
                break;
            }
        }

        // Try to find and modify the symbol
        for sid in scope_chain {
            if let Some(scope) = self.symbol_table.get_scope_mut(sid)
                && let Some(symbol) = scope.get_symbol_mut(name)
            {
                f(symbol);

                return true;
            }
        }

        false
    }

    /// Resolves a type annotation node to a Type.
    ///
    /// This converts AST type nodes to internal Type enum values.
    fn resolve_type_annotation(&mut self, type_node_id: NodeID) -> Result<Type, SemanticError> {
        let node = self.ast.get_node(type_node_id).ok_or_else(|| SemanticError::InvalidScope {
            message: format!("Type node {type_node_id} not found"),
            span: typhon_source::types::Span::new(0, 0),
        })?;

        // Check the node kind
        match node.kind {
            NodeKind::Type => {
                // Try different type node types
                if let Ok(union_type) = self.ast.get_as::<UnionType>(type_node_id) {
                    // Union type - resolve all member types
                    let mut types = Vec::new();
                    for &type_id in &union_type.type_ids {
                        types.push(self.resolve_type_annotation(type_id)?);
                    }

                    return Ok(Type::Union(types));
                }

                if let Ok(generic_type) = self.ast.get_as::<GenericType>(type_node_id) {
                    // Generic type like List[int], Dict[str, int]
                    let base_type = self.resolve_type_annotation(generic_type.base_id)?;

                    // Handle common generic types
                    if let Type::Class { name, .. } = base_type {
                        match name.as_str() {
                            "list" | "List" => {
                                if generic_type.arg_ids.len() == 1 {
                                    let elem_type =
                                        self.resolve_type_annotation(generic_type.arg_ids[0])?;

                                    return Ok(Type::List(Box::new(elem_type)));
                                }
                            }
                            "dict" | "Dict" => {
                                if generic_type.arg_ids.len() == 2 {
                                    let key_type =
                                        self.resolve_type_annotation(generic_type.arg_ids[0])?;
                                    let val_type =
                                        self.resolve_type_annotation(generic_type.arg_ids[1])?;

                                    return Ok(Type::Dict(Box::new(key_type), Box::new(val_type)));
                                }
                            }
                            "set" | "Set" => {
                                if generic_type.arg_ids.len() == 1 {
                                    let elem_type =
                                        self.resolve_type_annotation(generic_type.arg_ids[0])?;

                                    return Ok(Type::Set(Box::new(elem_type)));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let Ok(callable_type) = self.ast.get_as::<CallableType>(type_node_id) {
                    // Function type
                    let mut param_types = Vec::new();
                    for &param_id in &callable_type.param_ids {
                        param_types.push(self.resolve_type_annotation(param_id)?);
                    }
                    let return_type = self.resolve_type_annotation(callable_type.return_type_id)?;

                    return Ok(Type::Function {
                        params: param_types,
                        return_type: Box::new(return_type),
                    });
                }

                // Fall through to identifier check
                Ok(Type::Any)
            }
            NodeKind::Identifier => {
                // Simple type name like "int", "str", etc.
                if let Ok(ident) = self.ast.get_as::<BasicIdent>(type_node_id) {
                    return Ok(Self::type_name_to_type(&ident.name));
                }
                Ok(Type::Any)
            }
            NodeKind::Expression => {
                // Handle variable expressions for simple type names (e.g., "str", "int")
                // or base types in subscriptions (e.g., "list" in "list[int]")
                if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(type_node_id) {
                    return Ok(Self::type_name_to_type(&var_expr.name));
                }

                // Type annotations can be subscription expressions like list[int]
                if let Ok(subscript) = self.ast.get_as::<SubscriptionExpr>(type_node_id) {
                    // Resolve the base (e.g., "list")
                    let base_type = self.resolve_type_annotation(subscript.value)?;

                    // Handle common generic types
                    if let Type::Class { name, .. } = base_type {
                        match name.as_str() {
                            "list" | "List" => {
                                let elem_type = self.resolve_type_annotation(subscript.index)?;

                                return Ok(Type::List(Box::new(elem_type)));
                            }
                            "dict" | "Dict" => {
                                // For dict, slice should be a tuple of two types
                                // For now, just handle simple case
                                // TODO: Handle tuple slice for Dict[K, V]
                                return Ok(Type::Any);
                            }
                            "set" | "Set" => {
                                let elem_type = self.resolve_type_annotation(subscript.index)?;

                                return Ok(Type::Set(Box::new(elem_type)));
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Type::Any)
            }
            _ => Ok(Type::Any),
        }
    }

    /// Converts a type name string to a Type enum value.
    fn type_name_to_type(name: &str) -> Type {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "str" => Type::Str,
            "bool" => Type::Bool,
            "bytes" => Type::Bytes,
            "None" => Type::None,
            "Any" => Type::Any,
            "Never" => Type::Never,
            other => Type::Class { name: other.to_string(), type_params: Vec::new() },
        }
    }
}

impl MutVisitor<()> for NameResolverVisitor<'_> {
    fn visit(&mut self, node_id: NodeID) -> Option<()> {
        // Get node and dispatch based on kind
        let node = self.ast.get_node(node_id)?;

        match node.kind {
            NodeKind::Module => self.visit_module(node_id).ok(),
            NodeKind::Expression => {
                // Try specific expression types
                if self.visit_variable_expr(node_id).is_ok()
                    || self.visit_attribute_expr(node_id).is_ok()
                    || self.visit_call_expr(node_id).is_ok()
                    || self.visit_lambda_expr(node_id).is_ok()
                {
                    return Some(());
                }

                // For other expressions, visit children
                for child_id in node.data.children() {
                    let _ = self.visit(child_id);
                }
                Some(())
            }
            NodeKind::Statement => {
                // Try specific statement types
                if self.visit_assignment_stmt(node_id).is_ok()
                    || self.visit_for_stmt(node_id).is_ok()
                    || self.visit_with_stmt(node_id).is_ok()
                    || self.visit_return_stmt(node_id).is_ok()
                    || self.visit_global_stmt(node_id).is_ok()
                    || self.visit_nonlocal_stmt(node_id).is_ok()
                    || self.visit_import_stmt(node_id).is_ok()
                    || self.visit_from_import_stmt(node_id).is_ok()
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
                    || self.visit_class_decl(node_id).is_ok()
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

    fn visit_module(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let module = self.ast.get_as::<typhon_ast::nodes::Module>(node_id)?;

        // Process all statements
        for &stmt_id in &module.statements {
            let _ = self.visit(stmt_id);
        }

        Ok(())
    }

    fn visit_variable_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let var_expr = self.ast.get_as::<VariableExpr>(node_id)?;

        // Resolve the variable name
        if let Err(err) = self.resolve_name(&var_expr.name, node_id) {
            self.errors.push(err);
        }

        Ok(())
    }

    fn visit_attribute_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let attr_expr = self.ast.get_as::<AttributeExpr>(node_id)?;

        // Resolve the base expression first
        let _ = self.visit(attr_expr.value);

        // Note: Attribute names themselves don't need resolution at this stage
        // They'll be resolved during type checking

        Ok(())
    }

    fn visit_call_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let call_expr = self.ast.get_as::<CallExpr>(node_id)?;

        // Resolve the function expression
        let _ = self.visit(call_expr.func);

        // Resolve all arguments
        for &arg_id in &call_expr.args {
            let _ = self.visit(arg_id);
        }

        // Resolve all keyword arguments
        for &kwarg_id in &call_expr.keywords {
            let _ = self.visit(kwarg_id);
        }

        Ok(())
    }

    fn visit_lambda_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let lambda = self.ast.get_as::<LambdaExpr>(node_id)?;

        // Get the lambda's scope
        let lambda_scope_id = self.symbol_table.get_node_scope(node_id);

        // Save previous function context
        let prev_function = self.current_function;

        // Set current function to this lambda
        self.current_function = lambda_scope_id;

        // Enter lambda scope
        if let Some(scope_id) = lambda_scope_id {
            self.symbol_table.enter_scope(scope_id);
        }

        // Parameters were already collected, just visit the body
        let _ = self.visit(lambda.body);

        // Exit lambda scope
        if lambda_scope_id.is_some() {
            let _ = self.symbol_table.exit_scope();
        }

        // Restore previous function context
        self.current_function = prev_function;

        Ok(())
    }

    fn visit_function_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let func = self.ast.get_as::<FunctionDecl>(node_id)?;

        // Get the function's scope
        let func_scope_id = self.symbol_table.get_node_scope(node_id);

        // Save previous function context
        let prev_function = self.current_function;

        // Set current function to this function
        self.current_function = func_scope_id;

        // Enter function scope
        if let Some(scope_id) = func_scope_id {
            self.symbol_table.enter_scope(scope_id);
        }

        // Resolve return type annotation if present
        if let Some(return_type_id) = func.return_type
            && let Ok(ty) = self.resolve_type_annotation(return_type_id)
        {
            let type_id = self.type_env.add_type(ty);
            self.type_env.set_node_type(node_id, type_id);
        }

        // Resolve parameter type annotations
        for &param_id in &func.parameters {
            if let Ok(param) = self.ast.get_as::<typhon_ast::nodes::ParameterIdent>(param_id)
                && let Some(type_ann_id) = param.type_annotation
                && let Ok(ty) = self.resolve_type_annotation(type_ann_id)
            {
                let type_id = self.type_env.add_type(ty);
                self.type_env.set_node_type(param_id, type_id);
            }
        }

        // Visit function body
        for &stmt_id in &func.body {
            let _ = self.visit(stmt_id);
        }

        // Exit function scope
        if func_scope_id.is_some() {
            let _ = self.symbol_table.exit_scope();
        }

        // Restore previous function context
        self.current_function = prev_function;

        Ok(())
    }

    fn visit_class_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let class = self.ast.get_as::<ClassDecl>(node_id)?;

        // Resolve base classes
        for &base_id in &class.bases {
            let _ = self.visit(base_id);
        }

        // Enter class scope
        if let Some(scope_id) = self.symbol_table.get_node_scope(node_id) {
            self.symbol_table.enter_scope(scope_id);

            // Visit class body
            for &stmt_id in &class.body {
                let _ = self.visit(stmt_id);
            }

            let _ = self.symbol_table.exit_scope();
        }

        Ok(())
    }

    fn visit_variable_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let var_decl = self.ast.get_as::<VariableDecl>(node_id)?;

        // Resolve type annotation if present
        if let Some(type_ann_id) = var_decl.type_annotation {
            match self.resolve_type_annotation(type_ann_id) {
                Ok(ty) => {
                    let type_id = self.type_env.add_type(ty);
                    self.type_env.set_node_type(node_id, type_id);
                }
                Err(_e) => {
                    // Ignore type resolution errors for now
                }
            }
        }

        // Resolve value expression if present
        if let Some(value_id) = var_decl.value {
            let _ = self.visit(value_id);
        }

        Ok(())
    }

    fn visit_assignment_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let assign = self.ast.get_as::<AssignmentStmt>(node_id)?;

        // Visit the value expression
        let _ = self.visit(assign.value);

        // Visit the target (it might be a complex pattern)
        let _ = self.visit(assign.target);

        Ok(())
    }

    fn visit_for_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let for_stmt = self.ast.get_as::<ForStmt>(node_id)?;

        // Visit the iterable expression
        let _ = self.visit(for_stmt.iter);

        // Visit the target variable (it should already be defined)
        let _ = self.visit(for_stmt.target);

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

    fn visit_with_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let with_stmt = self.ast.get_as::<WithStmt>(node_id)?;

        // Visit context manager expressions
        for (context_expr, _opt_var) in &with_stmt.items {
            let _ = self.visit(*context_expr);
        }

        // Visit body
        for &stmt_id in &with_stmt.body {
            let _ = self.visit(stmt_id);
        }

        Ok(())
    }

    fn visit_return_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let return_stmt = self.ast.get_as::<ReturnStmt>(node_id)?;

        // Visit return value if present
        if let Some(value_id) = return_stmt.value {
            let _ = self.visit(value_id);
        }

        Ok(())
    }

    fn visit_global_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let global_stmt = self.ast.get_as::<GlobalStmt>(node_id)?;

        // Mark each name as global in the symbol table
        for &name_id in &global_stmt.names {
            // Get the name string from the identifier node
            if let Ok(ident) = self.ast.get_as::<BasicIdent>(name_id) {
                let _ = self.modify_symbol(&ident.name, |sym| {
                    sym.set_global(true);
                });
            }
        }

        Ok(())
    }

    fn visit_nonlocal_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let nonlocal_stmt = self.ast.get_as::<NonlocalStmt>(node_id)?;

        // Mark each name as nonlocal in the symbol table
        for &name_id in &nonlocal_stmt.names {
            // Get the name string from the identifier node
            if let Ok(ident) = self.ast.get_as::<BasicIdent>(name_id) {
                let _ = self.modify_symbol(&ident.name, |sym| {
                    sym.set_nonlocal(true);
                });
            }
        }

        Ok(())
    }

    fn visit_import_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        // Import statements don't need name resolution
        // The imported names were already collected
        Ok(())
    }

    fn visit_from_import_stmt(&mut self, _node_id: NodeID) -> VisitorResult<()> {
        // From-import statements don't need name resolution
        // The imported names were already collected
        Ok(())
    }
}
