//! Class lowering
//!
//! This module handles lowering of AST class declarations to MIR.

use typhon_ast::nodes::{
    AssignmentStmt,
    AttributeExpr,
    ClassDecl,
    FunctionDecl,
    NodeID,
    ParameterIdent,
    VariableExpr,
};
use typhon_mir::instr::{LocalID, MIRConst, MIRInstr, Terminator};
use typhon_mir::module::{MIRField, MIRTypeDef, MethodInfo};
use typhon_mir::types::{MIRType, TypeID};
use typhon_source::types::Span;

use crate::context::LoweringContext;
use crate::error::{LoweringError, LoweringResult};

impl LoweringContext<'_, '_> {
    /// Lowers a class declaration to MIR
    ///
    /// Creates a `MIRTypeDef` for the class and lowers all methods as functions.
    ///
    /// # Errors
    ///
    /// Returns an error if the class node is not found or if lowering fails.
    pub fn lower_class(&mut self, class_id: NodeID) -> LoweringResult<TypeID> {
        let class_node =
            self.ast().get_node(class_id).ok_or_else(|| LoweringError::InternalError {
                message: format!("Class node {class_id:?} not found"),
                span: Span::default(),
            })?;

        let class_decl = class_node
            .data
            .get_as::<ClassDecl>()
            .map_err(|err| LoweringError::InternalError { message: err, span: class_node.span })?;

        // Allocate type ID for this class
        let type_id = self.allocate_type_id();

        // Resolve base classes
        let base_type_ids = self.resolve_base_classes(&class_decl.bases);

        // Set current class context
        self.set_current_class(class_decl.name.clone(), type_id);

        // Analyze class body to extract fields and methods
        let (fields, methods, constructor, initializer, destructor) =
            self.analyze_class_body(&class_decl.body, type_id)?;

        // Create MIRTypeDef
        let type_def = MIRTypeDef {
            name: class_decl.name.clone(),
            type_id,
            fields,
            methods,
            base_classes: base_type_ids,
            is_abstract: false,
            constructor,
            initializer,
            destructor,
        };

        // Add type definition to module
        self.module_mut().types.push(type_def);

        // Clear current class context
        self.clear_current_class();

        Ok(type_id)
    }

    /// Analyzes class body to extract fields and methods
    fn analyze_class_body(
        &mut self,
        body: &[NodeID],
        type_id: TypeID,
    ) -> LoweringResult<(
        Vec<MIRField>,
        Vec<MethodInfo>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructor = None;
        let mut initializer = None;
        let mut destructor = None;
        let mut slot_index = 0;

        // First pass: collect methods
        for stmt_id in body {
            let node =
                self.ast().get_node(*stmt_id).ok_or_else(|| LoweringError::InternalError {
                    message: format!("Node {stmt_id:?} not found"),
                    span: Span::default(),
                })?;

            if let Ok(func) = node.data.get_as::<FunctionDecl>() {
                // Create method info
                let method_info =
                    self.create_method_info(func, &self.current_class().unwrap().0, slot_index)?;

                // Check for special methods
                if func.name == "__new__" {
                    constructor = Some(method_info.function_name.clone());
                } else if func.name == "__init__" {
                    initializer = Some(method_info.function_name.clone());
                } else if func.name == "__del__" {
                    destructor = Some(method_info.function_name.clone());
                }

                methods.push(method_info);
                slot_index += 1;
            }
        }

        // Second pass: extract fields from __init__ if it exists
        for stmt_id in body {
            let node =
                self.ast().get_node(*stmt_id).ok_or_else(|| LoweringError::InternalError {
                    message: format!("Node {stmt_id:?} not found"),
                    span: Span::default(),
                })?;

            if let Ok(func) = node.data.get_as::<FunctionDecl>()
                && func.name == "__init__"
            {
                fields = self.extract_fields_from_init(func)?;
                break;
            }
        }

        // Third pass: lower all methods
        for stmt_id in body {
            let node =
                self.ast().get_node(*stmt_id).ok_or_else(|| LoweringError::InternalError {
                    message: format!("Node {stmt_id:?} not found"),
                    span: Span::default(),
                })?;

            if node.data.get_as::<FunctionDecl>().is_ok() {
                let class_name = self.current_class().unwrap().0.clone();
                self.lower_method(*stmt_id, &class_name, type_id)?;
            }
        }

        Ok((fields, methods, constructor, initializer, destructor))
    }

    /// Creates method information from a function declaration
    ///
    /// ## Decorator Support
    ///
    /// Static and class methods are currently not detected because decorator checking is not
    /// yet implemented. Python uses decorators to mark methods as static or class methods:
    ///
    /// ```python
    /// class MyClass:
    ///     @staticmethod
    ///     def static_method():
    ///         pass
    ///
    ///     @classmethod
    ///     def class_method(cls):
    ///         pass
    /// ```
    ///
    /// To implement decorator checking, we need:
    ///
    /// - Access to the decorator list from the AST node
    /// - Decorator name resolution and comparison
    /// - Handling of decorator arguments and chaining
    ///
    /// TODO: Implement decorator checking once AST decorator support is added to method nodes
    fn create_method_info(
        &self,
        func: &FunctionDecl,
        class_name: &str,
        slot_index: usize,
    ) -> LoweringResult<MethodInfo> {
        // Mangle method name: ClassName__methodname
        let function_name = format!("{class_name}__{}", func.name);

        // Decorator checking not yet implemented - all methods treated as instance methods
        let is_static = false;
        let is_class_method = false;

        // Check if method is private (starts with underscore but not dunder)
        let is_private = func.name.starts_with('_') && !func.name.starts_with("__");

        Ok(MethodInfo {
            name: func.name.clone(),
            function_name,
            is_static,
            is_class_method,
            is_private,
            slot_index,
        })
    }

