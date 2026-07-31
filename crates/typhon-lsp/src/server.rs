//! LSP server implementation for the Typhon programming language.

use std::sync::Arc;

use parking_lot::RwLock;
use tower_lsp::jsonrpc::{Error as JsonRpcError, Result as JsonRpcResult};
use tower_lsp::lsp_types::{
    CompletionParams,
    CompletionResponse,
    Diagnostic,
    DiagnosticSeverity,
    DidChangeTextDocumentParams,
    DidCloseTextDocumentParams,
    DidOpenTextDocumentParams,
    DocumentSymbolParams,
    DocumentSymbolResponse,
    GotoDefinitionParams,
    GotoDefinitionResponse,
    Hover,
    HoverParams,
    InitializeParams,
    InitializeResult,
    InitializedParams,
    Location,
    MessageType,
    ReferenceParams,
    ServerInfo,
    Url,
};
use tower_lsp::{Client, LanguageServer};
use typhon_parser::parser::Parser;
use typhon_source::types::SourceManager;

use crate::capabilities::server_capabilities;
use crate::document::DocumentManager;
use crate::handlers::{
    completion_handler,
    definition_handler,
    document_symbol_handler,
    hover_handler,
    references_handler,
};

/// The main LSP server implementation for the Typhon language.
pub struct TyphonLanguageServer {
    /// LSP client connection for sending notifications and requests
    client: Client,
    /// Document manager for tracking open documents
    document_manager: Arc<RwLock<DocumentManager>>,
}

impl TyphonLanguageServer {
    /// Create a new Typhon language server.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self { client, document_manager: Arc::new(RwLock::new(DocumentManager::new())) }
    }

    /// Helper method to log information to the client.
    pub async fn log_info(&self, message: impl Into<String>) {
        self.client.log_message(MessageType::INFO, message.into()).await;
    }

    /// Helper method to log errors to the client.
    pub async fn log_error(&self, message: impl Into<String>) {
        self.client.log_message(MessageType::ERROR, message.into()).await;
    }

    /// Helper to publish diagnostics for a document.
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    /// Run diagnostics on a document and publish the results.
    ///
    /// Currently this only runs the parser and forwards parser-level diagnostics
    /// (syntax errors and lexer-bridged errors) to the client. Semantic
    /// (type-checker) diagnostics will be re-enabled once the LSP wires up an
    /// `AST` + `SymbolTable` + `TypeEnvironment` pipeline matching the current
    /// `typhon_analyzer::visitors::TypeCheckerVisitor::new(ast, type_env, symbol_table)`
    /// API; see Subtask 2A notes.
    async fn run_diagnostics(&self, uri: &Url) -> JsonRpcResult<()> {
        // Run the parser inside a synchronous scope so non-`Send` parser internals
        // (the AST arena, `Parser` itself) are dropped before the `await` below.
        let diagnostics = {
            let document_manager = self.document_manager.read();
            let document = document_manager
                .get_document(uri)
                .ok_or_else(|| JsonRpcError::invalid_params("Document not found"))?;

            let text = document.text();

            // Set up a single-file SourceManager so the parser can resolve spans for any
            // diagnostics it produces.
            let mut source_manager = SourceManager::new();
            let file_id = source_manager.add_file(uri.to_string(), text.clone());
            let source_manager = Arc::new(source_manager);

            let mut parser = Parser::new(&text, file_id, source_manager);
            drop(parser.parse_module());

            parser
                .diagnostics()
                .diagnostics()
                .iter()
                .map(|diag| Diagnostic {
                    range: document.range_from_span(diag.span.start.offset..diag.span.end.offset),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("typhon-parser".to_string()),
                    message: diag.message.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                })
                .collect::<Vec<_>>()
        };

        self.publish_diagnostics(uri.clone(), diagnostics).await;

        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TyphonLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        self.log_info("Typhon Language Server initialized").await;

        // Return server capabilities
        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo {
                name: "Typhon Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.log_info("Typhon Language Server is ready").await;
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        self.log_info("Shutting down Typhon Language Server").await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        {
            let mut document_manager = self.document_manager.write();
            document_manager.add_document(uri.clone(), &text, version);
        }

        // Run diagnostics on the opened document
        if let Err(e) = self.run_diagnostics(&uri).await {
            self.log_error(format!("Error running diagnostics: {e}")).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        {
            let mut document_manager = self.document_manager.write();
            for change in params.content_changes {
                if let Some(range) = change.range {
                    document_manager.update_document(&uri, range, &change.text, version);
                } else {
                    document_manager.replace_document(&uri, &change.text, version);
                }
            }
        }

        // Run diagnostics on the changed document
        if let Err(e) = self.run_diagnostics(&uri).await {
            self.log_error(format!("Error running diagnostics: {e}")).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        {
            let mut document_manager = self.document_manager.write();
            document_manager.remove_document(&uri);
        }

        // Clear diagnostics for the closed document
        self.publish_diagnostics(uri, Vec::new()).await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> JsonRpcResult<Option<CompletionResponse>> {
        let document_manager = self.document_manager.read();

        completion_handler(&document_manager, &params.text_document_position)
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        let document_manager = self.document_manager.read();

        hover_handler(&document_manager, &params.text_document_position_params)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> JsonRpcResult<Option<GotoDefinitionResponse>> {
        let document_manager = self.document_manager.read();

        definition_handler(&document_manager, &params.text_document_position_params)
    }

    async fn references(&self, params: ReferenceParams) -> JsonRpcResult<Option<Vec<Location>>> {
        let document_manager = self.document_manager.read();

        references_handler(
            &document_manager,
            &params.text_document_position,
            params.context.include_declaration,
        )
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> JsonRpcResult<Option<DocumentSymbolResponse>> {
        let document_manager = self.document_manager.read();

        document_symbol_handler(&document_manager, &params.text_document)
    }
}
