//! AST arena allocator for efficient node management.
//!
//! This module provides the core AST arena that manages memory allocation
//! for all AST nodes using a bump allocator with generation-based safety.

use std::any::type_name;

use bumpalo::Bump;
use typhon_source::types::Span;

use crate::nodes::{
    ASTNode,
    AnyNode,
    ArgumentExpr,
    AsPattern,
    AssertStmt,
    AssignmentExpr,
    AssignmentStmt,
    AsyncForStmt,
    AsyncFunctionDecl,
    AsyncWithStmt,
    AttributeExpr,
    AugmentedAssignmentStmt,
    AwaitExpr,
    BasicIdent,
    BinaryOpExpr,
    BreakStmt,
    CallExpr,
    CallableType,
    ClassDecl,
    ClassPattern,
    ContinueStmt,
    DeleteStmt,
    DictComprehensionExpr,
    DictExpr,
    ExceptHandler,
    ExpressionStmt,
    FmtStringExpr,
    ForStmt,
    FromImportStmt,
    FunctionDecl,
    GeneratorExpr,
    GenericType,
    GlobalStmt,
    GroupingExpr,
    IdentifierPattern,
    IfStmt,
    ImportStmt,
    LambdaExpr,
    ListComprehensionExpr,
    ListExpr,
    LiteralExpr,
    LiteralPattern,
    LiteralType,
    MappingPattern,
    MatchCase,
    MatchStmt,
    Module,
    Node,
    NodeID,
    NodeKind,
    NonlocalStmt,
    OrPattern,
    ParameterIdent,
    PassStmt,
    RaiseStmt,
    ReturnStmt,
    SequencePattern,
    SetComprehensionExpr,
    SetExpr,
    StarredExpr,
    SubscriptionExpr,
    TemplateStringExpr,
    TernaryExpr,
    TryStmt,
    TupleExpr,
    TupleType,
    TypeDecl,
    UnaryOpExpr,
    UnionType,
    VariableDecl,
    VariableIdent,
    WhileStmt,
    WildcardPattern,
    WithStmt,
    YieldExpr,
    YieldFromExpr,
};
use crate::visitor::{Visitor, VisitorError, VisitorResult};

/// Metadata for a single slot in the node arena.
///
/// Tracks the generation counter and occupancy status for each slot.
/// The generation counter is incremented each time a slot is reused,
/// preventing use-after-free bugs when old `NodeID`s reference removed nodes.
#[derive(Debug, Clone, Copy)]
struct SlotMetadata {
    /// Generation counter for this slot (incremented on removal)
    generation: u32,
    /// Whether this slot currently contains a node
    occupied: bool,
}

impl SlotMetadata {
    /// Creates new slot metadata with generation 1 and the specified occupancy
    const fn new(occupied: bool) -> Self { Self { generation: 1, occupied } }

    /// Increments the generation counter (called when slot is freed)
    const fn increment_generation(&mut self) { self.generation = self.generation.wrapping_add(1); }
}

/// An arena for allocating AST nodes.
///
/// The `AST` manages memory allocation for all AST nodes using a bump allocator.
/// It provides methods for allocating and accessing nodes with generation-based
/// safety to prevent use-after-free bugs.
#[derive(Debug)]
pub struct AST {
    /// The bump allocator for node allocation.
    allocator: Bump,
    /// Storage for nodes with associated metadata.
    nodes: Vec<Option<Node>>,
    /// Metadata for each slot (generation counter and occupancy status).
    metadata: Vec<SlotMetadata>,
    /// Free list for O(1) slot reuse (indices of freed slots).
    free_list: Vec<u32>,
    /// The root node of the AST, if any.
    root: Option<NodeID>,
}

impl AST {
    /// Creates a new empty AST arena.
    #[must_use]
    pub fn new() -> Self {
        Self {
            allocator: Bump::new(),
            nodes: Vec::new(),
            metadata: Vec::new(),
            free_list: Vec::new(),
            root: None,
        }
    }

