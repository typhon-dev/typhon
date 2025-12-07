//! Declarative macros for reducing boilerplate in AST node implementations.
//!
//! This module contains macros that generate repetitive trait implementations
//! for the `AnyNode` enum and related types. These macros significantly reduce
//! code duplication and make it easier to add new node types.
//!
//! ## Design Philosophy
//!
//! Rather than manually writing hundreds of match arms for each trait implementation,
//! we use a master list of node variants and generate all implementations from it.
//! This approach provides several benefits:
//!
//! - **Single Source of Truth**: All node variants are defined in one place
//! - **Type Safety**: The compiler ensures all variants are handled
//! - **Maintainability**: Adding a new node type requires updating only the master list
//! - **Zero Runtime Cost**: All macros expand at compile time
//!
//! ## Usage
//!
//! The main macro is `for_each_node_variant!`, which invokes a callback macro
//! for each node variant in the AST. Other macros use this to generate their
//! implementations.

/// Master list of all AST node variants.
///
/// This macro defines the complete list of node variants in the AST, along with
/// their associated types and visitor method names. It serves as the single source
/// of truth for all trait implementations.
///
/// ## Format
///
/// Each line follows the pattern:
/// ```text
/// EnumVariant(Type) => visitor_method_name,
/// ```
///
/// ## Adding a New Node Type
///
/// To add a new node type to the AST:
///
/// 1. Add the node's struct definition to the appropriate module
/// 2. Add a new line to this macro following the pattern above
/// 3. Recompile - all trait implementations will be automatically updated
///
/// ## Example
///
/// ```ignore
/// for_each_node_variant!(my_callback_macro);
/// ```
///
/// The callback macro will be invoked with all node variants as arguments.
#[macro_export]
macro_rules! for_each_node_variant {
    ($callback:ident) => {
        $callback! {
            ArgumentExpr(ArgumentExpr) => visit_argument_expr,
            AsPattern(AsPattern) => visit_as_pattern,
            AssertStmt(AssertStmt) => visit_assert_stmt,
            AssignmentExpr(AssignmentExpr) => visit_assignment_expr,
            AssignmentStmt(AssignmentStmt) => visit_assignment_stmt,
            AsyncForStmt(AsyncForStmt) => visit_async_for_stmt,
            AsyncFunctionDecl(AsyncFunctionDecl) => visit_async_function_decl,
            AsyncWithStmt(AsyncWithStmt) => visit_async_with_stmt,
            AttributeExpr(AttributeExpr) => visit_attribute_expr,
            AugmentedAssignmentStmt(AugmentedAssignmentStmt) => visit_augmented_assignment_stmt,
            AwaitExpr(AwaitExpr) => visit_await_expr,
            BinaryOpExpr(BinaryOpExpr) => visit_binary_op_expr,
            BreakStmt(BreakStmt) => visit_break_stmt,
            CallExpr(CallExpr) => visit_call_expr,
            CallableType(CallableType) => visit_callable_type,
            ClassDecl(ClassDecl) => visit_class_decl,
            ClassPattern(ClassPattern) => visit_class_pattern,
            BasicIdent(BasicIdent) => visit_basic_ident,
            ContinueStmt(ContinueStmt) => visit_continue_stmt,
            DeleteStmt(DeleteStmt) => visit_delete_stmt,
            DictExpr(DictExpr) => visit_dict_expr,
            DictComprehensionExpr(DictComprehensionExpr) => visit_dict_comprehension_expr,
            ExceptHandler(ExceptHandler) => visit_except_handler,
            ExpressionStmt(ExpressionStmt) => visit_expression_stmt,
            FmtStringExpr(FmtStringExpr) => visit_fmt_string_expr,
            ForStmt(ForStmt) => visit_for_stmt,
            FromImportStmt(FromImportStmt) => visit_from_import_stmt,
            FunctionDecl(FunctionDecl) => visit_function_decl,
            GeneratorExpr(GeneratorExpr) => visit_generator_expr,
            GenericType(GenericType) => visit_generic_type,
            GlobalStmt(GlobalStmt) => visit_global_stmt,
            GroupingExpr(GroupingExpr) => visit_grouping_expr,
            IdentifierPattern(IdentifierPattern) => visit_identifier_pattern,
            IfStmt(IfStmt) => visit_if_stmt,
            ImportStmt(ImportStmt) => visit_import_stmt,
            LambdaExpr(LambdaExpr) => visit_lambda_expr,
            ListExpr(ListExpr) => visit_list_expr,
            ListComprehensionExpr(ListComprehensionExpr) => visit_list_comprehension_expr,
            LiteralExpr(LiteralExpr) => visit_literal_expr,
            LiteralPattern(LiteralPattern) => visit_literal_pattern,
            LiteralType(LiteralType) => visit_literal_type,
            MappingPattern(MappingPattern) => visit_mapping_pattern,
            MatchStmt(MatchStmt) => visit_match_stmt,
            MatchCase(MatchCase) => visit_match_case,
            Module(Module) => visit_module,
            NonlocalStmt(NonlocalStmt) => visit_nonlocal_stmt,
            OrPattern(OrPattern) => visit_or_pattern,
            ParameterIdent(ParameterIdent) => visit_parameter_ident,
            PassStmt(PassStmt) => visit_pass_stmt,
            RaiseStmt(RaiseStmt) => visit_raise_stmt,
            ReturnStmt(ReturnStmt) => visit_return_stmt,
            SequencePattern(SequencePattern) => visit_sequence_pattern,
            SetExpr(SetExpr) => visit_set_expr,
            SetComprehensionExpr(SetComprehensionExpr) => visit_set_comprehension_expr,
            SliceExpr(SliceExpr) => visit_slice_expr,
            StarredExpr(StarredExpr) => visit_starred_expr,
            SubscriptionExpr(SubscriptionExpr) => visit_subscription_expr,
            TemplateStringExpr(TemplateStringExpr) => visit_template_string_expr,
            TernaryExpr(TernaryExpr) => visit_ternary_expr,
            TryStmt(TryStmt) => visit_try_stmt,
            TupleExpr(TupleExpr) => visit_tuple_expr,
            TupleType(TupleType) => visit_tuple_type,
            TypeDecl(TypeDecl) => visit_type_decl,
            UnaryOpExpr(UnaryOpExpr) => visit_unary_op_expr,
            UnionType(UnionType) => visit_union_type,
            VariableDecl(VariableDecl) => visit_variable_decl,
            VariableExpr(VariableExpr) => visit_variable_expr,
            WhileStmt(WhileStmt) => visit_while_stmt,
            WildcardPattern(WildcardPattern) => visit_wildcard_pattern,
            WithStmt(WithStmt) => visit_with_stmt,
            YieldExpr(YieldExpr) => visit_yield_expr,
            YieldFromExpr(YieldFromExpr) => visit_yield_from_expr,
        }
    };
}

