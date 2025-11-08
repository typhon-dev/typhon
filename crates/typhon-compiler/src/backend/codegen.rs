use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::{
    AnyTypeEnum,
    BasicType,
    BasicTypeEnum,
};
use inkwell::values::{
    AnyValueEnum,
    BasicValue,
    BasicValueEnum,
    FunctionValue,
};

use crate::backend::error::{
    CodeGenError,
    CodeGenResult,
};
use crate::backend::llvm::LLVMContext;
use crate::frontend::ast::visitor::Visitor;
use crate::frontend::ast::{
    BinaryOperator,
    Expression,
    Identifier,
    Literal,
    Statement,
    TypeExpression,
    UnaryOperator,
};
use crate::frontend::lexer::token::SourceInfo;
use crate::typesystem::types::{
    TypeId,
    TyphonType,
};

/// Represents the result of code generation for an AST node.
#[derive(Debug)]
pub enum CodeGenValue {
    /// A basic LLVM value (integer, float, pointer, etc.).
    Basic(BasicValueEnum<'static>),
    /// A function value.
    Function(FunctionValue<'static>),
    /// No value (void).
    Void,
}

impl CodeGenValue {
    /// Convert to a basic value, returns an error if this is not a basic value.
    pub fn as_basic_value(&self) -> CodeGenResult<BasicValueEnum<'static>> {
        match self {
            CodeGenValue::Basic(value) => Ok(*value),
            _ => Err(CodeGenError::code_gen_error(
                "Expected a basic value".to_string(),
                None,
            )),
        }
    }

    /// Convert to a function value, returns an error if this is not a function value.
    pub fn as_function_value(&self) -> CodeGenResult<FunctionValue<'static>> {
        match self {
            CodeGenValue::Function(value) => Ok(*value),
            _ => Err(CodeGenError::code_gen_error(
                "Expected a function value".to_string(),
                None,
            )),
        }
    }
}

/// A symbol entry in the symbol table.
#[derive(Debug)]
struct SymbolEntry {
    /// The LLVM value representing the variable.
    pub value: BasicValueEnum<'static>,
    /// The type of the variable.
    pub ty: Rc<TyphonType>,
    /// Whether the variable is mutable.
    pub mutable: bool,
}

/// A symbol table for tracking variables in scope.
#[derive(Debug)]
pub struct SymbolTable {
    /// Nested scopes, with the last one being the current scope.
    scopes: Vec<HashMap<String, SymbolEntry>>,
}

impl SymbolTable {
    /// Create a new symbol table.
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());
        SymbolTable { scopes }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add a symbol to the current scope.
    pub fn add_symbol(&mut self, name: String, entry: SymbolEntry) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, entry);
        }
    }

    /// Look up a symbol in the current scope chain.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        // Look in scopes from inner to outer
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }
}

/// Code generator for Typhon AST.
pub struct CodeGenerator {
    /// The LLVM context.
    context: Rc<RefCell<LLVMContext>>,
    /// The symbol table.
    symbol_table: RefCell<SymbolTable>,
    /// The current function being generated.
    current_function: Option<FunctionValue<'static>>,
}

impl CodeGenerator {
    /// Create a new code generator.
    pub fn new(context: Rc<RefCell<LLVMContext>>) -> Self {
        CodeGenerator {
            context,
            symbol_table: RefCell::new(SymbolTable::new()),
            current_function: None,
        }
    }

