//! Handler implementations for LSP requests.

use std::sync::Arc;

use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::{
    CompletionItem,
    CompletionItemKind,
    CompletionResponse,
    DocumentSymbol,
    DocumentSymbolResponse,
    GotoDefinitionResponse,
    Hover,
    HoverContents,
    Location,
    MarkupContent,
    MarkupKind,
    Position,
    Range,
    SymbolKind,
    TextDocumentIdentifier,
    TextDocumentPositionParams,
};
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

use crate::document::DocumentManager;
use crate::utils;

/// Handle a completion request.
pub(crate) fn completion_handler(
    document_manager: &DocumentManager,
    params: &TextDocumentPositionParams,
) -> jsonrpc::Result<Option<CompletionResponse>> {
    // Get the document
    let document = document_manager
        .get_document(&params.text_document.uri)
        .ok_or_else(|| jsonrpc::Error::invalid_params("Document not found"))?;

    let position = params.position;
    let text = document.text();

    // Find the word at the cursor position
    let word_info = utils::word_at_position(&text, &position);

    // Simple completion based on the current word
    let items = match word_info {
        Some((word, _)) => {
            // Create completions based on word
            let mut items = Vec::new();

            // Add keywords
            for keyword in &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "finally", "for", "from", "global", "if",
                "import", "in", "is", "lambda", "let", "mut", "nonlocal", "not", "or", "pass",
                "raise", "return", "try", "while", "with", "yield",
            ] {
                if keyword.starts_with(&word) {
                    items.push(CompletionItem {
                        label: keyword.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some("Typhon keyword".to_string()),
                        documentation: None,
                        deprecated: Some(false),
                        preselect: None,
                        sort_text: None,
                        filter_text: None,
                        insert_text: None,
                        insert_text_format: None,
                        insert_text_mode: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        commit_characters: None,
                        data: None,
                        tags: None,
                        label_details: None,
                    });
                }
            }

            // Add built-in functions
            for func in &[
                "abs",
                "all",
                "any",
                "bin",
                "bool",
                "chr",
                "dict",
                "dir",
                "divmod",
                "enumerate",
                "filter",
                "float",
                "format",
                "frozenset",
                "getattr",
                "hasattr",
                "hash",
                "hex",
                "id",
                "input",
                "int",
                "isinstance",
                "issubclass",
                "iter",
                "len",
                "list",
                "map",
                "max",
                "min",
                "next",
                "object",
                "oct",
                "open",
                "ord",
                "pow",
                "print",
                "property",
                "range",
                "repr",
                "reversed",
                "round",
                "set",
                "setattr",
                "slice",
                "sorted",
                "str",
                "sum",
                "super",
                "tuple",
                "type",
                "vars",
                "zip",
            ] {
                if func.starts_with(&word) {
                    items.push(CompletionItem {
                        label: func.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some("Built-in function".to_string()),
                        documentation: None,
                        deprecated: Some(false),
                        preselect: None,
                        sort_text: None,
                        filter_text: None,
                        insert_text: None,
                        insert_text_format: None,
                        insert_text_mode: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        commit_characters: None,
                        data: None,
                        tags: None,
                        label_details: None,
                    });
                }
            }

            // Add built-in types
            for type_name in &[
                "bool",
                "bytes",
                "complex",
                "dict",
                "float",
                "frozenset",
                "int",
                "list",
                "NoneType",
                "set",
                "str",
                "tuple",
            ] {
                if type_name.starts_with(&word) {
                    items.push(CompletionItem {
                        label: type_name.to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some("Built-in type".to_string()),
                        documentation: None,
                        deprecated: Some(false),
                        preselect: None,
                        sort_text: None,
                        filter_text: None,
                        insert_text: None,
                        insert_text_format: None,
                        insert_text_mode: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        commit_characters: None,
                        data: None,
                        tags: None,
                        label_details: None,
                    });
                }
            }

            for function_name in top_level_function_names(&text) {
                if function_name.starts_with(&word) {
                    items.push(CompletionItem {
                        label: function_name,
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some("Document function".to_string()),
                        documentation: None,
                        deprecated: Some(false),
                        preselect: None,
                        sort_text: None,
                        filter_text: None,
                        insert_text: None,
                        insert_text_format: None,
                        insert_text_mode: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        commit_characters: None,
                        data: None,
                        tags: None,
                        label_details: None,
                    });
                }
            }

            items
        }
        None => Vec::new(),
    };

    if items.is_empty() { Ok(None) } else { Ok(Some(CompletionResponse::Array(items))) }
}