/// Generates the `ASTNode` trait implementation for `AnyNode`.
///
/// This macro creates all six methods of the `ASTNode` trait by generating
/// match expressions that delegate to the corresponding method on each variant's
/// inner type.
///
/// ## Generated Methods
///
/// - `id(&self) -> NodeID`
/// - `parent(&self) -> Option<NodeID>`
/// - `with_parent(self, parent: NodeID) -> Self`
/// - `kind(&self) -> NodeKind`
/// - `span(&self) -> Span`
/// - `children(&self) -> Vec<NodeID>`
///
/// ## Example
///
/// ```ignore
/// for_each_node_variant!(impl_astnode_for_anynode);
/// ```
///
/// This will generate approximately 490 lines of match arms from ~85 lines of macro code.
#[macro_export]
macro_rules! impl_astnode_for_anynode {
    ($($variant:ident($type:ty) => $visit:ident),* $(,)?) => {
        impl $crate::nodes::ASTNode for $crate::nodes::AnyNode {
            fn id(&self) -> $crate::nodes::NodeID {
                match self {
                    $(Self::$variant(node) => node.id(),)*
                }
            }

            fn parent(&self) -> Option<$crate::nodes::NodeID> {
                match self {
                    $(Self::$variant(node) => node.parent(),)*
                }
            }

            fn with_parent(self, parent: $crate::nodes::NodeID) -> Self {
                match self {
                    $(Self::$variant(node) => Self::$variant(node.with_parent(parent)),)*
                }
            }

            fn kind(&self) -> $crate::nodes::NodeKind {
                match self {
                    $(Self::$variant(node) => node.kind(),)*
                }
            }

            fn span(&self) -> typhon_source::types::Span {
                match self {
                    $(Self::$variant(node) => node.span(),)*
                }
            }

            fn children(&self) -> Vec<$crate::nodes::NodeID> {
                match self {
                    $(Self::$variant(node) => node.children(),)*
                }
            }
        }
    };
}

/// Generates the `Visitable` trait implementation for `AnyNode`.
///
/// This macro creates both the `accept` and `accept_mut` methods that dispatch
/// to the appropriate visitor method based on the node's variant type.
///
/// ## Generated Methods
///
/// - `accept<T>(&self, visitor: &mut dyn Visitor<T>, node_id: NodeID) -> VisitorResult<T>`
/// - `accept_mut<T>(&self, visitor: &mut dyn MutVisitor<T>, node_id: NodeID) -> VisitorResult<T>`
///
/// ## Example
///
/// ```ignore
/// for_each_node_variant!(impl_visitable_for_anynode);
/// ```
///
/// This will generate approximately 160 lines of match arms from ~40 lines of macro code.
#[macro_export]
macro_rules! impl_visitable_for_anynode {
    ($($variant:ident($type:ty) => $visit:ident),* $(,)?) => {
        impl $crate::visitor::Visitable for $crate::nodes::AnyNode {
            fn accept<T>(
                &self,
                visitor: &mut dyn $crate::visitor::Visitor<T>,
                node_id: $crate::nodes::NodeID,
            ) -> $crate::visitor::VisitorResult<T> {
                match self {
                    $(Self::$variant(_) => visitor.$visit(node_id),)*
                }
            }

            fn accept_mut<T>(
                &self,
                visitor: &mut dyn $crate::visitor::MutVisitor<T>,
                node_id: $crate::nodes::NodeID,
            ) -> $crate::visitor::VisitorResult<T> {
                match self {
                    $(Self::$variant(_) => visitor.$visit(node_id),)*
                }
            }
        }
    };
}