    /// Build an alloca instruction.
    fn create_alloca(&self, name: &str, ty: BasicTypeEnum<'static>) -> BasicValueEnum<'static> {
        let context = self.context.borrow();
        let builder = context.builder();

        // Create alloca at the entry of the function
        if let Some(function) = self.current_function {
            let entry_block = function.get_first_basic_block().unwrap();

            // Move to the first instruction or the start of the block
            let current_block = builder.get_insert_block().unwrap();
            builder.position_at_start(entry_block);

            // Create the alloca
            let alloca = builder.build_alloca(ty, name);

            // Move back to where we were
            builder.position_at_end(current_block);

            return alloca.into();
        }

        panic!("Cannot create alloca outside of a function");
    }

    /// Build a load instruction.
    fn build_load(&self, ptr: BasicValueEnum<'static>, name: &str) -> BasicValueEnum<'static> {
        let context = self.context.borrow();
        let builder = context.builder();

        // Check if the pointer is actually a pointer type
        if let Some(ptr_value) = ptr.as_pointer_value() {
            // Get the type of the pointed-to value
            let pointee_type = ptr_value.get_type().get_element_type();

            // Build the load based on the pointee type
            match pointee_type {
                AnyTypeEnum::IntType(_) => builder
                    .build_load(pointee_type.into_int_type(), ptr_value, name)
                    .into(),
                AnyTypeEnum::FloatType(_) => builder
                    .build_load(pointee_type.into_float_type(), ptr_value, name)
                    .into(),
                AnyTypeEnum::PointerType(_) => builder
                    .build_load(pointee_type.into_pointer_type(), ptr_value, name)
                    .into(),
                AnyTypeEnum::StructType(_) => builder
                    .build_load(pointee_type.into_struct_type(), ptr_value, name)
                    .into(),
                AnyTypeEnum::ArrayType(_) => builder
                    .build_load(pointee_type.into_array_type(), ptr_value, name)
                    .into(),
                _ => panic!("Unsupported pointee type for load: {:?}", pointee_type),
            }
        } else {
            panic!("Expected pointer value for load, got: {:?}", ptr);
        }
    }

    /// Build a store instruction.
    fn build_store(&self, ptr: BasicValueEnum<'static>, value: BasicValueEnum<'static>) -> () {
        let context = self.context.borrow();
        let builder = context.builder();

        if let Some(ptr_value) = ptr.as_pointer_value() {
            builder.build_store(ptr_value, value);
        } else {
            panic!("Expected pointer value for store, got: {:?}", ptr);
        }
    }

    /// Create a global string constant.
    fn create_global_string(&self, string: &str, name: &str) -> BasicValueEnum<'static> {
        let context = self.context.borrow();
        let builder = context.builder();

        // Create a global string constant
        builder
            .build_global_string_ptr(string, name)
            .as_pointer_value()
            .into()
    }

    /// Build a binary operation.
    fn build_binary_op(
        &self,
        op: BinaryOperator,
        left: BasicValueEnum<'static>,
        right: BasicValueEnum<'static>,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum<'static>> {
        let context = self.context.borrow();
        let builder = context.builder();

        // Ensure both operands are of the same type
        if left.get_type() != right.get_type() {
            return Err(CodeGenError::code_gen_error(
                format!(
                    "Mismatched operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ));
        }

        // Handle integer operations
        if let (Some(left_int), Some(right_int)) = (left.as_int_value(), right.as_int_value()) {
            match op {
                BinaryOperator::Add => Ok(builder.build_int_add(left_int, right_int, name).into()),
                BinaryOperator::Sub => Ok(builder.build_int_sub(left_int, right_int, name).into()),
                BinaryOperator::Mul => Ok(builder.build_int_mul(left_int, right_int, name).into()),
                BinaryOperator::Div => Ok(builder
                    .build_int_signed_div(left_int, right_int, name)
                    .into()),
                BinaryOperator::Mod => Ok(builder
                    .build_int_signed_rem(left_int, right_int, name)
                    .into()),

                BinaryOperator::Eq => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::EQ, left_int, right_int, name)
                    .into()),
                BinaryOperator::Ne => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::NE, left_int, right_int, name)
                    .into()),
                BinaryOperator::Lt => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SLT, left_int, right_int, name)
                    .into()),
                BinaryOperator::Le => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SLE, left_int, right_int, name)
                    .into()),
                BinaryOperator::Gt => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SGT, left_int, right_int, name)
                    .into()),
                BinaryOperator::Ge => Ok(builder
                    .build_int_compare(inkwell::IntPredicate::SGE, left_int, right_int, name)
                    .into()),

                BinaryOperator::BitAnd => Ok(builder.build_and(left_int, right_int, name).into()),
                BinaryOperator::BitOr => Ok(builder.build_or(left_int, right_int, name).into()),
                BinaryOperator::BitXor => Ok(builder.build_xor(left_int, right_int, name).into()),
                BinaryOperator::LShift => {
                    Ok(builder.build_left_shift(left_int, right_int, name).into())
                }
                BinaryOperator::RShift => Ok(builder
                    .build_right_shift(left_int, right_int, true, name)
                    .into()),

                _ => Err(CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation: {:?}", op),
                    None,
                )),
            }
        }
        // Handle float operations
        else if let (Some(left_float), Some(right_float)) =
            (left.as_float_value(), right.as_float_value())
        {
            match op {
                BinaryOperator::Add => Ok(builder
                    .build_float_add(left_float, right_float, name)
                    .into()),
                BinaryOperator::Sub => Ok(builder
                    .build_float_sub(left_float, right_float, name)
                    .into()),
                BinaryOperator::Mul => Ok(builder
                    .build_float_mul(left_float, right_float, name)
                    .into()),
                BinaryOperator::Div => Ok(builder
                    .build_float_div(left_float, right_float, name)
                    .into()),

                BinaryOperator::Eq => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OEQ,
                        left_float,
                        right_float,
                        name,
                    )
                    .into()),
                BinaryOperator::Ne => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        left_float,
                        right_float,
                        name,
                    )
                    .into()),
                BinaryOperator::Lt => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OLT,
                        left_float,
                        right_float,
                        name,
                    )
                    .into()),
                BinaryOperator::Le => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OLE,
                        left_float,
                        right_float,
                        name,
                    )
                    .into()),
                BinaryOperator::Gt => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OGT,
                        left_float,
                        right_float,
                        name,
                    )
                    .into()),
                BinaryOperator::Ge => Ok(builder
                    .build_float_compare(
                        inkwell::FloatPredicate::OGE,
                        left_float,
                        right_float,
                        name,
                    )
                    .into()),

                _ => Err(CodeGenError::unsupported_feature(
                    format!("Unsupported binary operation for float: {:?}", op),
                    None,
                )),
            }
        } else {
            Err(CodeGenError::unsupported_feature(
                format!(
                    "Unsupported operand types for binary operation: {:?} and {:?}",
                    left.get_type(),
                    right.get_type()
                ),
                None,
            ))
        }
    }

    /// Build a unary operation.
    fn build_unary_op(
        &self,
        op: UnaryOperator,
        operand: BasicValueEnum<'static>,
        name: &str,
    ) -> CodeGenResult<BasicValueEnum<'static>> {
        let context = self.context.borrow();
        let builder = context.builder();

        // Handle integer operations
        if let Some(int_operand) = operand.as_int_value() {
            match op {
                UnaryOperator::Neg => {
                    // For integers, negation is 0 - operand
                    let zero = int_operand.get_type().const_int(0, false);
                    Ok(builder.build_int_sub(zero, int_operand, name).into())
                }
                UnaryOperator::Not => {
                    // For integers, logical not is comparison with zero
                    let zero = int_operand.get_type().const_int(0, false);
                    Ok(builder
                        .build_int_compare(inkwell::IntPredicate::EQ, int_operand, zero, name)
                        .into())
                }
                UnaryOperator::BitNot => {
                    // Bitwise not is XOR with all ones
                    let all_ones = int_operand.get_type().const_all_ones();
                    Ok(builder.build_xor(int_operand, all_ones, name).into())
                }
            }
        }
        // Handle float operations
        else if let Some(float_operand) = operand.as_float_value() {
            match op {
                UnaryOperator::Neg => Ok(builder.build_float_neg(float_operand, name).into()),
                UnaryOperator::Not => {
                    // For floats, logical not is comparison with zero
                    let zero = float_operand.get_type().const_float(0.0);
                    Ok(builder
                        .build_float_compare(
                            inkwell::FloatPredicate::OEQ,
                            float_operand,
                            zero,
                            name,
                        )
                        .into())
                }
                _ => Err(CodeGenError::unsupported_feature(
                    format!("Unsupported unary operation for float: {:?}", op),
                    None,
                )),
            }
        } else {
            Err(CodeGenError::unsupported_feature(
                format!(
                    "Unsupported operand type for unary operation: {:?}",
                    operand.get_type()
                ),
                None,
            ))
        }
    }

    // Code generation for a function declaration
    fn gen_function(&mut self, func: &Statement) -> CodeGenResult<FunctionValue<'static>> {
        if let Statement::FunctionDef {
            name,
            parameters,
            return_type,
            body,
            ..
        } = func
        {
            let context = self.context.borrow();
            let llvm_context = context.context();
            let module = context.module();

            // Get the function name
            let name = &name.name;

            // Create parameter types
            let param_types: Vec<_> = parameters
                .iter()
                .map(|param| {
                    // Simplify by using i64 for all parameters for now
                    llvm_context.i64_type().into()
                })
                .collect();

            // Create the function type
            // Simplify by using i64 as the return type for now
            let return_type = llvm_context.i64_type();
            let function_type = return_type.fn_type(&param_types, false);

            // Create the function
            let function = module.add_function(name, function_type, None);

            // Create a basic block
            let basic_block = llvm_context.append_basic_block(function, "entry");

            // Set the current function
            self.current_function = Some(function);

            // Position the builder at the end of the basic block
            let builder = context.builder();
            builder.position_at_end(basic_block);

            // Push a new scope for the function parameters
            self.symbol_table.borrow_mut().push_scope();

            // Add parameters to the symbol table
            for (i, param) in parameters.iter().enumerate() {
                let param_value = function.get_nth_param(i as u32).unwrap();
                let param_name = &param.name.name;

                // Create an alloca for the parameter
                let param_alloca = self.create_alloca(param_name, param_value.get_type());

                // Store the parameter value
                self.build_store(param_alloca, param_value.into());

                // Add to the symbol table
                let param_entry = SymbolEntry {
                    value: param_alloca,
                    ty: Rc::new(TyphonType::Any), // Simplified type handling
                    mutable: true,
                };

                self.symbol_table
                    .borrow_mut()
                    .add_symbol(param_name.clone(), param_entry);
            }

            // Generate code for the function body
            for stmt in body {
                self.visit_statement(stmt)?;
            }

            // If there's no return statement, add a default return value
            if !builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                // For now, just return 0 for any function
                let return_value = return_type.const_int(0, false);
                builder.build_return(Some(&return_value));
            }

            // Verify the function
            if function.verify(true) {
                Ok(function)
            } else {
                // Pop the scope before returning
                self.symbol_table.borrow_mut().pop_scope();

                Err(CodeGenError::code_gen_error(
                    format!("Invalid function: {}", name),
                    Some(name.source_info.span),
                ))
            }
        } else {
            Err(CodeGenError::code_gen_error(
                "Expected a function definition statement".to_string(),
                None,
            ))
        }
    }

    /// Complete variable declaration statement.
    pub fn complete_variable_decl_stmt(
        &mut self,
        name: &Identifier,
        type_annotation: &Option<TypeExpression>,
        value: &Option<Box<Expression>>,
        mutable: bool,
    ) -> CodeGenResult<()> {
        // Get the LLVM type based on the type annotation or infer from the value
        let context = self.context.borrow();
        let llvm_type = match type_annotation {
            Some(ty) => {
                // Convert the Typhon type to an LLVM type
                // This is a simplified implementation
                context.context().i64_type().into()
            }
            None => {
                if let Some(val) = value {
                    // Infer the type from the value
                    // This is a simplified implementation
                    context.context().i64_type().into()
                } else {
                    // Default to i64 if no type or value is provided
                    context.context().i64_type().into()
                }
            }
        };

        // Create an alloca for the variable
        let alloca = self.create_alloca(&name.name, llvm_type);

        // If there's an initial value, generate code for it and store it
        if let Some(val) = value {
            let init_value = self.visit_expression(val)?.as_basic_value()?;
            self.build_store(alloca, init_value);
        }

        // Add the variable to the symbol table
        let entry = SymbolEntry {
            value: alloca,
            ty: Rc::new(TyphonType::Any), // Simplified type handling
            mutable,
        };

        self.symbol_table
            .borrow_mut()
            .add_symbol(name.name.clone(), entry);

        Ok(())
    }

    /// Visit an expression.
    fn visit_expression(&mut self, expr: &Expression) -> CodeGenResult<CodeGenValue> {
        match expr {
            Expression::Literal { value, source_info } => self.visit_literal(value, source_info),
            Expression::BinaryOp {
                left,
                op,
                right,
                source_info,
            } => {
                let left_value = self.visit_expression(left)?.as_basic_value()?;
                let right_value = self.visit_expression(right)?.as_basic_value()?;

                // Binary operation implementation
                let result = self.build_binary_op(*op, left_value, right_value, "binop")?;
                Ok(CodeGenValue::Basic(result))
            }
            Expression::UnaryOp {
                op,
                operand,
                source_info,
            } => {
                let operand_value = self.visit_expression(operand)?.as_basic_value()?;

                // Unary operation implementation
                let result = self.build_unary_op(*op, operand_value, "unop")?;
                Ok(CodeGenValue::Basic(result))
            }
            Expression::Variable { name, source_info } => {
                // Variable lookup implementation
                let entry = self
                    .symbol_table
                    .borrow()
                    .lookup(&name.name)
                    .ok_or_else(|| {
                        CodeGenError::code_gen_error(
                            format!("Undefined variable: {}", name.name),
                            Some(source_info.span),
                        )
                    })?;

                // Load the variable value
                let value = self.build_load(entry.value, &name.name);
                Ok(CodeGenValue::Basic(value))
            }
            // Placeholder for other expression types
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported expression type: {:?}", expr),
                None,
            )),
        }
    }

    /// Visit a literal.
    fn visit_literal(
        &mut self,
        literal: &Literal,
        source_info: &SourceInfo,
    ) -> CodeGenResult<CodeGenValue> {
        let context = self.context.borrow();

        match literal {
            Literal::Int(i) => {
                let int_type = context.context().i64_type();
                let value = int_type.const_int(*i as u64, true);
                Ok(CodeGenValue::Basic(value.into()))
            }
            Literal::Float(f) => {
                let float_type = context.context().f64_type();
                let value = float_type.const_float(*f);
                Ok(CodeGenValue::Basic(value.into()))
            }
            Literal::String(s) => {
                let value = self.create_global_string(s, "str");
                Ok(CodeGenValue::Basic(value))
            }
            Literal::Bool(b) => {
                let bool_type = context.context().bool_type();
                let value = bool_type.const_int(*b as u64, false);
                Ok(CodeGenValue::Basic(value.into()))
            }
            // Placeholder for other literal types
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported literal type: {:?}", literal),
                Some(source_info.span),
            )),
        }
    }
}