/// Handle a hover request.
pub(crate) fn hover_handler(
    document_manager: &DocumentManager,
    params: &TextDocumentPositionParams,
) -> jsonrpc::Result<Option<Hover>> {
    // Get the document
    let document = document_manager
        .get_document(&params.text_document.uri)
        .ok_or_else(|| jsonrpc::Error::invalid_params("Document not found"))?;

    let position = params.position;
    let text = document.text();

    // Find the word at the cursor position
    if let Some((word, range)) = utils::word_at_position(&text, &position) {
        // Simple hover info based on the word
        let hover_text = match word.as_str() {
            // Keywords
            "and" => "Logical AND operator.",
            "as" => "Used in import statements and with expressions.",
            "assert" => "Assert that a condition is true, otherwise raise an AssertionError.",
            "async" => "Define an asynchronous function or context.",
            "await" => "Wait for a coroutine to complete.",
            "break" => "Exit from a loop.",
            "class" => "Define a class.",
            "continue" => "Skip to the next iteration of a loop.",
            "def" => "Define a function.",
            "del" => "Delete an object or attribute.",
            "elif" => "Else if condition in an if statement.",
            "else" => "Alternative execution block in conditional statements.",
            "except" => "Catch exceptions in a try block.",
            "finally" => "Code that always executes in a try statement.",
            "for" => "Loop over an iterable.",
            "from" => "Import specific attributes from a module.",
            "global" => "Declare a global variable.",
            "if" => "Conditional execution.",
            "import" => "Import modules.",
            "in" => "Check if a value is in a sequence.",
            "is" => "Identity operator.",
            "lambda" => "Create an anonymous function.",
            "let" => "Typhon-specific: Declare an immutable variable.",
            "mut" => "Typhon-specific: Declare a mutable variable.",
            "nonlocal" => "Declare a variable from the nearest enclosing scope.",
            "not" => "Logical NOT operator.",
            "or" => "Logical OR operator.",
            "pass" => "Do nothing statement.",
            "raise" => "Raise an exception.",
            "return" => "Exit a function and return a value.",
            "try" => "Try a block of code and catch exceptions.",
            "while" => "Execute a block of code as long as a condition is true.",
            "with" => "Context manager for resource cleanup.",
            "yield" => "Return a value from a generator.",

            // Add more word-specific hover information as needed
            _ => return Ok(None),
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```python\n{}\n```\n\n{}", word, hover_text),
            }),
            range: Some(range),
        }))
    } else {
        Ok(None)
    }
}

/// Handle a goto definition request.
pub(crate) const fn definition_handler(
    _document_manager: &DocumentManager,
    _params: &TextDocumentPositionParams,
) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
    // TODO: This is a placeholder implementation

    // 1. Parse the document
    // 2. Find the symbol at the position
    // 3. Look up its definition in the symbol table
    // 4. Return the location of the definition

    Ok(None)
}

/// Handle a find references request.
pub(crate) const fn references_handler(
    _document_manager: &DocumentManager,
    _params: &TextDocumentPositionParams,
    _include_declaration: bool,
) -> jsonrpc::Result<Option<Vec<Location>>> {
    // TODO: This is a placeholder implementation

    // 1. Parse the document
    // 2. Find the symbol at the position
    // 3. Search for all references to the symbol
    // 4. Return the locations of the references

    Ok(None)
}

/// Handle a document symbol request.
pub(crate) fn document_symbol_handler(
    document_manager: &DocumentManager,
    params: &TextDocumentIdentifier,
) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
    // Get the document
    let document = document_manager
        .get_document(&params.uri)
        .ok_or_else(|| jsonrpc::Error::invalid_params("Document not found"))?;

    let text = document.text();

    // Create a SourceManager and register this document so the parser can resolve spans.
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file(params.uri.to_string(), text.clone());
    let source_manager = Arc::new(source_manager);

    // Parse the document for diagnostics side effects and future nested-symbol support.
    // The top-level module symbol remains available even if parsing fails.
    let mut parser = Parser::new(&text, file_id, source_manager);
    drop(parser.parse_module());

    // Derive a friendly module name from the URI path. We currently expose only the
    // top-level module symbol; recursive AST traversal for nested function/class/variable
    // symbols is tracked as future work.
    let module_name = params
        .uri
        .path_segments()
        .and_then(|mut segs| segs.next_back())
        .map_or_else(|| "module".to_string(), |s| s.trim_end_matches(".ty").to_string());

    let module_name_len = u32::try_from(module_name.len()).unwrap_or(u32::MAX);
    let symbols = vec![{
        #[allow(deprecated)]
        DocumentSymbol {
            name: module_name,
            detail: Some("Module".to_string()),
            kind: SymbolKind::FILE,
            tags: None,
            deprecated: None,
            range: Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
            selection_range: Range::new(Position::new(0, 0), Position::new(0, module_name_len)),
            children: None,
        }
    }];

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

fn top_level_function_names(text: &str) -> impl Iterator<Item = String> + '_ {
    text.lines().filter_map(|line| {
        let trimmed = line.trim_start();

        if trimmed.len() == line.len() && trimmed.starts_with("def ") {
            let name = trimmed[4..]
                .split(|character: char| !character.is_alphanumeric() && character != '_')
                .next()
                .unwrap_or_default();

            if !name.is_empty() {
                return Some(name.to_string());
            }
        }

        None
    })
}