/// Generates a complete `Visitable` trait implementation for a concrete node type.
///
/// This macro creates both the `accept` and `accept_mut` methods that call the appropriate
/// visitor method for the specific node type.
///
/// ## Usage
///
/// ```ignore
/// impl_visitable!(TypeName, visit_method_name);
/// ```
///
/// ## Example
///
/// ```ignore
/// impl_visitable!(Variable, visit_variable);
/// ```
///
/// This expands to:
///
/// ```ignore
/// impl Visitable for Variable {
///     fn accept<T>(&self, visitor: &mut dyn Visitor<T>, node_id: NodeID) -> VisitorResult<T> {
///         visitor.visit_variable(node_id)
///     }
///
///     fn accept_mut<T>(&self, visitor: &mut dyn MutVisitor<T>, node_id: NodeID) -> VisitorResult<T> {
///         visitor.visit_variable(node_id)
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_visitable {
    ($type:ty, $method:ident) => {
        impl $crate::visitor::Visitable for $type {
            fn accept<T>(
                &self,
                visitor: &mut dyn $crate::visitor::Visitor<T>,
                node_id: $crate::nodes::NodeID,
            ) -> $crate::visitor::VisitorResult<T> {
                visitor.$method(node_id)
            }

            fn accept_mut<T>(
                &self,
                visitor: &mut dyn $crate::visitor::MutVisitor<T>,
                node_id: $crate::nodes::NodeID,
            ) -> $crate::visitor::VisitorResult<T> {
                visitor.$method(node_id)
            }
        }
    };
}

/// Generates the `Display` trait implementation for `AnyNode`.
///
/// This macro creates the `fmt` method that delegates to the inner type's
/// `Display` implementation for each variant.
///
/// ## Generated Method
///
/// - `fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
///
/// ## Example
///
/// ```ignore
/// for_each_node_variant!(impl_display_for_anynode);
/// ```
///
/// This will generate approximately 80 lines of match arms from ~15 lines of macro code.
#[macro_export]
macro_rules! impl_display_for_anynode {
    ($($variant:ident($type:ty) => $visit:ident),* $(,)?) => {
        impl std::fmt::Display for $crate::nodes::AnyNode {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant(node) => node.fmt(f),)*
                }
            }
        }
    };
}

/// Generates the complete `get_as<T>()` method implementation for `AnyNode`.
///
/// This macro creates a method that performs runtime type checking and safe
/// pointer casting to return strongly-typed references to specific node types.
///
/// ## Safety
///
/// While this macro uses `unsafe` code internally for pointer casting, it is
/// actually safe because:
///
/// 1. We verify the type matches using `type_name::<T>()` before casting
/// 2. The pointer is derived from a valid reference with sufficient lifetime
/// 3. The cast preserves the memory layout since we're casting to the exact type
///
/// ## Example
///
/// ```ignore
/// for_each_node_variant!(impl_get_as_for_anynode);
/// ```
///
/// This generates the complete `get_as<T>()` method for `AnyNode`.
#[macro_export]
macro_rules! impl_get_as_for_anynode {
    ($($variant:ident($type:ty) => $visit:ident),* $(,)?) => {
        impl $crate::nodes::AnyNode {
            /// Gets a strongly-typed reference to the inner node data.
            ///
            /// This method performs runtime type checking and returns a reference to the
            /// specific node type if the variant matches the requested type.
            ///
            /// ## Type Parameters
            ///
            /// - `T` - The specific node type to retrieve, such as `BinaryOpExpr`, `FunctionDecl`, etc.
            ///
            /// ## Returns
            ///
            /// A result containing a reference to the node of type `T`, or an error message
            /// if the type doesn't match.
            ///
            /// ## Example
            ///
            /// ```ignore
            /// let any_node: AnyNode = /* ... */;
            /// let binary_op: &BinaryOpExpr = any_node.get_as::<BinaryOpExpr>()?;
            /// ```
            ///
            /// ## Errors
            ///
            /// Returns an error if the node type doesn't match the requested type `T`.
            #[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
            pub fn get_as<T: 'static>(&self) -> Result<&T, String> {
                let expected_type = std::any::type_name::<T>();

                match self {
                    $(
                        Self::$variant(inner) if std::any::type_name::<$type>() == expected_type => {
                            // SAFETY: We've verified the type matches via type_name comparison.
                            // The pointer is derived from a valid reference with sufficient lifetime.
                            // The cast is safe because we're casting to the exact type we checked for.
                            Ok(unsafe { &*std::ptr::from_ref::<$type>(inner).cast::<T>() })
                        }
                    )*
                    _ => Err(format!("Type mismatch: expected {}, got {:?}", expected_type, self.kind())),
                }
            }
        }
    };
}