// Visitor trait implementation for the code generator
impl<'ast> Visitor<'ast, CodeGenResult<CodeGenValue>> for CodeGenerator {
    fn visit_module(
        &mut self,
        module: &'ast crate::frontend::ast::Module,
    ) -> CodeGenResult<CodeGenValue> {
        // Process each statement in the module
        for stmt in &module.statements {
            self.visit_statement(stmt)?;
        }
        Ok(CodeGenValue::Void)
    }

    fn visit_expression(&mut self, expr: &'ast Expression) -> CodeGenResult<CodeGenValue> {
        // We already have an implementation of this method in the CodeGenerator class
        // Call that implementation here
        CodeGenerator::visit_expression(self, expr)
    }

    fn visit_type_expression(
        &mut self,
        type_expr: &'ast TypeExpression,
    ) -> CodeGenResult<CodeGenValue> {
        // Type expressions don't generate code directly, just return Void
        Ok(CodeGenValue::Void)
    }

    fn visit_literal(&mut self, lit: &'ast Literal) -> CodeGenResult<CodeGenValue> {
        // Simplified implementation just to make the Visitor trait happy
        // Visit_literal takes a source_info parameter as well, which we don't have here
        // Use a dummy SourceInfo
        let dummy_source = SourceInfo { span: 0..0 };
        CodeGenerator::visit_literal(self, lit, &dummy_source)
    }

