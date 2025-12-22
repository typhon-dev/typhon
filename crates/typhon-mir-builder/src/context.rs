//! Lowering context
//!
//! This module defines the context used during AST to MIR lowering, which tracks
//! the current state and provides helper methods for generating MIR.

use std::collections::HashMap;

use typhon_ast::ast::AST;
use typhon_ast::nodes::NodeID;
use typhon_mir::builder::FunctionBuilder;
use typhon_mir::instr::{BasicBlockID, LocalID, MIRInstr, ValueID};
use typhon_mir::module::MIRModule;
use typhon_mir::types::{MIRType, TypeID};
use typhon_source::types::Span;

use crate::error::{LoweringError, LoweringResult};

/// Context for a loop (break/continue targets)
#[derive(Debug, Clone, Copy)]
pub struct LoopContext {
    /// Block to jump to on continue
    pub continue_block: BasicBlockID,
    /// Block to jump to on break
    pub break_block: BasicBlockID,
}

/// Capture information for closures
#[derive(Debug, Clone)]
pub struct CaptureInfo {
    /// The name of the captured variable
    pub name: String,
    /// The MIR type of the captured variable
    pub ty: MIRType,
    /// The source local ID of the captured variable
    pub source_local: LocalID,
}

/// Context for exception handling (try/except)
#[derive(Debug, Clone, Copy)]
pub struct ExceptionContext {
    /// Handler block for exceptions
    pub handler_block: BasicBlockID,
    /// Cleanup block (finally clause)
    pub cleanup_block: Option<BasicBlockID>,
}

/// Context for AST→MIR lowering
#[derive(Debug)]
pub struct LoweringContext<'ast> {
    /// Reference to the AST being lowered
    ast: &'ast AST,
    /// Capture stack for closures (full capture information)
    capture_stack: Vec<Vec<CaptureInfo>>,
    /// Current class being lowered (class name, `type_id`)
    current_class: Option<(String, TypeID)>,
    /// Current function builder (if lowering a function)
    current_function: Option<FunctionBuilder>,
    /// Exception handler stack (for try/except)
    exception_stack: Vec<ExceptionContext>,
    /// Local variable mapping (variable name → MIR `LocalID`)
    locals: HashMap<String, LocalID>,
    /// Block stack (for break/continue)
    loop_stack: Vec<LoopContext>,
    /// MIR module being built
    module: MIRModule,
    /// Type ID counter for generating unique type IDs
    next_type_id: TypeID,
    /// Value mapping (for SSA)
    values: HashMap<NodeID, ValueID>,
}

impl<'ast> LoweringContext<'ast> {
    /// Creates a new lowering context
    pub fn new(ast: &'ast AST, module_name: String) -> Self {
        Self {
            ast,
            capture_stack: Vec::new(),
            current_class: None,
            current_function: None,
            exception_stack: Vec::new(),
            locals: HashMap::new(),
            loop_stack: Vec::new(),
            module: MIRModule {
                name: module_name,
                functions: Vec::new(),
                globals: Vec::new(),
                types: Vec::new(),
            },
            next_type_id: 1,
            values: HashMap::new(),
        }
    }

    /// Allocates a new unique type ID
    pub const fn allocate_type_id(&mut self) -> TypeID {
        let id = self.next_type_id;
        self.next_type_id += 1;

        id
    }

    /// Gets a reference to the AST
    #[must_use]
    pub const fn ast(&self) -> &'ast AST { self.ast }

    /// Get the MIR module
    #[must_use]
    pub fn build(self) -> MIRModule { self.module }

    /// Clears the current class
    pub fn clear_current_class(&mut self) { self.current_class = None; }

    /// Gets the current class being lowered
    #[must_use]
    pub const fn current_class(&self) -> Option<&(String, TypeID)> { self.current_class.as_ref() }

    /// Gets the current function builder
    ///
    /// # Errors
    ///
    /// Returns an error if no function is currently being lowered.
    pub fn current_function(&mut self) -> LoweringResult<&mut FunctionBuilder> {
        self.current_function.as_mut().ok_or_else(|| LoweringError::InternalError {
            message: "no current function".to_string(),
            span: Span::new(0, 0),
        })
    }

    /// Gets the current loop context (for break/continue)
    ///
    /// # Errors
    ///
    /// Returns an error if not currently inside a loop.
    pub fn current_loop(&self) -> LoweringResult<&LoopContext> {
        self.loop_stack
            .last()
            .ok_or_else(|| LoweringError::BreakOutsideLoop { span: Span::new(0, 0) })
    }

    /// Emit an instruction and return its value ID
    ///
    /// # Panics
    ///
    /// Panics if no current function is set.
    pub fn emit(&mut self, instr: MIRInstr) -> ValueID {
        self.current_function.as_mut().map_or_else(
            || panic!("No current function set"),
            |builder| builder.add_instruction(instr),
        )
    }

    /// Gets the capture index for a variable name
    #[must_use]
    pub fn get_capture_index(&self, name: &str) -> Option<usize> {
        self.capture_stack.last()?.iter().position(|cap| cap.name == name)
    }

    /// Gets capture information for a variable name
    #[must_use]
    pub fn get_capture_info(&self, name: &str) -> Option<&CaptureInfo> {
        self.capture_stack.last()?.iter().find(|cap| cap.name == name)
    }

