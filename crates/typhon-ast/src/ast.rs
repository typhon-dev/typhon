//! AST arena allocator for efficient node management.
//!
//! This module provides the core AST arena that manages memory allocation
//! for all AST nodes using a bump allocator with generation-based safety.

use std::any::type_name;

use bumpalo::Bump;
use typhon_source::types::Span;

use crate::nodes::{ASTNode, AnyNode, Node, NodeID, NodeKind};
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
    ///
    /// ## Errors
    ///
    /// Returns an error if the node doesn't exist or the type doesn't match.
    pub fn get_as<T: 'static>(&self, node_id: NodeID) -> VisitorResult<&T> {
        let node = self.get_node(node_id).ok_or(VisitorError::NodeNotFound(node_id))?;

        node.data.get_as::<T>().map_err(|_msg| {
            let expected = type_name::<T>().to_string();
            let actual = format!("{:?}", node.kind);
            VisitorError::TypeMismatch { node_id, expected, actual }
        })
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
