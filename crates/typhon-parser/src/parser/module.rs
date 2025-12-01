//! Module parsing for the Typhon programming language.
//!
//! This module handles parsing top-level modules, import statements,
//! and maintains file-level context information.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use typhon_ast::ast::AST;
use typhon_ast::nodes::{AnyNode, FromImportStmt, ImportStmt, Module, NodeID, NodeKind};
use typhon_source::types::{SourceManager, Span};

use crate::diagnostics::{ParseError, ParseResult};
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::parser::context::{Context, ContextType};

impl Parser<'_> {
    /// Parse a dotted name (e.g. `module.submodule.name`).
    ///
    /// ## Errors
    ///
    /// This function will return a `ParserError` if the dotted name cannot be parsed.
    fn parse_dotted_name(&mut self) -> ParseResult<Vec<String>> {
        let mut parts = Vec::new();

        // Parse the first part of the dotted name
        // Accept both Identifier and Underscore tokens (for names like _annotations)
        if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
            return Err(self.error("Expected identifier in import statement"));
        }

        // Add the first part
        parts.push(self.current_token().lexeme.to_string());
        let _ = self.advance();

        // Parse additional parts separated by dots
        while self.check(TokenKind::Dot) {
            let _ = self.advance(); // Consume the dot

            // Expect an identifier or underscore after the dot
            if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
                return Err(self.error("Expected identifier after '.' in import statement"));
            }

            // Add the part
            parts.push(self.current_token().lexeme.to_string());
            let _ = self.advance();
        }

        Ok(parts)
    }

    /// Parse a from-import statement (e.g. `from module import name [as alias]`).
    ///
    /// ## Errors
    ///
    /// This function will return a `ParserError` if the from-import statement cannot be parsed.
    pub fn parse_from_import_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span().start;

        // Consume the 'from' token
        let _ = self.advance();

        // Parse relative import dots
        let mut level = 0;
        while self.check(TokenKind::Dot) {
            let _ = self.advance();
            level += 1;
        }

        // Parse the module name (dotted name)
        let module_parts = self.parse_dotted_name()?;

        // Expect 'import' keyword
        self.expect(TokenKind::Import)?;

        // Check for optional opening parenthesis (for multi-line imports)
        let has_parens = if self.check(TokenKind::LeftParen) {
            let _ = self.advance(); // Consume '('
            true
        } else {
            false
        };

        // Parse the imported names
        let names = if self.check(TokenKind::Star) {
            // Handle "from module import *"
            let _ = self.advance(); // Consume '*'
            vec![("*".to_string(), None)]
        } else {
            // Parse a comma-separated list of names with optional aliases
            self.parse_import_names()?
        };

        // If we had an opening paren, expect the closing paren
        if has_parens {
            self.expect(TokenKind::RightParen)?;
        }

        // Get the end position
        let end_pos = self.current_token().span.start;

        // Create a span for the from-import statement
        let span = Span::new(start_pos, end_pos);

        // Create the AST FromImport node
        let from_import =
            FromImportStmt::new(module_parts, names, level, NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::FromImportStmt(from_import), span);

        // Only expect explicit statement terminator if parentheses were not used.
        // When parentheses are used, the lexer's implicit line continuation consumes
        // newlines inside brackets, leaving no explicit newline token after the closing paren.
        if !has_parens {
            self.expect_statement_end()?;
        }

        Ok(node_id)
    }

    /// Parse a comma-separated list of import names with optional aliases.
    ///
    /// This function parses names until it encounters:
    /// - A closing paren (when called from parenthesized context)
    /// - A newline (when in non-parenthesized context)
    /// - End of file
    ///
    /// The caller is responsible for handling parentheses.
    ///
    /// ## Errors
    ///
    /// This function will return a `ParserError` if the import names cannot be parsed.
    fn parse_import_names(&mut self) -> ParseResult<Vec<(String, Option<String>)>> {
        let mut names = Vec::new();

        // Parse the first name
        // Accept both Identifier and Underscore tokens (for names like _annotations)
        if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
            return Err(self.error("Expected identifier in import statement"));
        }

        loop {
            // Parse a name
            let name = self.current_token().lexeme.to_string();
            let _ = self.advance();

            // Check for alias
            let alias = if self.check(TokenKind::As) {
                let _ = self.advance(); // Consume 'as'

                // Expect an identifier or underscore for the alias
                if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
                    return Err(self.error("Expected identifier after 'as'"));
                }

                let alias_name = self.current_token().lexeme.to_string();
                let _ = self.advance();

                Some(alias_name)
            } else {
                None
            };

            // Add the name-alias pair
            names.push((name, alias));

            // Check for comma to continue the list
            if !self.check(TokenKind::Comma) {
                // No comma means end of import list
                break;
            }

            let _ = self.advance(); // Consume the comma

            // Check if we've reached the closing paren (trailing comma case)
            // The lexer handles implicit line continuation inside parens, so newlines
            // are automatically skipped there.
            if self.check(TokenKind::RightParen) {
                // Trailing comma before closing paren - this is valid
                break;
            }

            // Expect another identifier or underscore
            if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
                return Err(self.error("Expected identifier after ',' in import statement"));
            }
        }

        Ok(names)
    }

    /// Parse an import statement (e.g. `import module [as name]`).
    ///
    /// ## Errors
    ///
    /// This function will return a `ParserError` if the import statement cannot be parsed.
    pub fn parse_import_statement(&mut self) -> ParseResult<NodeID> {
        // Get the start position
        let start_pos = self.current_token().span.start;

        // Consume the 'import' token
        let _ = self.advance();

        // Parse the module name (dotted name)
        let module_parts = self.parse_dotted_name()?;

        // Check for 'as' to handle aliases
        let alias = if self.check(TokenKind::As) {
            let _ = self.advance(); // Consume 'as'

            // Expect an identifier or underscore for the alias
            if !self.matches(&[TokenKind::Identifier, TokenKind::Underscore]) {
                return Err(self.error("Expected identifier after 'as'"));
            }

            let alias_name = self.current_token().lexeme.to_string();
            let _ = self.advance(); // Consume the identifier

            Some(alias_name)
        } else {
            None
        };

        // Get the end position
        let end_pos = self.current_token().span.start;

        // Create a span for the import statement
        let span = Span::new(start_pos, end_pos);

        // Create the AST ImportStmt node
        let import_stmt = ImportStmt::new(module_parts, alias, NodeID::placeholder(), span);

        // Allocate the statement node in the AST
        let stmt_node_id =
            self.ast.alloc_node(NodeKind::Statement, AnyNode::ImportStmt(import_stmt), span);

        // Expect a newline or semicolon after the import statement
        self.expect_statement_end()?;

        Ok(stmt_node_id)
    }

    /// Parse a module from source code.
    ///
    /// This is the top-level parsing function that processes an entire file.
    ///
    /// ## Errors
    ///
    /// This function will return a `ParserError` if the module cannot be parsed.
    pub fn parse_module(&mut self) -> ParseResult<NodeID> {
        // Create a module context
        self.context_stack.push(Context::new(
            ContextType::Global,
            None,
            0, // Top-level indentation
        ));

        // Parse all statements in the module
        let mut statements = Vec::new();

        // Continue parsing statements until we reach the end of the file
        while !self.check(TokenKind::EndOfFile) {
            // Handle any indentation tokens at the module level
            self.handle_indentation()?;

            // Parse the statement
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            // Skip newlines between statements
            while self.check(TokenKind::Newline) {
                let _ = self.advance();
            }
        }

        // Create a module node
        let start_pos = 0; // Start at beginning of file
        let end_pos = self.source.len(); // End at end of file
        let span = Span::new(start_pos, end_pos);

        // Determine the module name (often based on the file path)
        let file_path = self.source_manager.get_file(self.file_id).map_or_else(
            || PathBuf::from("unnamed_module"),
            |file| file.path.clone().unwrap_or_else(|| PathBuf::from("unnamed_module")),
        );

        // Extract the filename without extension as the module name
        let module_name = Path::new(&file_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unnamed_module")
            .to_string();

        // Create the AST module node
        let ast_module = Module::new(module_name, statements.clone(), NodeID::placeholder(), span);

        // Allocate the node in the AST
        let node_id = self.ast.alloc_node(NodeKind::Module, AnyNode::Module(ast_module), span);

        // Set parent-child relationships
        for stmt in &statements {
            let _ = self.ast.set_parent(*stmt, node_id);
        }

        // Pop the module context
        let _ = self.context_stack.pop();

        Ok(node_id)
    }

    /// Parse a module from a file.
    ///
    /// This is a convenience function to parse a module directly from a file path.
    ///
    /// ## Errors
    ///
    /// This function will return a `ParserError` if the file cannot be read or parsed.
    pub fn parse_module_from_file(file_path: &str) -> ParseResult<(AST, NodeID)> {
        // Read the file contents
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| ParseError::other(format!("Failed to read file '{file_path}': {e}")))?;

        // Create a source manager
        let mut source_manager = SourceManager::new();

        // Register the file with the source manager
        let file_id = source_manager.add_file(file_path.to_string(), source.clone());

        // Create a shared source manager
        let source_manager = Arc::new(source_manager);

        // Create a parser
        let mut parser = Parser::new(&source, file_id, source_manager);

        // Parse the module
        let module_id = parser.parse_module()?;

        Ok((parser.ast, module_id))
    }
}