    /// Gets a local ID for a variable name
    ///
    /// # Errors
    ///
    /// Returns an error if the variable is not found in the local scope.
    pub fn get_local(&self, name: &str) -> LoweringResult<LocalID> {
        self.locals.get(name).copied().ok_or_else(|| LoweringError::MissingSymbolInfo {
            node_id: NodeID::placeholder(),
            span: Span::new(0, 0),
        })
    }

    /// Gets the type for a node
    ///
    /// ## Type Information
    ///
    /// Currently returns [`MIRType::Object`] with `type_id: None` as the default type.
    /// This is appropriate for the MIR lowering phase as it operates independently of
    /// the semantic analysis phase.
    ///
    /// Future integration will connect this to the type environment from the semantic
    /// analysis phase, allowing precise type information to flow through to MIR when
    /// available. The type environment would provide:
    ///
    /// - Inferred types from type checker
    /// - Annotated types from source code
    /// - Class and function signature types
    ///
    /// TODO: Connect to type checker once semantic analysis integration is completed
    #[must_use]
    pub const fn get_type(&self, _node_id: NodeID) -> MIRType {
        // Use Object type as safe default - actual type determined at runtime
        MIRType::Object { type_id: None }
    }

    /// Gets the type for a local variable by ID
    ///
    /// ## Local Type Tracking
    ///
    /// Currently returns [`MIRType::Object`] as the default type for all local variables.
    /// Local types could be tracked more precisely by:
    /// - Storing type information in the local variable registry
    /// - Querying the function builder's local metadata
    /// - Propagating type information from assignments
    ///
    /// TODO: Track local types once type environment integration is completed
    #[must_use]
    pub const fn get_type_for_local(&self, _local_id: LocalID) -> MIRType {
        // Use Object type as safe default - actual type determined at runtime
        MIRType::Object { type_id: None }
    }

    /// Gets the type for a variable by name
    #[must_use]
    pub fn get_type_for_name(&self, name: &str) -> MIRType {
        // Check if it's a captured variable first
        if let Some(capture) = self.get_capture_info(name) {
            return capture.ty.clone();
        }

        // Otherwise use default object type
        MIRType::Object { type_id: None }
    }

    /// Gets a value ID for a node
    ///
    /// # Errors
    ///
    /// Returns an error if no value has been registered for the given node.
    pub fn get_value(&self, node_id: NodeID) -> LoweringResult<ValueID> {
        self.values
            .get(&node_id)
            .copied()
            .ok_or_else(|| LoweringError::MissingTypeInfo { node_id, span: Span::new(0, 0) })
    }

    /// Checks if we're currently in a closure
    #[must_use]
    pub const fn in_closure(&self) -> bool { !self.capture_stack.is_empty() }

    /// Checks if the current block is terminated
    #[must_use]
    pub fn is_current_block_terminated(&self) -> bool {
        self.current_function.as_ref().is_some_and(FunctionBuilder::is_current_block_terminated)
    }

    /// Gets a reference to the MIR module
    #[must_use]
    pub const fn module(&self) -> &MIRModule { &self.module }

    /// Gets a mutable reference to the MIR module
    pub const fn module_mut(&mut self) -> &mut MIRModule { &mut self.module }

    /// Create a new basic block
    ///
    /// # Panics
    ///
    /// Panics if no current function is set.
    pub fn new_block(&mut self) -> BasicBlockID {
        self.current_function
            .as_mut()
            .map_or_else(|| panic!("No current function set"), FunctionBuilder::create_block)
    }

    /// Allocate a new local variable
    ///
    /// # Panics
    ///
    /// Panics if no current function is set.
    pub fn new_local(&mut self, name: Option<String>, ty: MIRType, mutable: bool) -> LocalID {
        self.current_function.as_mut().map_or_else(
            || panic!("No current function set"),
            |builder| builder.allocate_local(name, ty, mutable),
        )
    }

    /// Pops the current closure scope
    pub fn pop_closure_scope(&mut self) { drop(self.capture_stack.pop()); }

    /// Pops the exception context
    pub fn pop_exception_context(&mut self) { let _ = self.exception_stack.pop(); }

    /// Pops the loop context
    pub fn pop_loop_context(&mut self) { let _ = self.loop_stack.pop(); }

    /// Pushes a new closure scope with captured variables
    pub fn push_closure_scope(&mut self, captures: &[CaptureInfo]) {
        self.capture_stack.push(captures.to_vec());
    }

    /// Pushes an exception context
    pub fn push_exception_context(&mut self, ctx: ExceptionContext) {
        self.exception_stack.push(ctx);
    }

    /// Pushes a loop context
    pub fn push_loop_context(&mut self, ctx: LoopContext) { self.loop_stack.push(ctx); }

    /// Registers a local variable mapping
    pub fn register_local(&mut self, name: String, local_id: LocalID) {
        let _ = self.locals.insert(name, local_id);
    }

    /// Registers a value mapping
    pub fn register_value(&mut self, node_id: NodeID, value_id: ValueID) {
        let _ = self.values.insert(node_id, value_id);
    }

    /// Sets the current class being lowered
    pub fn set_current_class(&mut self, class_name: String, type_id: TypeID) {
        self.current_class = Some((class_name, type_id));
    }

    /// Sets the current function builder
    pub fn set_current_function(&mut self, builder: Option<FunctionBuilder>) {
        self.current_function = builder;
    }

    /// Takes the current function builder, leaving None in its place
    pub const fn take_current_function(&mut self) -> Option<FunctionBuilder> {
        self.current_function.take()
    }
}
