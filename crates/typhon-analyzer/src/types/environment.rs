//! Type environment for tracking type information during analysis.

use rustc_hash::FxHashMap;
use typhon_ast::nodes::NodeID;

use super::ty::{Type, TypeID};

/// Type environment tracking type information during analysis.
///
/// The type environment maintains mappings from AST nodes to their inferred
/// or annotated types, and manages type storage.
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Storage for all types, indexed by `TypeID`.
    types: Vec<Type>,
    /// Map from AST node ID to type ID.
    node_types: FxHashMap<NodeID, TypeID>,
    /// Map from type variables to their substituted types.
    #[allow(dead_code)] // Reserved for future type inference implementation
    substitutions: FxHashMap<String, TypeID>,
}

impl TypeEnvironment {
    /// Creates a new empty type environment.
    #[must_use]
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            node_types: FxHashMap::default(),
            substitutions: FxHashMap::default(),
        }
    }

    /// Adds a type to the environment and returns its ID.
    pub fn add_type(&mut self, ty: Type) -> TypeID {
        let id = TypeID::new(self.types.len());
        self.types.push(ty);
        id
    }

    /// Gets the type ID for an AST node.
    #[must_use]
    pub fn get_node_type(&self, node_id: NodeID) -> Option<TypeID> {
        self.node_types.get(&node_id).copied()
    }

    /// Gets a type by its ID.
    #[must_use]
    pub fn get_type(&self, type_id: TypeID) -> Option<&Type> { self.types.get(type_id.value()) }

    /// Sets the type for an AST node.
    pub fn set_node_type(&mut self, node_id: NodeID, type_id: TypeID) {
        let _ = self.node_types.insert(node_id, type_id);
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self { Self::new() }
}
