//! Closure lowering
//!
//! This module handles lowering of nested functions with captured variables to MIR.

use std::collections::HashSet;

use typhon_ast::nodes::{ASTNode, FunctionDecl, NodeID, ParameterIdent, VariableExpr};
use typhon_mir::function::MIRCapture;
use typhon_mir::instr::{LocalID, MIRConst, MIRInstr, Terminator};
use typhon_mir::types::MIRType;
use typhon_source::types::Span;

use crate::context::{CaptureInfo, LoweringContext};
use crate::error::{LoweringError, LoweringResult};

impl LoweringContext<'_> {
    /// Analyzes what variables a function captures from its enclosing scope
    fn analyze_captures(&self, func_id: NodeID) -> LoweringResult<Vec<String>> {
        let node = self.ast().get_node(func_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Function node {func_id:?} not found"),
            span: Span::new(0, 0),
        })?;

        let func = node
            .data
            .get_as::<FunctionDecl>()
            .map_err(|e| LoweringError::InternalError { message: e, span: node.span })?;

        // Variables referenced in the function body
        let mut referenced = HashSet::new();

        // Variables defined in the function (params + local assignments)
        let mut defined = HashSet::new();

        // Add parameters to defined set
        for param_id in &func.parameters {
            if let Some(param_node) = self.ast().get_node(*param_id)
                && let Ok(param_ident) = param_node.data.get_as::<ParameterIdent>()
            {
                let _ = defined.insert(param_ident.name.clone());
            }
        }

        // Walk function body to find referenced and defined variables
        for stmt_id in &func.body {
            self.collect_variables(*stmt_id, &mut referenced, &mut defined)?;
        }

        // Captures are variables that are referenced but not defined locally
        // We don't filter by scope here because the outer function may not be fully lowered yet
        let mut captures: Vec<String> = referenced.difference(&defined).cloned().collect();

        // Sort for deterministic output
        captures.sort();

        Ok(captures)
    }

    /// Collects variables referenced and defined in a statement subtree
    fn collect_variables(
        &self,
        node_id: NodeID,
        referenced: &mut HashSet<String>,
        defined: &mut HashSet<String>,
    ) -> LoweringResult<()> {
        let node = self.ast().get_node(node_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Node {node_id:?} not found"),
            span: Span::new(0, 0),
        })?;

        // Check if this is a variable expression (reference)
        if let Ok(var) = node.data.get_as::<VariableExpr>() {
            let _ = referenced.insert(var.name.clone());

            return Ok(()); // Variable nodes don't have children to recurse on
        }

        // Check if this is an assignment (definition)
        if let Ok(assign) = node.data.get_as::<typhon_ast::nodes::AssignmentStmt>() {
            // Get the target - if it's a simple variable, mark it as defined
            if let Some(target_node) = self.ast().get_node(assign.target)
                && let Ok(var) = target_node.data.get_as::<VariableExpr>()
            {
                let _ = defined.insert(var.name.clone());
            }
            // Don't recurse on the assignment target, but do recurse on the value
            // (handled below by children() call)
        }

        // Recurse on all children using Visitable trait
        for child_id in node.data.children() {
            self.collect_variables(child_id, referenced, defined)?;
        }

        Ok(())
    }

    /// Gets the type for a value
    ///
    /// ## Type Tracking
    ///
    /// Currently returns [`MIRType::Object`] with `type_id: None` as the default type for all
    /// values. This is a safe default for the MIR phase as Python's dynamic type system
    /// determines actual types at runtime.
    ///
    /// Future enhancements could track more specific type information by:
    ///
    /// - Querying the type environment from semantic analysis
    /// - Tracking types through SSA value definitions
    /// - Propagating type information through data flow analysis
    ///
    /// TODO: Implement proper type tracking for values once type environment integration is completed
    const fn get_type_for_value(&self, _value: typhon_mir::instr::ValueID) -> MIRType {
        // Use Object type as safe default - actual type determined at runtime
        MIRType::Object { type_id: None }
    }

    /// Lowers a nested function as a closure
    ///
    /// Analyzes captured variables and creates a closure with an environment.
    ///
    /// ## Errors
    ///
    /// Returns an error if the function node is not found or lowering fails.
    ///
    /// ## Panics
    ///
    /// Panics if the current function builder is not set when expected.
    pub fn lower_nested_function(&mut self, func_id: NodeID) -> LoweringResult<()> {
        let node = self.ast().get_node(func_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Function node {func_id:?} not found"),
            span: Span::new(0, 0),
        })?;

        let func = node
            .data
            .get_as::<FunctionDecl>()
            .map_err(|e| LoweringError::InternalError { message: e, span: node.span })?;

        // Analyze what variables this function captures
        let captures = self.analyze_captures(func_id)?;

        // Create function name (nested functions get mangled with parent name)
        let parent_func = self.module().functions.last().map(|f| f.name.clone());
        let func_name = parent_func
            .map_or_else(|| func.name.clone(), |parent| format!("{parent}__{}", func.name));

        // Build parameter list
        let mut params = Vec::new();
        for param_id in &func.parameters {
            if let Some(param_node) = self.ast().get_node(*param_id)
                && let Ok(param_ident) = param_node.data.get_as::<ParameterIdent>()
            {
                let param_ty = self.get_type(*param_id);
                params.push((param_ident.name.clone(), param_ty));
            }
        }

        // Get return type
        let return_type = func.return_type.map_or(MIRType::None, |rt| self.get_type(rt));
        let return_type_clone = return_type.clone();

        // Create function builder
        let mut builder = typhon_mir::builder::FunctionBuilder::new(
            func_name.clone(),
            params.clone(),
            return_type,
        );

        // Build MIRCapture structs for the builder
        let mir_captures: Vec<MIRCapture> = captures
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let ty = self.get_type_for_name(name);
                #[allow(clippy::cast_possible_truncation)]
                let source_local = LocalID(idx as u32);
                MIRCapture { name: name.clone(), ty, source_local }
            })
            .collect();

        // Set captures on the builder
        builder.set_captures(mir_captures);

        // Create entry block
        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);

        // Save the old function builder
        let old_function = self.take_current_function();

        // Set new function as current
        self.set_current_function(Some(builder));

        // Create new closure scope with captures
        let capture_infos: Vec<CaptureInfo> = captures
            .iter()
            .enumerate()
            .map(|(index, name)| {
                #[allow(clippy::cast_possible_truncation)]
                let fallback_local = LocalID(index as u32);
                CaptureInfo {
                    name: name.clone(),
                    ty: self.get_type_for_name(name),
                    source_local: self.get_local(name).unwrap_or(fallback_local),
                }
            })
            .collect();
        self.push_closure_scope(&capture_infos);

        // Register parameters as locals
        for (idx, param_id) in func.parameters.iter().enumerate() {
            if let Some(param_node) = self.ast().get_node(*param_id)
                && let Ok(param_ident) = param_node.data.get_as::<ParameterIdent>()
            {
                #[allow(clippy::cast_possible_truncation)]
                let local_id = LocalID(idx as u32);
                self.register_local(param_ident.name.clone(), local_id);
            }
        }

        // Lower function body
        for stmt_id in &func.body {
            self.lower_stmt(*stmt_id)?;
        }

        // Ensure return
        if !self.is_current_block_terminated() {
            let builder = self.current_function()?;
            let none_val = builder.add_instruction(MIRInstr::Const(MIRConst::None));
            builder.set_terminator(Terminator::Return(Some(none_val)));
        }

        // Build the function
        let builder = self.take_current_function().unwrap();
        let mir_func = builder.build();

        // Pop closure scope
        self.pop_closure_scope();

        // Restore old function
        self.set_current_function(old_function);

        // Add function to module
        self.module_mut().functions.push(mir_func);

        // If we're in an enclosing function, emit CreateClosure instruction
        if self.current_function().is_ok() {
            // Collect local IDs and types first (immutable borrows)
            let capture_locals: Vec<(LocalID, MIRType)> = captures
                .iter()
                .filter_map(|name| {
                    let local_id = self.get_local(name).ok()?;
                    let ty = self.get_type_for_local(local_id);
                    Some((local_id, ty))
                })
                .collect();

            // Determine closure type
            let param_types: Vec<MIRType> = params.iter().map(|(_, ty)| ty.clone()).collect();
            let captured_types: Vec<MIRType> =
                captures.iter().map(|name| self.get_type_for_name(name)).collect();

            let closure_ty = MIRType::Closure {
                params: param_types,
                return_type: Box::new(return_type_clone),
                captured: captured_types,
            };

            // Now get mutable borrow and emit instructions
            let enclosing_builder = self.current_function()?;

            // Load all captures
            let capture_values: Vec<_> = capture_locals
                .into_iter()
                .map(|(local_id, ty)| {
                    enclosing_builder.add_instruction(MIRInstr::Load { local: local_id, ty })
                })
                .collect();

            // Create the closure
            let closure_val = enclosing_builder.add_instruction(MIRInstr::CreateClosure {
                function_name: func_name.clone(),
                captures: capture_values,
                ty: closure_ty,
            });

            // Store closure in a local variable so it can be referenced and called
            // The closure is stored with the function's name in the enclosing scope
            let closure_ty = MIRType::Object { type_id: None };
            let closure_local = enclosing_builder.allocate_local(
                Some(func_name),
                closure_ty,
                false, // immutable
            );
            let _ = enclosing_builder
                .add_instruction(MIRInstr::Store { local: closure_local, value: closure_val });

            // Register the closure local so it can be referenced by name
            self.register_local(func.name.clone(), closure_local);
        }

        Ok(())
    }

    /// Lowers a variable access, handling both locals and captures
    ///
    /// ## Errors
    ///
    /// Returns an error if the variable is not found in any scope.
    pub fn lower_variable_access(
        &mut self,
        name: &str,
    ) -> LoweringResult<typhon_mir::instr::ValueID> {
        // Check if it's a local variable
        if let Ok(local_id) = self.get_local(name) {
            let ty = self.get_type_for_local(local_id);
            let builder = self.current_function()?;

            return Ok(builder.add_instruction(MIRInstr::Load { local: local_id, ty }));
        }

        // Check if it's a captured variable
        if let Some(capture_idx) = self.get_capture_index(name) {
            let capture_ty = self
                .get_capture_info(name)
                .map_or_else(|| MIRType::Object { type_id: None }, |c| c.ty.clone());
            let builder = self.current_function()?;

            // For GetCapture, we need the closure value itself
            // We track the closure object through the current function context
            let closure_val = builder.allocate_value();

            return Ok(builder.add_instruction(MIRInstr::GetCapture {
                closure: closure_val,
                index: capture_idx,
                ty: capture_ty,
            }));
        }

        // Otherwise it's a global or undefined variable
        Err(LoweringError::InternalError {
            message: format!("Undefined variable: {name}"),
            span: Span::new(0, 0),
        })
    }

    /// Lowers a variable assignment, handling both locals and captures
    ///
    /// ## Errors
    ///
    /// Returns an error if the assignment cannot be lowered.
    pub fn lower_variable_assignment(
        &mut self,
        name: &str,
        value: typhon_mir::instr::ValueID,
    ) -> LoweringResult<()> {
        // Check if it's a local variable
        if let Ok(local_id) = self.get_local(name) {
            let builder = self.current_function()?;
            let _ = builder.add_instruction(MIRInstr::Store { local: local_id, value });

            return Ok(());
        }

        // Check if it's a captured variable
        if let Some(capture_idx) = self.get_capture_index(name) {
            let builder = self.current_function()?;
            // For SetCapture, we need the closure value itself
            let closure_val = builder.allocate_value();
            let _ = builder.add_instruction(MIRInstr::SetCapture {
                closure: closure_val,
                index: capture_idx,
                value,
            });

            return Ok(());
        }

        // Otherwise it's a new local variable - allocate it
        let ty = self.get_type_for_value(value);
        let name_str = name.to_string();
        let builder = self.current_function()?;
        let local_id = builder.allocate_local(Some(name_str.clone()), ty, true);
        let _ = builder.add_instruction(MIRInstr::Store { local: local_id, value });

        self.register_local(name_str, local_id);

        Ok(())
    }
}