    /// Extracts fields from __init__ method
    fn extract_fields_from_init(&self, init_func: &FunctionDecl) -> LoweringResult<Vec<MIRField>> {
        let mut fields = Vec::new();
        let mut field_offset = 0;

        // Walk the __init__ body looking for self.x = value patterns
        for stmt_id in &init_func.body {
            if let Some(field) = self.try_extract_field_assignment(*stmt_id, &mut field_offset)? {
                fields.push(field);
            }
        }

        Ok(fields)
    }

    /// Lowers a method as a regular function with self parameter
    fn lower_method(
        &mut self,
        method_id: NodeID,
        class_name: &str,
        type_id: TypeID,
    ) -> LoweringResult<()> {
        let node = self.ast().get_node(method_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Method node {method_id:?} not found"),
            span: Span::default(),
        })?;

        let func = node
            .data
            .get_as::<FunctionDecl>()
            .map_err(|e| LoweringError::InternalError { message: e, span: node.span })?;

        // Mangle method name
        let mangled_name = format!("{class_name}__{}", func.name);

        // Create parameters - self is first, then regular parameters
        let mut params = vec![("self".to_string(), MIRType::Object { type_id: Some(type_id) })];

        for param_id in &func.parameters {
            if let Some(param_node) = self.ast().get_node(*param_id)
                && let Ok(param_ident) = param_node.data.get_as::<ParameterIdent>()
            {
                // Skip 'self' parameter (already added)
                if param_ident.name != "self" {
                    let param_ty = self.get_type(*param_id);
                    params.push((param_ident.name.clone(), param_ty));
                }
            }
        }

        // Get return type
        let return_type = func.return_type.map_or(MIRType::None, |rt| self.get_type(rt));

        // Create function builder
        let mut builder =
            typhon_mir::builder::FunctionBuilder::new(mangled_name, params.clone(), return_type);

        // Create entry block
        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);

        // Set current function
        self.set_current_function(Some(builder));

        // Register self parameter
        self.register_local("self".to_string(), LocalID(0));

        // Register other parameters
        for (idx, param_id) in func.parameters.iter().enumerate() {
            if let Some(param_node) = self.ast().get_node(*param_id)
                && let Ok(param_ident) = param_node.data.get_as::<ParameterIdent>()
            {
                // Skip self parameter
                if param_ident.name != "self" {
                    let local_id = LocalID((idx + 1) as u32);
                    self.register_local(param_ident.name.clone(), local_id);
                }
            }
        }

        // Lower method body
        for stmt_id in &func.body {
            self.lower_stmt(*stmt_id)?;
        }

        // Ensure return
        if !self.is_current_block_terminated() {
            let builder = self.current_function()?;
            let none_val = builder.add_instruction(MIRInstr::Const(MIRConst::None));
            builder.set_terminator(Terminator::Return(Some(none_val)));
        }

        // Build function and add to module
        let builder = self.take_current_function().unwrap();
        let mir_func = builder.build();
        self.module_mut().functions.push(mir_func);

        Ok(())
    }

    /// Resolves base class type IDs
    ///
    /// ## Base Class Resolution
    ///
    /// Resolves base class names to their type IDs using the symbol table when semantic
    /// context is available. For each base class:
    ///
    /// 1. Gets the base class node from the AST
    /// 2. Extracts the class name from the node
    /// 3. Looks up the class in the symbol table
    /// 4. Retrieves the type ID from the symbol
    ///
    /// ## Future Work
    ///
    /// - Track class inheritance hierarchy in MIR type definitions
    /// - Implement MRO calculation for multiple inheritance
    /// - Add validation for circular inheritance
    /// - Handle complex base class expressions (e.g., parameterized generics)
    fn resolve_base_classes(&self, bases: &[NodeID]) -> Vec<TypeID> {
        let mut base_type_ids = Vec::new();

        for &base_id in bases {
            // Get the base class node and extract the class name
            if let Some(node) = self.ast().get_node(base_id)
                && let Ok(var) = node.data.get_as::<VariableExpr>()
                && let Some(type_id) = self.resolve_base_class(&var.name)
            {
                base_type_ids.push(type_id);
            }
        }

        base_type_ids
    }

    /// Tries to extract a field assignment from a statement
    fn try_extract_field_assignment(
        &self,
        stmt_id: NodeID,
        offset: &mut usize,
    ) -> LoweringResult<Option<MIRField>> {
        let node = self.ast().get_node(stmt_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Statement node {stmt_id:?} not found"),
            span: Span::default(),
        })?;

        // Check if this is an assignment statement
        if let Ok(assign) = node.data.get_as::<AssignmentStmt>() {
            let target_node =
                self.ast().get_node(assign.target).ok_or_else(|| LoweringError::InternalError {
                    message: format!("Target node {:?} not found", assign.target),
                    span: Span::default(),
                })?;

            // Check if target is self.field_name
            if let Ok(attr) = target_node.data.get_as::<AttributeExpr>() {
                let obj_node = self.ast().get_node(attr.value).ok_or_else(|| {
                    LoweringError::InternalError {
                        message: format!("Object node {:?} not found", attr.value),
                        span: Span::default(),
                    }
                })?;

                if let Ok(var) = obj_node.data.get_as::<VariableExpr>()
                    && var.name == "self"
                {
                    // This is a field assignment!
                    let field_ty = self.get_type(assign.value);
                    let field = MIRField { name: attr.name.clone(), ty: field_ty, offset: *offset };
                    *offset += 1;
                    return Ok(Some(field));
                }
            }
        }

        Ok(None)
    }
}
