use std::sync::{Arc, Mutex};

use inkwell::types::{AnyType, AnyTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue};

use crate::backend::codegen::{DefaultNodeVisitor, NodeVisitor};
use crate::backend::{
    CodeGenContext,
    CodeGenError,
    CodeGenResult,
    CodeGenValue,
    LLVMContext,
    SymbolTable,
};
use crate::frontend::{
    BinaryOperator,
    Expression,
    Identifier,
    Literal,
    Module,
    Statement,
    TypeExpression,
    UnaryOperator,
};

/// Represents the current state of code generation.
pub struct CodeGenState {
    /// The symbol table.
    pub symbol_table: SymbolTable,
    /// The current function being generated.
    pub current_function: Option<FunctionValue>,
    /// Whether a return statement has been generated.
    pub returned: bool,
}

impl Default for CodeGenState {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenState {
    /// Create a new code generation state.
    pub fn new() -> Self {
        Self { symbol_table: SymbolTable::new(), current_function: None, returned: false }
    }
}

/// Code generator for Typhon AST.
pub struct CodeGenerator {
    /// The immutable context.
    pub context: CodeGenContext,
    /// The mutable state.
    pub state: Arc<Mutex<CodeGenState>>,
    /// The node visitor implementation.
    pub visitor: DefaultNodeVisitor,
}

impl CodeGenerator {
    /// Create a new code generator.
    pub fn new(llvm_context: Arc<LLVMContext>) -> Self {
        let context = CodeGenContext::new(llvm_context.clone());
        let state = CodeGenState::new();

        Self { context, state: Arc::new(Mutex::new(state)), visitor: DefaultNodeVisitor {} }
    }

    /// Build a load instruction.
    pub fn build_load(&self, ptr: BasicValueEnum, name: &str) -> BasicValueEnum {
        // Check if the pointer is actually a pointer type
        let ptr_value = ptr.into_pointer_value();

        // Get the type of the pointed-to value
        let pointee_type = ptr_value.get_type().as_any_type_enum();

        // Build the load based on the pointee type
        let builder = self.context.llvm_context.builder();
        match pointee_type {
            AnyTypeEnum::IntType(_) => builder
                .build_load(pointee_type.into_int_type(), ptr_value, name)
                .expect("Failed to build int load instruction"),
            AnyTypeEnum::FloatType(_) => builder
                .build_load(pointee_type.into_float_type(), ptr_value, name)
                .expect("Failed to build float load instruction"),
            AnyTypeEnum::PointerType(_) => builder
                .build_load(pointee_type.into_pointer_type(), ptr_value, name)
                .expect("Failed to build pointer load instruction"),
            AnyTypeEnum::StructType(_) => builder
                .build_load(pointee_type.into_struct_type(), ptr_value, name)
                .expect("Failed to build struct load instruction"),
            AnyTypeEnum::ArrayType(_) => builder
                .build_load(pointee_type.into_array_type(), ptr_value, name)
                .expect("Failed to build array load instruction"),
            _ => panic!("Unsupported pointee type for load: {pointee_type:?}"),
        }
    }

    /// Build a store instruction.
    pub fn build_store(&self, ptr: BasicValueEnum, value: BasicValueEnum) {
        // Check if the pointer is actually a pointer type
        let ptr_value = ptr.into_pointer_value();
        let builder = self.context.llvm_context.builder();

        builder.build_store(ptr_value, value).expect("Failed to build store instruction");
    }

    /// Compile an AST to LLVM IR.
    pub fn compile(&mut self, statements: &[Statement]) -> CodeGenResult<()> {
        let symbol_table = &mut SymbolTable::new();
        let context = &mut CodeGenContext::new(self.context.llvm_context.clone());
        self.context.clone_into(context);

        // Process each top-level statement
        for stmt in statements {
            self.visit_statement(context, symbol_table, stmt)?;
        }

        // Verify the module
        if self.context.llvm_context.module().verify().is_err() {
            return Err(CodeGenError::code_gen_error(
                "Module verification failed".to_string(),
                None,
            ));
        }

        Ok(())
    }

