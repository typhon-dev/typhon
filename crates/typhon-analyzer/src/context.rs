//! Semantic analysis context.
//!
//! This module provides the main context for semantic analysis, which coordinates
//! symbol table management and type environment tracking.

use typhon_ast::ast::AST;
use typhon_ast::nodes::NodeID;

use crate::analysis::DeadCodeWarning;
use crate::error::SemanticError;
use crate::symbol::SymbolTable;
use crate::types::TypeEnvironment;
use crate::visitors::{
    NameResolverVisitor,
    SemanticValidatorVisitor,
    SymbolCollectorVisitor,
    TypeCheckerVisitor,
};

/// Main context for semantic analysis.
///
/// The `SemanticContext` brings together the symbol table and type environment,
/// providing a unified interface for semantic analysis operations.
///
/// This is a simplified version for Phase 1. Full analysis logic will be added
/// in later phases.
#[derive(Debug)]
pub struct SemanticContext {
    /// The symbol table for managing scopes and symbols.
    pub symbol_table: SymbolTable,
    /// The type environment for tracking type information.
    pub type_env: TypeEnvironment,
    /// Dead code warnings from semantic validation.
    warnings: Vec<DeadCodeWarning>,
}

impl SemanticContext {
    /// Creates a new semantic context with empty symbol table and type environment.
    #[must_use]
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            type_env: TypeEnvironment::new(),
            warnings: Vec::new(),
        }
    }

    /// Checks types in a module, performing type inference and validation.
    ///
    /// This performs the third pass of semantic analysis by inferring types
    /// for expressions, checking type compatibility, and validating operators,
    /// function calls, and assignments.
    ///
    /// ## Errors
    ///
    /// Returns semantic errors if any were encountered during type checking,
    /// such as type mismatches or invalid operations.
    pub fn check_types(&mut self, ast: &AST, module_id: NodeID) -> Result<(), Vec<SemanticError>> {
        let visitor = TypeCheckerVisitor::new(ast, &mut self.type_env, &mut self.symbol_table);

        visitor.check(module_id)
    }

    /// Collects symbols from a module AST, building the symbol table.
    ///
    /// This performs the first pass of semantic analysis by traversing the AST
    /// and collecting all symbol declarations while building the scope hierarchy.
    ///
    /// ## Errors
    ///
    /// Returns semantic errors if any were encountered during symbol collection,
    /// such as duplicate symbol definitions.
    pub fn collect_symbols(
        &mut self,
        ast: &AST,
        module_id: NodeID,
    ) -> Result<(), Vec<SemanticError>> {
        let visitor = SymbolCollectorVisitor::new(ast);
        self.symbol_table = visitor.collect(module_id)?;

        Ok(())
    }

    /// Resolves names in a module, binding references to definitions.
    ///
    /// This performs the second pass of semantic analysis by resolving all
    /// identifier references to their symbol definitions, performing closure
    /// analysis, and resolving type annotations.
    ///
    /// ## Errors
    ///
    /// Returns semantic errors if any were encountered during name resolution,
    /// such as undefined names or invalid type references.
    pub fn resolve_names(
        &mut self,
        ast: &AST,
        module_id: NodeID,
    ) -> Result<(), Vec<SemanticError>> {
        let visitor = NameResolverVisitor::new(ast, &mut self.symbol_table, &mut self.type_env);
        visitor.resolve(module_id)
    }

    /// Gets a reference to the symbol table.
    #[must_use]
    pub const fn symbol_table(&self) -> &SymbolTable { &self.symbol_table }

    /// Gets a mutable reference to the symbol table.
    pub const fn symbol_table_mut(&mut self) -> &mut SymbolTable { &mut self.symbol_table }

    /// Gets a reference to the type environment.
    #[must_use]
    pub const fn type_environment(&self) -> &TypeEnvironment { &self.type_env }

    /// Gets a mutable reference to the type environment.
    pub const fn type_environment_mut(&mut self) -> &mut TypeEnvironment { &mut self.type_env }

    /// Validates semantic rules in a module.
    ///
    /// This performs the fourth pass of semantic analysis by validating
    /// context-dependent semantic rules such as break/continue in loops,
    /// return in functions, and control flow requirements.
    ///
    /// This includes:
    /// - Context validation (break/continue/return in proper contexts)
    /// - Control flow analysis (CFG construction)
    /// - Missing return statement detection
    /// - Definite assignment analysis
    /// - Dead code detection (produces warnings, not errors)
    ///
    /// ## Errors
    ///
    /// Returns semantic errors if any were encountered during validation,
    /// such as break outside loop or missing return statements. Warnings
    /// (such as unreachable code) are collected but don't cause validation
    /// to fail.
    pub fn validate_semantics(
        &mut self,
        ast: &AST,
        module_id: NodeID,
    ) -> Result<(), Vec<SemanticError>> {
        // Run context validation which includes:
        // - break/continue only in loops
        // - return only in functions
        // - missing return statements (via CFG analysis)
        // - definite assignment checking
        // - dead code detection
        match SemanticValidatorVisitor::validate(ast, &self.symbol_table, module_id) {
            Ok(warnings) => {
                // Validation succeeded, collect warnings
                self.warnings = warnings;
                Ok(())
            }
            Err(errors) => {
                // Validation failed with errors
                Err(errors)
            }
        }
    }

    /// Gets a reference to the warnings.
    #[must_use]
    pub fn warnings(&self) -> &[DeadCodeWarning] { &self.warnings }
}

impl Default for SemanticContext {
    fn default() -> Self { Self::new() }
}
