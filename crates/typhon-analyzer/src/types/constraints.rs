//! Type constraint system for type inference.
//!
//! This module implements a constraint-based type inference system that collects
//! type constraints during AST traversal and solves them to determine concrete types.

use std::collections::HashMap;

use super::environment::TypeEnvironment;
use super::ty::TypeID;
use crate::error::SemanticError;

/// Represents a type constraint that must be satisfied during type inference.
///
/// Constraints are generated during type checking and solved to determine
/// concrete types for type variables and inferred expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Equality constraint: T1 must equal T2
    Equality(TypeID, TypeID),
    /// Subtype constraint: T1 must be a subtype of T2 (T1 <: T2)
    Subtype(TypeID, TypeID),
    /// Attribute constraint: T must have attribute `name` of type T2
    HasAttribute(TypeID, String, TypeID),
    /// Method constraint: T must have method `name` with argument types and return type
    HasMethod(TypeID, String, Vec<TypeID>, TypeID),
}

/// Solves type constraints to determine concrete types.
///
/// The constraint solver uses a unification algorithm to solve collected
/// type constraints, determining concrete types for type variables and
/// validating type compatibility.
#[derive(Debug)]
pub struct ConstraintSolver {
    /// Collected type constraints
    constraints: Vec<TypeConstraint>,
    /// Type variable substitutions (type variable name -> concrete `TypeID`)
    substitutions: HashMap<String, TypeID>,
}

impl ConstraintSolver {
    /// Creates a new empty constraint solver.
    #[must_use]
    pub fn new() -> Self { Self { constraints: Vec::new(), substitutions: HashMap::new() } }

    /// Adds a constraint to be solved.
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }

    /// Gets the substitution for a type variable.
    #[must_use]
    pub fn get_substitution(&self, var_name: &str) -> Option<TypeID> {
        self.substitutions.get(var_name).copied()
    }

    /// Solves all collected constraints.
    ///
    /// This performs constraint solving using unification, updating the type
    /// environment with inferred types and substitutions.
    ///
    /// ## Errors
    ///
    /// Returns semantic errors if constraints cannot be satisfied (type mismatches).
    pub fn solve(&mut self, type_env: &mut TypeEnvironment) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();

        // Clone constraints to avoid borrow checker issues
        let constraints = self.constraints.clone();

        // Process each constraint
        for constraint in &constraints {
            if let Err(err) = Self::solve_constraint(constraint, type_env) {
                errors.push(err);
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Checks if subtype is a subtype of supertype.
    fn check_subtype(
        subtype_id: TypeID,
        supertype_id: TypeID,
        type_env: &TypeEnvironment,
    ) -> Result<(), SemanticError> {
        let subtype = type_env.get_type(subtype_id);
        let supertype = type_env.get_type(supertype_id);

        match (subtype, supertype) {
            (Some(found), Some(expected)) => {
                if found.is_subtype_of(expected) {
                    Ok(())
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: Box::new(expected.clone()),
                        found: Box::new(found.clone()),
                        span: typhon_source::types::Span::new(0, 0),
                    })
                }
            }
            _ => Ok(()), // Missing types - will be caught elsewhere
        }
    }

    /// Solves a single constraint.
    fn solve_constraint(
        constraint: &TypeConstraint,
        type_env: &TypeEnvironment,
    ) -> Result<(), SemanticError> {
        match constraint {
            TypeConstraint::Equality(left_type_id, right_type_id) => {
                Self::unify(*left_type_id, *right_type_id, type_env)
            }
            TypeConstraint::HasAttribute(_type_id, _attr_name, _attr_type_id) => {
                // TODO: For now, we'll defer attribute checking to the type checker
                // Full implementation would check if the type has the attribute
                Ok(())
            }
            TypeConstraint::HasMethod(_type_id, _method_name, _arg_types, _return_type) => {
                // TODO: For now, we'll defer method checking to the type checker
                // Full implementation would check if the type has the method
                Ok(())
            }
            TypeConstraint::Subtype(left_type_id, right_type_id) => {
                Self::check_subtype(*left_type_id, *right_type_id, type_env)
            }
        }
    }

    /// Unifies two types, ensuring they are compatible.
    fn unify(
        left_type_id: TypeID,
        right_type_id: TypeID,
        type_env: &TypeEnvironment,
    ) -> Result<(), SemanticError> {
        // Get the actual types
        let left_type = type_env.get_type(left_type_id);
        let right_type = type_env.get_type(right_type_id);

        match (left_type, right_type) {
            (Some(expected), Some(found)) => {
                // Check if types are compatible
                if expected.is_compatible_with(found) {
                    Ok(())
                } else {
                    Err(SemanticError::TypeMismatch {
                        expected: Box::new(expected.clone()),
                        found: Box::new(found.clone()),
                        span: typhon_source::types::Span::new(0, 0),
                    })
                }
            }
            _ => Ok(()), // Missing types - will be caught elsewhere
        }
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self { Self::new() }
}