    /// Creates a new AST arena with the given initial capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocator: Bump::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            metadata: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            root: None,
        }
    }

    /// Sets the root node of the AST.
    pub const fn set_root(&mut self, root: NodeID) { self.root = Some(root); }

    /// Returns the root node of the AST, if any.
    pub const fn root(&self) -> Option<NodeID> { self.root }

    /// Allocates a new node in the arena with O(1) slot allocation.
    ///
    /// Uses a free list to achieve constant-time allocation by reusing freed slots.
    /// Generation counters prevent use-after-free bugs.
    pub fn alloc_node(&mut self, kind: NodeKind, data: AnyNode, span: Span) -> NodeID {
        let (index, generation) = if let Some(free_index) = self.free_list.pop() {
            // Reuse a freed slot - use its current generation
            let metadata = &self.metadata[free_index as usize];

            (free_index, metadata.generation)
        } else {
            // No free slots - allocate a new one
            let index = self.nodes.len() as u32;
            self.nodes.push(None);
            self.metadata.push(SlotMetadata::new(true));

            (index, 1)
        };

        // Create and store the new node
        let node = Node { kind, data, span, parent: None };
        self.nodes[index as usize] = Some(node);
        self.metadata[index as usize].occupied = true;

        NodeID::new(index, generation)
    }

    /// Gets a reference to a node by its ID with generation validation.
    ///
    /// Returns None if the node doesn't exist or the generation doesn't match,
    /// preventing access to stale nodes that have been removed and reused.
    pub fn get_node(&self, id: NodeID) -> Option<&Node> {
        // Check bounds
        let index = id.index as usize;
        if index >= self.nodes.len() {
            return None;
        }

        // Validate generation counter
        if self.metadata[index].generation != id.generation {
            return None;
        }

        // Return the node if it exists
        self.nodes[index].as_ref()
    }

    /// Gets a mutable reference to a node by its ID with generation validation.
    ///
    /// Returns None if the node doesn't exist or the generation doesn't match,
    /// preventing access to stale nodes that have been removed and reused.
    pub fn get_node_mut(&mut self, id: NodeID) -> Option<&mut Node> {
        // Check bounds
        let index = id.index as usize;
        if index >= self.nodes.len() {
            return None;
        }

        // Validate generation counter
        if self.metadata[index].generation != id.generation {
            return None;
        }

        // Return the node if it exists
        self.nodes[index].as_mut()
    }

    /// Checks if a node exists in the arena.
    pub fn node_exists(&self, id: NodeID) -> bool {
        if id.index as usize >= self.nodes.len() {
            return false;
        }

        self.nodes[id.index as usize].is_some()
    }

    /// Sets the parent of a node.
    pub fn set_parent(&mut self, child: NodeID, parent: NodeID) -> bool {
        if let Some(Some(node)) = self.nodes.get_mut(child.index as usize) {
            node.parent = Some(parent);
            return true;
        }

        false
    }

    /// Gets the parent of a node, if any.
    pub fn get_parent(&self, id: NodeID) -> Option<NodeID> {
        self.nodes.get(id.index as usize)?.as_ref()?.parent
    }

    /// Allocates a string in the arena.
    pub fn alloc_str(&self, s: &str) -> &str { self.allocator.alloc_str(s) }

    /// Returns the number of nodes currently stored in the arena.
    pub fn node_count(&self) -> usize { self.nodes.iter().filter(|n| n.is_some()).count() }

    /// Removes a node from the arena with proper generation handling.
    ///
    /// Increments the generation counter and adds the slot to the free list,
    /// preventing old `NodeID`s from accessing the reused slot.
    pub fn remove(&mut self, id: NodeID) -> bool {
        let index = id.index as usize;

        // Check bounds
        if index >= self.nodes.len() {
            return false;
        }

        // Validate generation and check if node exists
        if self.metadata[index].generation != id.generation || self.nodes[index].is_none() {
            return false;
        }

        // Remove the node
        self.nodes[index] = None;

        // Increment generation counter to invalidate old NodeIDs
        self.metadata[index].increment_generation();
        self.metadata[index].occupied = false;

        // Add to free list for reuse
        self.free_list.push(id.index);

        true
    }

    /// Helper method for visitor pattern
    pub fn visit_as<T>(&mut self, node_id: NodeID) -> Option<T>
    where Self: Visitor<T> {
        <Self as Visitor<T>>::visit(self, node_id)
    }

    /// Performs a pre-order traversal of the AST starting from the given node.
    /// Pre-order traversal visits the current node first, then its children.
    ///
    /// ## Arguments
    ///
    /// * `node_id` - The ID of the node to start traversal from.
    /// * `visit_fn` - A function that is called for each node during traversal.
    ///
    /// ## Returns
    ///
    /// Returns true if the traversal completed successfully, false if it was aborted.
    pub fn traverse_pre_order<F>(&self, node_id: NodeID, visit_fn: &mut F) -> bool
    where F: FnMut(NodeID) -> bool {
        // Visit the current node first
        if !visit_fn(node_id) {
            return false; // Abort traversal if visitor returns false
        }

        // Get the node's children
        let Some(node) = self.get_node(node_id) else { return false };

        // Visit each child in pre-order
        for child_id in node.data.children() {
            if !self.traverse_pre_order(child_id, visit_fn) {
                return false;
            }
        }

        true
    }

    /// Performs a post-order traversal of the AST starting from the given node.
    /// Post-order traversal visits the children first, then the current node.
    ///
    /// ## Arguments
    ///
    /// * `node_id` - The ID of the node to start traversal from.
    /// * `visit_fn` - A function that is called for each node during traversal.
    ///
    /// ## Returns
    ///
    /// Returns true if the traversal completed successfully, false if it was aborted.
    pub fn traverse_post_order<F>(&self, node_id: NodeID, visit_fn: &mut F) -> bool
    where F: FnMut(NodeID) -> bool {
        // Get the node's children
        let Some(node) = self.get_node(node_id) else { return false };

        // Visit each child in post-order
        for child_id in node.data.children() {
            if !self.traverse_post_order(child_id, visit_fn) {
                return false;
            }
        }

        // Visit the current node last
        visit_fn(node_id)
    }

    /// Finds all nodes of a specific kind in the AST using pre-order traversal.
    ///
    /// ## Arguments
    ///
    /// * `start_node` - The ID of the node to start traversal from.
    /// * `node_kind` - The kind of nodes to find.
    ///
    /// ## Returns
    ///
    /// A vector of `NodeID`s for all nodes of the specified kind.
    pub fn find_nodes_of_kind(&self, start_node: NodeID, node_kind: NodeKind) -> Vec<NodeID> {
        let mut result = Vec::new();

        let _ = self.traverse_pre_order(start_node, &mut |node_id| {
            if let Some(node) = self.get_node(node_id)
                && node.kind == node_kind
            {
                result.push(node_id);
            }
            true // Continue traversal
        });

        result
    }

    /// Collects all nodes in pre-order traversal.
    ///
    /// ## Arguments
    ///
    /// * `start_node` - The ID of the node to start traversal from.
    ///
    /// ## Returns
    ///
    /// A vector of `NodeID`s for all nodes in pre-order traversal.
    pub fn collect_nodes_pre_order(&self, start_node: NodeID) -> Vec<NodeID> {
        let mut result = Vec::new();

        let _ = self.traverse_pre_order(start_node, &mut |node_id| {
            result.push(node_id);
            true // Continue traversal
        });

        result
    }

    /// Collects all nodes in post-order traversal.
    ///
    /// ## Arguments
    ///
    /// * `start_node` - The ID of the node to start traversal from.
    ///
    /// ## Returns
    ///
    /// A vector of `NodeID`s for all nodes in post-order traversal.
    pub fn collect_nodes_post_order(&self, start_node: NodeID) -> Vec<NodeID> {
        let mut result = Vec::new();

        let _ = self.traverse_post_order(start_node, &mut |node_id| {
            result.push(node_id);
            true // Continue traversal
        });

        result
    }

    /// Maps a function over all nodes in a pre-order traversal.
    ///
    /// This method visits all nodes in a pre-order traversal and applies the provided function
    /// to each node. It returns a vector containing the results of applying the function to each node.
    pub fn map_pre_order<F, T>(&self, start_node: NodeID, f: F) -> Vec<T>
    where F: FnMut(NodeID) -> T {
        let mut results = Vec::new();
        let mut func = f;

        let _ = self.traverse_pre_order(start_node, &mut |node_id| {
            results.push(func(node_id));
            true // Continue traversal
        });

        results
    }

    /// Maps a function over all nodes in a post-order traversal.
    ///
    /// This method visits all nodes in a post-order traversal and applies the provided function
    /// to each node. It returns a vector containing the results of applying the function to each node.
    pub fn map_post_order<F, T>(&self, start_node: NodeID, f: F) -> Vec<T>
    where F: FnMut(NodeID) -> T {
        let mut results = Vec::new();
        let mut func = f;

        let _ = self.traverse_post_order(start_node, &mut |node_id| {
            results.push(func(node_id));
            true // Continue traversal
        });

        results
    }

    /// Filters nodes that match a predicate in a pre-order traversal.
    ///
    /// This method visits all nodes in a pre-order traversal and returns a vector of
    /// `NodeID`s for those nodes that satisfy the provided predicate function.
    pub fn filter_nodes<F>(&self, start_node: NodeID, mut pred: F) -> Vec<NodeID>
    where F: FnMut(NodeID) -> bool {
        let mut results = Vec::new();

        let _ = self.traverse_pre_order(start_node, &mut |node_id| {
            if pred(node_id) {
                results.push(node_id);
            }
            true // Continue traversal
        });

        results
    }

    /// Finds the first node that matches a predicate in a pre-order traversal.
    ///
    /// This method visits nodes in a pre-order traversal and returns the first `NodeID`
    /// that satisfies the provided predicate function.
    pub fn find_node<F>(&self, start_node: NodeID, mut pred: F) -> Option<NodeID>
    where F: FnMut(NodeID) -> bool {
        let mut result = None;

        let _ = self.traverse_pre_order(start_node, &mut |node_id| {
            if pred(node_id) {
                result = Some(node_id);
                false // Stop traversal
            } else {
                true // Continue traversal
            }
        });

        result
    }

    /// Visits an AST node recursively using a visitor function.
    ///
    /// This method applies the visitor function to the node and all its children
    /// in a pre-order traversal. It stops when the visitor function returns false.
    pub fn visit_with<F>(&self, node_id: NodeID, mut visitor: F) -> bool
    where F: FnMut(&Node) -> bool {
        // Get the node
        let Some(node) = self.get_node(node_id) else { return false };

        // Visit the node
        if !visitor(node) {
            return false;
        }

        // Visit all children
        for child_id in node.data.children() {
            if !self.visit_with(child_id, |n| visitor(n)) {
                return false;
            }
        }

        true
    }

    /// Gets a specific node type from the AST by ID, with proper error handling.
    ///
    /// This method attempts to get a node of a specific type by ID, returning
    /// an error result if the node doesn't exist or is of the wrong type.
    ///
    /// ## Errors
    ///
    /// This method returns a `VisitorError` in the following cases:
    ///
    /// - `VisitorError::NodeNotFound`: If the node with the provided ID does not exist in the AST.
    /// - `VisitorError::TypeMismatch`: If the node exists but is not of the expected type `T`.
    pub fn get_node_as<T>(&mut self, node_id: NodeID) -> VisitorResult<T>
    where
        Self: Visitor<T>,
        T: 'static, {
        // Get the node
        let node = self.get_node(node_id).ok_or(VisitorError::NodeNotFound(node_id))?;
        let expected = type_name::<T>().to_string();
        let actual = format!("{:?}", node.kind);

        <Self as Visitor<T>>::visit(self, node_id).ok_or(VisitorError::TypeMismatch {
            node_id,
            expected,
            actual,
        })
    }

    /// Gets a strongly-typed reference to a specific node type from the AST by ID, with proper error handling.
    ///
    /// This enhanced version provides direct access to specific node types using the enum-based structure,
    /// eliminating the need for custom visitor implementations for simple node access patterns.
    ///
    /// ## Type Parameters
    ///
    /// - `T` - The specific node type to retrieve, such as `BinaryOpExpr`, `FunctionDecl`, etc.
    ///
    /// ## Arguments
    ///
    /// - `node_id` - The ID of the node to retrieve
    ///
    /// ## Returns
    ///
    /// A result containing a reference to the node of type `T`
    ///
    /// ## Errors
    ///
    /// This method returns a `VisitorError` in the following cases:
    ///
    /// - The node does not exist (`VisitorError::NodeNotFound`)
    /// - The node is not of the expected type (`VisitorError::TypeMismatch`)
    ///
    /// ## Example
    ///
    /// ```
    /// # use typhon_ast::ast::AST;
    /// # use typhon_ast::nodes::NodeID;
    /// # use typhon_ast::nodes::BinaryOpExpr;
    /// # let mut ast = AST::new();
    /// # let node_id = NodeID::new(0, 0);
    /// let binary_op = ast.get_as::<BinaryOpExpr>(node_id);
    /// ```
    #[allow(unsafe_code, trivial_casts, clippy::undocumented_unsafe_blocks, clippy::too_many_lines)]
    pub fn get_as<T: 'static>(&self, node_id: NodeID) -> VisitorResult<&T> {
        // Get the node first
        let node = self.get_node(node_id).ok_or(VisitorError::NodeNotFound(node_id))?;
        let expected_type = type_name::<T>().to_string();

        // Match on the node data to get the appropriate type
        match &node.data {
            AnyNode::ArgumentExpr(arg) if type_name::<ArgumentExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ArgumentExpr>(arg).cast::<T>() })
            }
            AnyNode::AssertStmt(assert_stmt) if type_name::<AssertStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AssertStmt>(assert_stmt).cast::<T>() })
            }
            AnyNode::AssignmentExpr(assignment)
                if type_name::<AssignmentExpr>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<AssignmentExpr>(assignment).cast::<T>() })
            }
            AnyNode::AssignmentStmt(stmt) if type_name::<AssignmentStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AssignmentStmt>(stmt).cast::<T>() })
            }
            AnyNode::AttributeExpr(attr) if type_name::<AttributeExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AttributeExpr>(attr).cast::<T>() })
            }
            AnyNode::BinaryOpExpr(binary_op) if type_name::<BinaryOpExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<BinaryOpExpr>(binary_op).cast::<T>() })
            }
            AnyNode::BreakStmt(break_stmt) if type_name::<BreakStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<BreakStmt>(break_stmt).cast::<T>() })
            }
            AnyNode::CallExpr(call) if type_name::<CallExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<CallExpr>(call).cast::<T>() })
            }
            AnyNode::ClassDecl(class_def) if type_name::<ClassDecl>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ClassDecl>(class_def).cast::<T>() })
            }
            AnyNode::BasicIdent(ident) if type_name::<BasicIdent>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<BasicIdent>(ident).cast::<T>() })
            }
            AnyNode::ContinueStmt(continue_stmt)
                if type_name::<ContinueStmt>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<ContinueStmt>(continue_stmt).cast::<T>() })
            }
            AnyNode::DictExpr(dict) if type_name::<DictExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<DictExpr>(dict).cast::<T>() })
            }
            AnyNode::ExceptHandler(handler) if type_name::<ExceptHandler>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ExceptHandler>(handler).cast::<T>() })
            }
            AnyNode::ExpressionStmt(expr_stmt)
                if type_name::<ExpressionStmt>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<ExpressionStmt>(expr_stmt).cast::<T>() })
            }
            AnyNode::ForStmt(for_stmt) if type_name::<ForStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ForStmt>(for_stmt).cast::<T>() })
            }
            AnyNode::FromImportStmt(from_import)
                if type_name::<FromImportStmt>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<FromImportStmt>(from_import).cast::<T>() })
            }
            AnyNode::FunctionDecl(func_def) if type_name::<FunctionDecl>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<FunctionDecl>(func_def).cast::<T>() })
            }
            AnyNode::GenericType(generic_type) if type_name::<GenericType>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<GenericType>(generic_type).cast::<T>() })
            }
            AnyNode::IfStmt(if_stmt) if type_name::<IfStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<IfStmt>(if_stmt).cast::<T>() })
            }
            AnyNode::ImportStmt(import_stmt) if type_name::<ImportStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ImportStmt>(import_stmt).cast::<T>() })
            }
            AnyNode::LambdaExpr(lambda) if type_name::<LambdaExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<LambdaExpr>(lambda).cast::<T>() })
            }
            AnyNode::ListExpr(list) if type_name::<ListExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ListExpr>(list).cast::<T>() })
            }
            AnyNode::LiteralExpr(literal) if type_name::<LiteralExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<LiteralExpr>(literal).cast::<T>() })
            }
            AnyNode::Module(module) if type_name::<Module>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<Module>(module).cast::<T>() })
            }
            AnyNode::ParameterIdent(param) if type_name::<ParameterIdent>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ParameterIdent>(param).cast::<T>() })
            }
            AnyNode::PassStmt(pass) if type_name::<PassStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<PassStmt>(pass).cast::<T>() })
            }
            AnyNode::RaiseStmt(raise) if type_name::<RaiseStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<RaiseStmt>(raise).cast::<T>() })
            }
            AnyNode::ReturnStmt(return_stmt) if type_name::<ReturnStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<ReturnStmt>(return_stmt).cast::<T>() })
            }
            AnyNode::SubscriptionExpr(subscription)
                if type_name::<SubscriptionExpr>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<SubscriptionExpr>(subscription).cast::<T>() })
            }
            AnyNode::TryStmt(try_stmt) if type_name::<TryStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<TryStmt>(try_stmt).cast::<T>() })
            }
            AnyNode::TupleExpr(tuple) if type_name::<TupleExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<TupleExpr>(tuple).cast::<T>() })
            }
            AnyNode::TypeDecl(type_def) if type_name::<TypeDecl>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<TypeDecl>(type_def).cast::<T>() })
            }
            AnyNode::UnaryOpExpr(unary_op) if type_name::<UnaryOpExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<UnaryOpExpr>(unary_op).cast::<T>() })
            }
            AnyNode::UnionType(union_type) if type_name::<UnionType>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<UnionType>(union_type).cast::<T>() })
            }
            AnyNode::VariableIdent(variable) if type_name::<VariableIdent>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<VariableIdent>(variable).cast::<T>() })
            }
            AnyNode::VariableDecl(var_decl) if type_name::<VariableDecl>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<VariableDecl>(var_decl).cast::<T>() })
            }
            AnyNode::WhileStmt(while_stmt) if type_name::<WhileStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<WhileStmt>(while_stmt).cast::<T>() })
            }
            AnyNode::AsPattern(as_pattern) if type_name::<AsPattern>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AsPattern>(as_pattern).cast::<T>() })
            }
            AnyNode::AsyncFunctionDecl(async_func)
                if type_name::<AsyncFunctionDecl>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<AsyncFunctionDecl>(async_func).cast::<T>() })
            }
            AnyNode::AsyncForStmt(async_for) if type_name::<AsyncForStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AsyncForStmt>(async_for).cast::<T>() })
            }
            AnyNode::AsyncWithStmt(async_with) if type_name::<AsyncWithStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AsyncWithStmt>(async_with).cast::<T>() })
            }
            AnyNode::AugmentedAssignmentStmt(aug_assign)
                if type_name::<AugmentedAssignmentStmt>() == expected_type =>
            {
                Ok(unsafe {
                    &*std::ptr::from_ref::<AugmentedAssignmentStmt>(aug_assign).cast::<T>()
                })
            }
            AnyNode::AwaitExpr(await_expr) if type_name::<AwaitExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<AwaitExpr>(await_expr).cast::<T>() })
            }
            AnyNode::CallableType(callable) if type_name::<CallableType>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<CallableType>(callable).cast::<T>() })
            }
            AnyNode::MatchCase(case) if type_name::<MatchCase>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<MatchCase>(case).cast::<T>() })
            }
            AnyNode::ClassPattern(class_pattern)
                if type_name::<ClassPattern>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<ClassPattern>(class_pattern).cast::<T>() })
            }
            AnyNode::DeleteStmt(delete) if type_name::<DeleteStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<DeleteStmt>(delete).cast::<T>() })
            }
            AnyNode::DictComprehensionExpr(dict_comp)
                if type_name::<DictComprehensionExpr>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<DictComprehensionExpr>(dict_comp).cast::<T>() })
            }
            AnyNode::FmtStringExpr(fmt_str) if type_name::<FmtStringExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<FmtStringExpr>(fmt_str).cast::<T>() })
            }
            AnyNode::GeneratorExpr(generator) if type_name::<GeneratorExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<GeneratorExpr>(generator).cast::<T>() })
            }
            AnyNode::GlobalStmt(global) if type_name::<GlobalStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<GlobalStmt>(global).cast::<T>() })
            }
            AnyNode::GroupingExpr(grouping) if type_name::<GroupingExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<GroupingExpr>(grouping).cast::<T>() })
            }
            AnyNode::IdentifierPattern(ident_pattern)
                if type_name::<IdentifierPattern>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<IdentifierPattern>(ident_pattern).cast::<T>() })
            }
            AnyNode::ListComprehensionExpr(list_comp)
                if type_name::<ListComprehensionExpr>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<ListComprehensionExpr>(list_comp).cast::<T>() })
            }
            AnyNode::LiteralPattern(lit_pattern)
                if type_name::<LiteralPattern>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<LiteralPattern>(lit_pattern).cast::<T>() })
            }
            AnyNode::LiteralType(lit_type) if type_name::<LiteralType>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<LiteralType>(lit_type).cast::<T>() })
            }
            AnyNode::MappingPattern(mapping_pattern)
                if type_name::<MappingPattern>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<MappingPattern>(mapping_pattern).cast::<T>() })
            }
            AnyNode::MatchStmt(match_stmt) if type_name::<MatchStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<MatchStmt>(match_stmt).cast::<T>() })
            }
            AnyNode::NonlocalStmt(nonlocal) if type_name::<NonlocalStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<NonlocalStmt>(nonlocal).cast::<T>() })
            }
            AnyNode::OrPattern(or_pattern) if type_name::<OrPattern>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<OrPattern>(or_pattern).cast::<T>() })
            }
            AnyNode::SequencePattern(seq_pattern)
                if type_name::<SequencePattern>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<SequencePattern>(seq_pattern).cast::<T>() })
            }
            AnyNode::SetExpr(set) if type_name::<SetExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<SetExpr>(set).cast::<T>() })
            }
            AnyNode::SetComprehensionExpr(set_comp)
                if type_name::<SetComprehensionExpr>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<SetComprehensionExpr>(set_comp).cast::<T>() })
            }
            AnyNode::StarredExpr(starred) if type_name::<StarredExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<StarredExpr>(starred).cast::<T>() })
            }
            AnyNode::TemplateStringExpr(template)
                if type_name::<TemplateStringExpr>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<TemplateStringExpr>(template).cast::<T>() })
            }
            AnyNode::TernaryExpr(ternary) if type_name::<TernaryExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<TernaryExpr>(ternary).cast::<T>() })
            }
            AnyNode::TupleType(tuple_type) if type_name::<TupleType>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<TupleType>(tuple_type).cast::<T>() })
            }
            AnyNode::WildcardPattern(wildcard)
                if type_name::<WildcardPattern>() == expected_type =>
            {
                Ok(unsafe { &*std::ptr::from_ref::<WildcardPattern>(wildcard).cast::<T>() })
            }
            AnyNode::WithStmt(with_stmt) if type_name::<WithStmt>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<WithStmt>(with_stmt).cast::<T>() })
            }
            AnyNode::YieldExpr(yield_expr) if type_name::<YieldExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<YieldExpr>(yield_expr).cast::<T>() })
            }
            AnyNode::YieldFromExpr(yield_from) if type_name::<YieldFromExpr>() == expected_type => {
                Ok(unsafe { &*std::ptr::from_ref::<YieldFromExpr>(yield_from).cast::<T>() })
            }
            _ => {
                let actual = format!("{:?}", node.kind);
                Err(VisitorError::TypeMismatch { node_id, expected: expected_type, actual })
            }
        }
    }
}

impl Clone for AST {
    fn clone(&self) -> Self {
        let allocator = Bump::new();

        Self {
            allocator,
            nodes: self.nodes.clone(),
            metadata: self.metadata.clone(),
            free_list: self.free_list.clone(),
            root: self.root,
        }
    }
}

impl Default for AST {
    fn default() -> Self { Self::new() }
}

impl Drop for AST {
    fn drop(&mut self) {
        // Clear all node references to avoid any potential issues
        for node in &mut self.nodes {
            *node = None;
        }
    }
}