    fn visit_statement(&mut self, stmt: &'ast Statement) -> CodeGenResult<CodeGenValue> {
        match stmt {
            Statement::FunctionDecl(func) => {
                let function = self.gen_function(func)?;
                Ok(CodeGenValue::Function(function))
            }
            Statement::Return { value, .. } => {
                let context = self.context.borrow();
                let builder = context.builder();

                if let Some(expr) = value {
                    // Evaluate the expression
                    let value_result = self.visit_expression(expr)?;
                    let return_value = value_result.as_basic_value()?;

                    // Build the return instruction
                    builder.build_return(Some(&return_value));
                } else {
                    // Return void
                    builder.build_return(None);
                }

                Ok(CodeGenValue::Void)
            }
            Statement::VariableDecl {
                name,
                type_annotation,
                value,
                mutable,
                ..
            } => {
                // Implement the variable declaration
                self.complete_variable_decl_stmt(name, type_annotation, value, *mutable)?;
                Ok(CodeGenValue::Void)
            }
            Statement::Assignment { target, value, .. } => {
                // Evaluate the target
                match target {
                    Expression::Variable { name, .. } => {
                        // Look up the variable in the symbol table
                        let entry =
                            self.symbol_table
                                .borrow()
                                .lookup(&name.name)
                                .ok_or_else(|| {
                                    CodeGenError::code_gen_error(
                                        format!("Undefined variable: {}", name.name),
                                        Some(name.source_info.span),
                                    )
                                })?;

                        // Check if the variable is mutable
                        if !entry.mutable {
                            return Err(CodeGenError::code_gen_error(
                                format!("Cannot assign to immutable variable: {}", name.name),
                                Some(name.source_info.span),
                            ));
                        }

                        // Evaluate the value
                        let value_result = self.visit_expression(value)?;
                        let value_basic = value_result.as_basic_value()?;

                        // Store the value
                        self.build_store(entry.value, value_basic);

                        Ok(CodeGenValue::Void)
                    }
                    _ => Err(CodeGenError::unsupported_feature(
                        format!("Unsupported assignment target: {:?}", target),
                        None,
                    )),
                }
            }
            Statement::ExprStmt { expr, .. } => {
                // Evaluate the expression and discard the result
                self.visit_expression(expr)?;
                Ok(CodeGenValue::Void)
            }
            _ => Err(CodeGenError::unsupported_feature(
                format!("Unsupported statement type: {:?}", stmt),
                None,
            )),
        }
    }
}