    /// Build an alloca instruction.
    pub fn create_alloca(&self, name: &str, ty: BasicTypeEnum) -> BasicValueEnum {
        // Get the state with a scoped lock
        let state = self.state.lock().unwrap();

        // Create alloca at the entry of the function
        if let Some(function) = state.current_function {
            // Get builder from context
            let builder = self.context.llvm_context.builder();
            let entry_block = function.get_first_basic_block().unwrap();
            let current_block = builder.get_insert_block().unwrap();

            // Move to the first instruction or the start of the block
            builder.position_at(
                entry_block,
                &entry_block.get_first_instruction().unwrap_or_else(|| {
                    // If there are no instructions yet, add a temporary one to position at
                    let temp = builder
                        .build_alloca(ty, "temp_positioning")
                        .expect("Failed to build temporary alloca for positioning");
                    builder.build_free(temp).expect("Failed to build free instruction");
                    entry_block.get_last_instruction().unwrap()
                }),
            );

            // Create the alloca
            let alloca = builder.build_alloca(ty, name);

            // Move back to where we were
            builder.position_at_end(current_block);

            return alloca.unwrap().into();
        }

        panic!("Cannot create alloca outside of a function");
    }

    /// Create a global string constant.
    pub fn create_global_string(&self, string: &str, name: &str) -> BasicValueEnum {
        // Get builder directly from our context
        let builder = self.context.llvm_context.builder();

        // Create the global string
        builder
            .build_global_string_ptr(string, name)
            .expect("Failed to build global string pointer")
            .as_basic_value_enum()
    }

    /// Get the current function being generated.
    pub fn current_function(&self) -> Option<FunctionValue> {
        let state = self.state.lock().expect("Failed to lock state");
        state.current_function
    }

    /// Set the current function being generated.
    pub fn set_current_function(&mut self, function: Option<FunctionValue>) {
        let mut state = self.state.lock().expect("Failed to lock state");
        state.current_function = function;
    }

    /// Visit a binary operation.
    fn visit_binary_op(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> CodeGenResult<CodeGenValue> {
        self.visitor.visit_binary_op(context, symbol_table, left, op, right)
    }

    /// Visit an expression node.
    fn visit_expression(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        expr: &Expression,
    ) -> CodeGenResult<CodeGenValue> {
        self.visitor.visit_expression(context, symbol_table, expr)
    }

    /// Visit a literal node.
    fn visit_literal(
        &mut self,
        context: &mut CodeGenContext,
        lit: &Literal,
    ) -> CodeGenResult<CodeGenValue> {
        self.visitor.visit_literal(context, lit)
    }

    /// Visit a module node.
    fn visit_module(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        module: &Module,
    ) -> CodeGenResult<()> {
        self.visitor.visit_module(context, symbol_table, module)
    }

    /// Visit a statement node.
    fn visit_statement(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        stmt: &Statement,
    ) -> CodeGenResult<CodeGenValue> {
        self.visitor.visit_statement(context, symbol_table, stmt)
    }

    /// Visit a type expression node.
    fn visit_type_expression(
        &mut self,
        context: &mut CodeGenContext,
        type_expr: &TypeExpression,
    ) -> CodeGenResult<()> {
        self.visitor.visit_type_expression(context, type_expr)
    }

    /// Visit a unary operation.
    fn visit_unary_op(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> CodeGenResult<CodeGenValue> {
        self.visitor.visit_unary_op(context, symbol_table, op, operand)
    }

    /// Visit a variable reference.
    fn visit_variable(
        &mut self,
        context: &mut CodeGenContext,
        symbol_table: &mut SymbolTable,
        name: &Identifier,
    ) -> CodeGenResult<CodeGenValue> {
        self.visitor.visit_variable(context, symbol_table, name)
    }
}
