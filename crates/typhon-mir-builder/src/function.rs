//! Function lowering
//!
//! This module handles lowering of function declarations to MIR.

use typhon_ast::nodes::{AnyNode, NodeID};
use typhon_mir::builder::FunctionBuilder;
use typhon_mir::instr::{MIRConst, Terminator};
use typhon_mir::types::MIRType;

use crate::context::LoweringContext;
use crate::error::{LoweringError, LoweringResult};

impl LoweringContext<'_, '_> {
    /// Lower a function declaration to MIR
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - The function node does not exist in the AST
    /// - Lowering of the function body fails
    /// - Type resolution fails
    pub fn lower_function(&mut self, func_id: NodeID) -> LoweringResult<()> {
        // Get the function node from the AST
        let node = self.ast().get_node(func_id).ok_or_else(|| LoweringError::InternalError {
            message: format!("Function node not found: {func_id}"),
            span: typhon_source::types::Span::new(0, 0),
        })?;

        let AnyNode::FunctionDecl(func) = &node.data else {
            return Err(LoweringError::InternalError {
                message: format!("Node is not a function: {func_id}"),
                span: node.span,
            });
        };

        // Create parameters with types
        let params: Vec<(String, MIRType)> = func
            .parameters
            .iter()
            .map(|param_id| {
                let param_node =
                    self.ast().get_node(*param_id).ok_or_else(|| LoweringError::InternalError {
                        message: format!("Parameter node not found: {param_id}"),
                        span: func.span,
                    })?;

                let param_name = match &param_node.data {
                    AnyNode::ParameterIdent(param) => param.name.clone(),
                    _ => {
                        return Err(LoweringError::InternalError {
                            message: format!("Parameter is not a ParameterIdent: {param_id}"),
                            span: param_node.span,
                        });
                    }
                };

                // Get parameter type (for now, use Object type as placeholder)
                let param_ty = self.get_type(*param_id);

                Ok((param_name, param_ty))
            })
            .collect::<LoweringResult<Vec<_>>>()?;

        // Get return type
        let return_type = func
            .return_type
            .map_or(MIRType::None, |return_annotation| self.get_type(return_annotation));

        // Create function builder
        let mut builder = FunctionBuilder::new(func.name.clone(), params, return_type);

        // Create entry block
        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);

        // Register parameter locals by creating a mapping
        // Parameters become locals with IDs 0, 1, 2, ... in order
        for (idx, &param_id) in func.parameters.iter().enumerate() {
            let local_id = typhon_mir::instr::LocalID(idx as u32);

            // Get the parameter name to register it
            if let Some(param_node) = self.ast().get_node(param_id)
                && let AnyNode::ParameterIdent(param_ident) = &param_node.data
            {
                self.register_local(param_ident.name.clone(), local_id);
            }
        }

        // Set current function
        self.set_current_function(Some(builder));

        // Lower function body
        for stmt_id in &func.body {
            self.lower_stmt(*stmt_id)?;
        }

        // Ensure function returns (return None if there's no explicit return)
        if !self.is_current_block_terminated() {
            let builder = self.current_function()?;
            let none_val =
                builder.add_instruction(typhon_mir::instr::MIRInstr::Const(MIRConst::None));
            builder.set_terminator(Terminator::Return(Some(none_val)));
        }

        // Build the MIR function and add it to the module
        let builder = self.take_current_function().ok_or_else(|| LoweringError::InternalError {
            message: "No current function set".to_string(),
            span: func.span,
        })?;

        let mir_func = builder.build();
        self.module_mut().functions.push(mir_func);

        Ok(())
    }
}
