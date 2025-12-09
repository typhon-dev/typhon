//! Symbol collection visitor for building symbol tables.
//!
//! This visitor performs the first pass of semantic analysis by traversing
//! the AST to collect all symbol declarations and build the scope hierarchy.

use typhon_ast::ast::AST;
use typhon_ast::nodes::{
    ASTNode,
    AssignmentStmt,
    AsyncForStmt,
    AsyncFunctionDecl,
    AsyncWithStmt,
    BasicIdent,
    ClassDecl,
    DictComprehensionExpr,
    ExceptHandler,
    ForStmt,
    FromImportStmt,
    FunctionDecl,
    GeneratorExpr,
    IfStmt,
    ImportStmt,
    LambdaExpr,
    ListComprehensionExpr,
    Module,
    NodeID,
    NodeKind,
    ParameterIdent,
    SetComprehensionExpr,
    TryStmt,
    TypeDecl,
    VariableDecl,
    VariableExpr,
    WhileStmt,
    WithStmt,
};
use typhon_ast::visitor::{MutVisitor, VisitorResult};

use crate::error::SemanticError;
use crate::symbol::{ScopeID, ScopeKind, Symbol, SymbolKind, SymbolTable};

/// Visitor that collects symbol declarations and builds the scope hierarchy.
///
/// This visitor implements the first phase of semantic analysis by:
/// - Traversing the AST in pre-order
/// - Creating scopes for module, functions, classes, and blocks
/// - Collecting symbols from all declarations
/// - Handling Python-specific scoping rules (hoisting, etc.)
/// - Detecting duplicate symbol definitions
#[derive(Debug)]
pub struct SymbolCollectorVisitor<'ast> {
    /// Reference to the AST being analyzed
    ast: &'ast AST,
    /// The symbol table being built
    symbol_table: SymbolTable,
    /// Collected errors during traversal
    errors: Vec<SemanticError>,
    /// Current module name (if any)
    current_module: Option<String>,
}

impl<'ast> SymbolCollectorVisitor<'ast> {
    /// Creates a new symbol collector visitor.
    pub fn new(ast: &'ast AST) -> Self {
        Self { ast, symbol_table: SymbolTable::new(), errors: Vec::new(), current_module: None }
    }

    /// Collects symbols from a module, returning the built symbol table.
    ///
    /// ## Errors
    ///
    /// Returns collected semantic errors if any were found during collection.
    pub fn collect(mut self, module_id: NodeID) -> Result<SymbolTable, Vec<SemanticError>> {
        // Visit the module to collect all symbols
        drop(self.visit_module(module_id));

        // Return errors if any were collected
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(self.symbol_table)
    }

    /// Enters a new scope of the specified kind.
    ///
    /// Creates a new scope as a child of the current scope and pushes it onto the scope stack.
    fn enter_scope(&mut self, kind: ScopeKind) -> ScopeID {
        let parent = self.symbol_table.current_scope();
        let scope_id = self.symbol_table.create_scope(kind, parent);
        self.symbol_table.enter_scope(scope_id);

        scope_id
    }

    /// Exits the current scope.
    fn exit_scope(&mut self) { let _ = self.symbol_table.exit_scope(); }

    /// Defines a symbol in the current scope.
    ///
    /// ## Errors
    ///
    /// Adds a duplicate symbol error to the errors list if a symbol with the
    /// same name already exists in the current scope.
    fn define_symbol(&mut self, name: String, kind: SymbolKind, node_id: NodeID) {
        let Some(scope_id) = self.symbol_table.current_scope() else {
            self.errors.push(SemanticError::NoActiveScope);
            return;
        };

        // Get the node's span
        let span = match self.ast.get_node(node_id) {
            Some(node) => node.span,
            None => {
                // Node doesn't exist, skip symbol definition
                return;
            }
        };

        let symbol = Symbol::new(name.clone(), kind, node_id, span, scope_id);

        if let Err(err) = self.symbol_table.define_symbol(name, symbol) {
            self.errors.push(err);
        }
    }

    /// Collects parameters from a function or lambda.
    fn collect_parameters(&mut self, parameters: &[NodeID]) {
        for &param_id in parameters {
            // Parameters are ParameterIdent nodes
            if let Ok(param) = self.ast.get_as::<ParameterIdent>(param_id) {
                self.define_symbol(param.name.clone(), SymbolKind::Parameter, param_id);
            }
        }
    }