// Implement conversion between Typhon types and LLVM types
impl<'a> CodeGenerator {
    /// Convert a Typhon type to an LLVM type.
    pub fn get_llvm_type(&self, ty: &TyphonType) -> BasicTypeEnum<'static> {
        let context = self.context.borrow();
        let llvm_context = context.context();

        match ty {
            TyphonType::Int => llvm_context.i64_type().into(),
            TyphonType::Float => llvm_context.f64_type().into(),
            TyphonType::Bool => llvm_context.bool_type().into(),
            TyphonType::Str => llvm_context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            TyphonType::Bytes => llvm_context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            TyphonType::None => llvm_context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            TyphonType::Any => llvm_context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            _ => {
                // Default to a void pointer for complex types
                llvm_context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
        }
    }
}

// Compilation entry point
impl CodeGenerator {
    /// Compile an AST to LLVM IR.
    pub fn compile(&mut self, statements: &[Statement]) -> CodeGenResult<()> {
        // Process each top-level statement
        for stmt in statements {
            self.visit_statement(stmt)?;
        }

        // Verify the module
        let context = self.context.borrow();
        if context.module().verify().is_err() {
            return Err(CodeGenError::code_gen_error(
                "Module verification failed".to_string(),
                None,
            ));
        }

        Ok(())
    }
}
