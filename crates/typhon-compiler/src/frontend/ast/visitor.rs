use super::*;

/// Visitor trait for AST nodes
pub trait Visitor<T> {
    /// Visits a module
    fn visit_module(&mut self, module: &Module) -> T;

    /// Visits a statement
    fn visit_statement(&mut self, stmt: &Statement) -> T;

    /// Visits an expression
    fn visit_expression(&mut self, expr: &Expression) -> T;

    /// Visits a type expression
    fn visit_type_expression(&mut self, type_expr: &TypeExpression) -> T;

    /// Visits a literal
    fn visit_literal(&mut self, lit: &Literal) -> T;
}

/// Mutable visitor trait for AST nodes
pub trait MutVisitor {
    /// Visits a module
    fn visit_module_mut(&mut self, module: &mut Module);

    /// Visits a statement
    fn visit_statement_mut(&mut self, stmt: &mut Statement);

    /// Visits an expression
    fn visit_expression_mut(&mut self, expr: &mut Expression);

    /// Visits a type expression
    fn visit_type_expression_mut(&mut self, type_expr: &mut TypeExpression);

    /// Visits a literal
    fn visit_literal_mut(&mut self, lit: &mut Literal);
}

/// Default implementation of the Visitor trait
pub trait DefaultVisitor<T>: Visitor<T> {
    /// Default value for a visitor
    fn default_value(&self) -> T;

    /// Visits a module with default implementation
    fn default_visit_module(&mut self, module: &Module) -> T {
        for stmt in &module.statements {
            self.visit_statement(stmt);
        }
        self.default_value()
    }

    /// Visits a statement with default implementation
    fn default_visit_statement(&mut self, stmt: &Statement) -> T {
        match stmt {
            Statement::Expression(expr) => {
                self.visit_expression(expr);
            }
            Statement::Assignment { target, value, .. } => {
                self.visit_expression(target);
                self.visit_expression(value);
            }
            Statement::FunctionDef {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                for param in parameters {
                    if let Some(ref type_annotation) = param.type_annotation {
                        self.visit_type_expression(type_annotation);
                    }
                    if let Some(ref default_value) = param.default_value {
                        self.visit_expression(default_value);
                    }
                }
                if let Some(ref ret_type) = return_type {
                    self.visit_type_expression(ret_type);
                }
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::ClassDef { bases, body, .. } => {
                for base in bases {
                    self.visit_expression(base);
                }
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::Return { value, .. } => {
                if let Some(ref val) = value {
                    self.visit_expression(val);
                }
            }
            Statement::Import { .. } | Statement::FromImport { .. } => {
                // No expressions to visit
            }
            Statement::If {
                condition,
                body,
                else_body,
                ..
            } => {
                self.visit_expression(condition);
                for stmt in body {
                    self.visit_statement(stmt);
                }
                if let Some(ref else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.visit_statement(stmt);
                    }
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                self.visit_expression(condition);
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::For {
                target, iter, body, ..
            } => {
                self.visit_expression(target);
                self.visit_expression(iter);
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::Pass { .. } | Statement::Break { .. } | Statement::Continue { .. } => {
                // No expressions to visit
            }
            Statement::VariableDecl {
                type_annotation,
                value,
                ..
            } => {
                if let Some(ref type_annot) = type_annotation {
                    self.visit_type_expression(type_annot);
                }
                if let Some(ref val) = value {
                    self.visit_expression(val);
                }
            }
        }
        self.default_value()
    }

    /// Visits an expression with default implementation
    fn default_visit_expression(&mut self, expr: &Expression) -> T {
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.visit_expression(operand);
            }
            Expression::Literal { value, .. } => {
                self.visit_literal(value);
            }
            Expression::Variable { .. } => {
                // No expressions to visit
            }
            Expression::Attribute { value, .. } => {
                self.visit_expression(value);
            }
            Expression::Subscript { value, index, .. } => {
                self.visit_expression(value);
                self.visit_expression(index);
            }
            Expression::Call {
                func,
                args,
                keywords,
                ..
            } => {
                self.visit_expression(func);
                for arg in args {
                    self.visit_expression(arg);
                }
                for (_, value) in keywords {
                    self.visit_expression(value);
                }
            }
            Expression::Lambda {
                parameters, body, ..
            } => {
                for param in parameters {
                    if let Some(ref type_annotation) = param.type_annotation {
                        self.visit_type_expression(type_annotation);
                    }
                    if let Some(ref default_value) = param.default_value {
                        self.visit_expression(default_value);
                    }
                }
                self.visit_expression(body);
            }
            Expression::List { elements, .. } | Expression::Tuple { elements, .. } => {
                for element in elements {
                    self.visit_expression(element);
                }
            }
            Expression::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.visit_expression(key);
                    self.visit_expression(value);
                }
            }
        }
        self.default_value()
    }

    /// Visits a type expression with default implementation
    fn default_visit_type_expression(&mut self, type_expr: &TypeExpression) -> T {
        match type_expr {
            TypeExpression::Named { .. } => {
                // No type expressions to visit
            }
            TypeExpression::Generic { base, args, .. } => {
                self.visit_type_expression(base);
                for arg in args {
                    self.visit_type_expression(arg);
                }
            }
            TypeExpression::Union { types, .. } => {
                for ty in types {
                    self.visit_type_expression(ty);
                }
            }
            TypeExpression::Optional { inner, .. } => {
                self.visit_type_expression(inner);
            }
            TypeExpression::Callable {
                parameter_types,
                return_type,
                ..
            } => {
                for param_type in parameter_types {
                    self.visit_type_expression(param_type);
                }
                self.visit_type_expression(return_type);
            }
        }
        self.default_value()
    }

    /// Visits a literal with default implementation
    fn default_visit_literal(&mut self, _lit: &Literal) -> T {
        self.default_value()
    }
}