    /// Pre-registers function and class names for hoisting.
    ///
    /// In Python, functions and classes are available throughout their containing scope,
    /// not just after their definition. This method pre-registers them before processing
    /// the body.
    fn hoist_declarations(&mut self, statements: &[NodeID]) {
        for &stmt_id in statements {
            if let Some(node) = self.ast.get_node(stmt_id)
                && node.kind == NodeKind::Declaration
            {
                // Check if it's a function or class declaration
                if let Ok(func) = self.ast.get_as::<FunctionDecl>(stmt_id) {
                    self.define_symbol(func.name.clone(), SymbolKind::Function, stmt_id);
                } else if let Ok(class) = self.ast.get_as::<ClassDecl>(stmt_id) {
                    self.define_symbol(class.name.clone(), SymbolKind::Class, stmt_id);
                }
            }
        }
    }
}

impl MutVisitor<()> for SymbolCollectorVisitor<'_> {
    fn visit(&mut self, node_id: NodeID) -> Option<()> {
        // Get node and dispatch based on kind
        let node = self.ast.get_node(node_id)?;

        match node.kind {
            NodeKind::Module => self.visit_module(node_id).ok(),
            NodeKind::Declaration => {
                // Try each declaration type
                if self.ast.get_as::<FunctionDecl>(node_id).is_ok() {
                    self.visit_function_decl(node_id).ok()
                } else if self.ast.get_as::<AsyncFunctionDecl>(node_id).is_ok() {
                    self.visit_async_function_decl(node_id).ok()
                } else if self.ast.get_as::<ClassDecl>(node_id).is_ok() {
                    self.visit_class_decl(node_id).ok()
                } else if self.ast.get_as::<VariableDecl>(node_id).is_ok() {
                    self.visit_variable_decl(node_id).ok()
                } else if self.ast.get_as::<TypeDecl>(node_id).is_ok() {
                    self.visit_type_decl(node_id).ok()
                } else {
                    Some(())
                }
            }
            NodeKind::Statement => {
                // Try specific statement types
                if self.visit_assignment_stmt(node_id).is_ok()
                    || self.visit_for_stmt(node_id).is_ok()
                    || self.visit_async_for_stmt(node_id).is_ok()
                    || self.visit_while_stmt(node_id).is_ok()
                    || self.visit_if_stmt(node_id).is_ok()
                    || self.visit_with_stmt(node_id).is_ok()
                    || self.visit_async_with_stmt(node_id).is_ok()
                    || self.visit_try_stmt(node_id).is_ok()
                    || self.visit_import_stmt(node_id).is_ok()
                    || self.visit_from_import_stmt(node_id).is_ok()
                {
                    return Some(());
                }

                // For other statements, just visit their children
                if let Some(node) = self.ast.get_node(node_id) {
                    for child_id in node.data.children() {
                        let _ = self.visit(child_id);
                    }
                }

                Some(())
            }
            NodeKind::Expression => {
                // Try specific expression types that create scopes
                if self.visit_lambda_expr(node_id).is_ok()
                    || self.visit_list_comprehension_expr(node_id).is_ok()
                    || self.visit_dict_comprehension_expr(node_id).is_ok()
                    || self.visit_set_comprehension_expr(node_id).is_ok()
                    || self.visit_generator_expr(node_id).is_ok()
                {
                    return Some(());
                }

                // For other expressions, just visit their children
                if let Some(node) = self.ast.get_node(node_id) {
                    for child_id in node.data.children() {
                        let _ = self.visit(child_id);
                    }
                }

                Some(())
            }
            _ => Some(()),
        }
    }

    fn visit_module(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let module = self.ast.get_as::<Module>(node_id)?;

        // Store module name
        self.current_module = Some(module.name.clone());

        // The module scope was already created in SymbolTable::new()
        // Just hoist declarations and process statements
        self.hoist_declarations(&module.statements);

        // Process all statements
        for &stmt_id in &module.statements {
            let _ = self.visit(stmt_id);
        }

        Ok(())
    }

    fn visit_function_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let func = self.ast.get_as::<FunctionDecl>(node_id)?;

        // Function name was already hoisted, enter function scope
        let scope_id = self.enter_scope(ScopeKind::Function);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        // Collect parameters
        self.collect_parameters(&func.parameters);

        // Hoist nested declarations in function body
        self.hoist_declarations(&func.body);

        // Process function body
        for &stmt_id in &func.body {
            let _ = self.visit(stmt_id);
        }

        self.exit_scope();

        Ok(())
    }

    fn visit_async_function_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let func = self.ast.get_as::<AsyncFunctionDecl>(node_id)?;

        // Similar to regular function
        let scope_id = self.enter_scope(ScopeKind::Function);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        self.collect_parameters(&func.parameters);
        self.hoist_declarations(&func.body);

        for &stmt_id in &func.body {
            let _ = self.visit(stmt_id);
        }

        self.exit_scope();

        Ok(())
    }

    fn visit_class_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let class = self.ast.get_as::<ClassDecl>(node_id)?;

        // Class name was already hoisted, enter class scope
        let scope_id = self.enter_scope(ScopeKind::Class);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        // Hoist method declarations
        self.hoist_declarations(&class.body);

        // Process class body
        for &stmt_id in &class.body {
            let _ = self.visit(stmt_id);
        }

        self.exit_scope();

        Ok(())
    }

    fn visit_variable_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let var = self.ast.get_as::<VariableDecl>(node_id)?;

        self.define_symbol(var.name.clone(), SymbolKind::Variable, node_id);

        // Visit the value expression if present
        if let Some(value_id) = var.value {
            let _ = self.visit(value_id);
        }

        Ok(())
    }

    fn visit_type_decl(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let type_decl = self.ast.get_as::<TypeDecl>(node_id)?;

        self.define_symbol(type_decl.name.clone(), SymbolKind::Variable, node_id);

        Ok(())
    }

    fn visit_assignment_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let assign = self.ast.get_as::<AssignmentStmt>(node_id)?;

        // For simple assignments, collect the target as a variable
        // The target might be an identifier, tuple, or other pattern
        // Only define as a new symbol if it doesn't already exist in the scope chain
        // (this allows reassignment while tracking first declarations)
        if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(assign.target) {
            // Check if symbol already exists in scope chain
            if self.symbol_table.lookup_in_scope_chain(&var_expr.name).is_none() {
                self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, assign.target);
            }
        } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(assign.target) {
            // Check if symbol already exists in scope chain
            if self.symbol_table.lookup_in_scope_chain(&basic_ident.name).is_none() {
                self.define_symbol(basic_ident.name.clone(), SymbolKind::Variable, assign.target);
            }
        }

        // Visit the value expression
        let _ = self.visit(assign.value);

        Ok(())
    }

    fn visit_for_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let for_stmt = self.ast.get_as::<ForStmt>(node_id)?;

        // Python: loop variables are scoped to the containing function/module, not the loop
        // So we define them in the current scope, not a block scope

        // Collect target as a variable (iteration variable)
        if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(for_stmt.target) {
            self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, for_stmt.target);
        } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(for_stmt.target) {
            self.define_symbol(basic_ident.name.clone(), SymbolKind::Variable, for_stmt.target);
        }

        // Visit the iterable
        let _ = self.visit(for_stmt.iter);

        // For-loops don't create a new scope - just visit body statements directly
        for &stmt_id in &for_stmt.body {
            let _ = self.visit(stmt_id);
        }

        // Handle else body if present
        if let Some(else_body) = &for_stmt.else_body {
            for &stmt_id in else_body {
                let _ = self.visit(stmt_id);
            }
        }

        Ok(())
    }

    fn visit_async_for_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let for_stmt = self.ast.get_as::<AsyncForStmt>(node_id)?;

        // Similar to regular for loop - no new scope for loop body
        if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(for_stmt.target) {
            self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, for_stmt.target);
        } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(for_stmt.target) {
            self.define_symbol(basic_ident.name.clone(), SymbolKind::Variable, for_stmt.target);
        }

        let _ = self.visit(for_stmt.iter);

        for &stmt_id in &for_stmt.body {
            let _ = self.visit(stmt_id);
        }

        if let Some(else_body) = &for_stmt.else_body {
            for &stmt_id in else_body {
                let _ = self.visit(stmt_id);
            }
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let while_stmt = self.ast.get_as::<WhileStmt>(node_id)?;

        // Visit condition
        let _ = self.visit(while_stmt.test);

        // While-loops don't create new scopes - variables defined in while
        // are scoped to the containing function/module
        for &stmt_id in &while_stmt.body {
            let _ = self.visit(stmt_id);
        }

        // Handle else body if present
        if let Some(else_body) = &while_stmt.else_body {
            for &stmt_id in else_body {
                let _ = self.visit(stmt_id);
            }
        }

        Ok(())
    }

    fn visit_if_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let if_stmt = self.ast.get_as::<IfStmt>(node_id)?;

        // Visit condition
        let _ = self.visit(if_stmt.condition);

        // If-statements don't create new scopes - variables defined in if/else
        // are scoped to the containing function/module
        for &stmt_id in &if_stmt.body {
            let _ = self.visit(stmt_id);
        }

        // Handle elif branches
        for (cond_id, elif_body) in &if_stmt.elif_branches {
            let _ = self.visit(*cond_id);
            for &stmt_id in elif_body {
                let _ = self.visit(stmt_id);
            }
        }

        // Handle else body if present
        if let Some(else_body) = &if_stmt.else_body {
            for &stmt_id in else_body {
                let _ = self.visit(stmt_id);
            }
        }

        Ok(())
    }

    fn visit_with_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let with_stmt = self.ast.get_as::<WithStmt>(node_id)?;

        // Collect context manager variables
        for item in &with_stmt.items {
            // Visit the context expression (first element of tuple)
            let _ = self.visit(item.0);

            // If there's an optional variable (second element of tuple), define it
            if let Some(opt_var_id) = item.1 {
                if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(opt_var_id) {
                    self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, opt_var_id);
                } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(opt_var_id) {
                    self.define_symbol(basic_ident.name.clone(), SymbolKind::Variable, opt_var_id);
                }
            }
        }

        // Enter block scope for body
        let _ = self.enter_scope(ScopeKind::Block);
        for &stmt_id in &with_stmt.body {
            let _ = self.visit(stmt_id);
        }
        self.exit_scope();

        Ok(())
    }

    fn visit_async_with_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let with_stmt = self.ast.get_as::<AsyncWithStmt>(node_id)?;

        // Similar to regular with statement
        for item in &with_stmt.items {
            // Visit the context expression (first element of tuple)
            let _ = self.visit(item.0);

            // If there's an optional variable (second element of tuple), define it
            if let Some(opt_var_id) = item.1 {
                if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(opt_var_id) {
                    self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, opt_var_id);
                } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(opt_var_id) {
                    self.define_symbol(basic_ident.name.clone(), SymbolKind::Variable, opt_var_id);
                }
            }
        }

        let _ = self.enter_scope(ScopeKind::Block);

        for &stmt_id in &with_stmt.body {
            let _ = self.visit(stmt_id);
        }

        self.exit_scope();

        Ok(())
    }

    fn visit_try_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let try_stmt = self.ast.get_as::<TryStmt>(node_id)?;

        // Enter block scope for try body
        let _ = self.enter_scope(ScopeKind::Block);
        for &stmt_id in &try_stmt.body {
            let _ = self.visit(stmt_id);
        }

        self.exit_scope();

        // Handle exception handlers
        for &handler_id in &try_stmt.handlers {
            if let Ok(handler) = self.ast.get_as::<ExceptHandler>(handler_id) {
                // Enter scope for handler
                let _ = self.enter_scope(ScopeKind::Block);

                // If there's a name for the exception, define it as a variable
                if let Some(name_id) = handler.name {
                    if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(name_id) {
                        self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, name_id);
                    } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(name_id) {
                        self.define_symbol(basic_ident.name.clone(), SymbolKind::Variable, name_id);
                    }
                }

                // Process handler body
                for &stmt_id in &handler.body {
                    let _ = self.visit(stmt_id);
                }

                self.exit_scope();
            }
        }

        // Handle else body if present
        if let Some(else_body) = &try_stmt.else_body {
            let _ = self.enter_scope(ScopeKind::Block);
            for &stmt_id in else_body {
                let _ = self.visit(stmt_id);
            }

            self.exit_scope();
        }

        // Handle finally body if present
        if let Some(finally_body) = &try_stmt.finally_body {
            let _ = self.enter_scope(ScopeKind::Block);
            for &stmt_id in finally_body {
                let _ = self.visit(stmt_id);
            }

            self.exit_scope();
        }

        Ok(())
    }

    fn visit_import_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let import = self.ast.get_as::<ImportStmt>(node_id)?;

        // Define the imported name
        let name = import.alias.as_ref().unwrap_or(&import.module_parts[0]).clone();
        self.define_symbol(name, SymbolKind::Import, node_id);

        Ok(())
    }

    fn visit_from_import_stmt(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let import = self.ast.get_as::<FromImportStmt>(node_id)?;

        // Define each imported name
        for (name, alias) in &import.names {
            let symbol_name = alias.as_ref().unwrap_or(name).clone();
            self.define_symbol(symbol_name, SymbolKind::Import, node_id);
        }

        Ok(())
    }

    fn visit_lambda_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let lambda = self.ast.get_as::<LambdaExpr>(node_id)?;

        // Enter lambda scope
        let scope_id = self.enter_scope(ScopeKind::Lambda);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        // Collect parameters
        self.collect_parameters(&lambda.parameters);

        // Visit body expression
        let _ = self.visit(lambda.body);

        self.exit_scope();

        Ok(())
    }

    fn visit_list_comprehension_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let comp = self.ast.get_as::<ListComprehensionExpr>(node_id)?;

        // Enter comprehension scope
        let scope_id = self.enter_scope(ScopeKind::Comprehension);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        // Collect iteration variables from generators
        for generator in &comp.generators {
            // Define the target variable(s)
            if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(generator.target) {
                self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, generator.target);
            } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(generator.target) {
                self.define_symbol(
                    basic_ident.name.clone(),
                    SymbolKind::Variable,
                    generator.target,
                );
            }

            // Visit the iterable
            let _ = self.visit(generator.iter);

            // Visit filter conditions
            for &filter_id in &generator.ifs {
                let _ = self.visit(filter_id);
            }
        }

        // Visit the element expression
        let _ = self.visit(comp.element);

        self.exit_scope();

        Ok(())
    }

    fn visit_dict_comprehension_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let comp = self.ast.get_as::<DictComprehensionExpr>(node_id)?;

        let scope_id = self.enter_scope(ScopeKind::Comprehension);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        for generator in &comp.generators {
            if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(generator.target) {
                self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, generator.target);
            } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(generator.target) {
                self.define_symbol(
                    basic_ident.name.clone(),
                    SymbolKind::Variable,
                    generator.target,
                );
            }

            let _ = self.visit(generator.iter);

            for &filter_id in &generator.ifs {
                let _ = self.visit(filter_id);
            }
        }

        let _ = self.visit(comp.key);
        let _ = self.visit(comp.value);

        self.exit_scope();

        Ok(())
    }

    fn visit_set_comprehension_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let comp = self.ast.get_as::<SetComprehensionExpr>(node_id)?;

        let scope_id = self.enter_scope(ScopeKind::Comprehension);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        for generator in &comp.generators {
            if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(generator.target) {
                self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, generator.target);
            } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(generator.target) {
                self.define_symbol(
                    basic_ident.name.clone(),
                    SymbolKind::Variable,
                    generator.target,
                );
            }

            let _ = self.visit(generator.iter);

            for &filter_id in &generator.ifs {
                let _ = self.visit(filter_id);
            }
        }

        let _ = self.visit(comp.element);

        self.exit_scope();

        Ok(())
    }

    fn visit_generator_expr(&mut self, node_id: NodeID) -> VisitorResult<()> {
        let comp = self.ast.get_as::<GeneratorExpr>(node_id)?;
        let scope_id = self.enter_scope(ScopeKind::Comprehension);
        self.symbol_table.associate_node_with_scope(node_id, scope_id);

        for generator in &comp.generators {
            if let Ok(var_expr) = self.ast.get_as::<VariableExpr>(generator.target) {
                self.define_symbol(var_expr.name.clone(), SymbolKind::Variable, generator.target);
            } else if let Ok(basic_ident) = self.ast.get_as::<BasicIdent>(generator.target) {
                self.define_symbol(
                    basic_ident.name.clone(),
                    SymbolKind::Variable,
                    generator.target,
                );
            }

            let _ = self.visit(generator.iter);

            for &filter_id in &generator.ifs {
                let _ = self.visit(filter_id);
            }
        }

        let _ = self.visit(comp.element);

        self.exit_scope();

        Ok(())
    }
}
