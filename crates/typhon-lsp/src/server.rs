// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-lsp/src/server.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
//! LSP server implementation for the Typhon programming language.

use std::sync::Arc;

use parking_lot::RwLock;
use tower_lsp::jsonrpc::{
    Error as JsonRpcError,
    Result as JsonRpcResult,
};
use tower_lsp::lsp_types::*;
use tower_lsp::{
    Client,
    LanguageServer,
};
use typhon_compiler::frontend::lexer::Lexer;
use typhon_compiler::frontend::parser::Parser;
use typhon_compiler::typesystem::TypeChecker;

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
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_manager: Arc::new(RwLock::new(DocumentManager::new())),
        }
    }

    /// Helper method to log information to the client.
    pub async fn log_info(&self, message: impl Into<String>) {
        self.client
            .log_message(MessageType::INFO, message.into())
            .await;
    }

    /// Helper method to log errors to the client.
    pub async fn log_error(&self, message: impl Into<String>) {
        self.client
            .log_message(MessageType::ERROR, message.into())
            .await;
    }

    /// Helper to publish diagnostics for a document.
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Run diagnostics on a document and publish the results.
    async fn run_diagnostics(&self, uri: &Url) -> JsonRpcResult<()> {
        let document_manager = self.document_manager.read();
        let document = document_manager
            .get_document(uri)
            .ok_or_else(|| JsonRpcError::invalid_params("Document not found"))?;

        // Create a new lexer for the document
        let lexer = Lexer::new(document.text());

        // Create a new parser with the lexer
        let mut parser = Parser::new(lexer);

        // Parse the document
        let parse_result = parser.parse();

        let mut diagnostics = Vec::new();

        // Add syntax errors to diagnostics
        for error in parser.errors() {
            let range = document.range_from_span(error.span.clone());
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("typhon-parser".to_string()),
                message: error.to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        // If parsing was successful, run the type checker
        if let Ok(module) = parse_result {
            let mut type_checker = TypeChecker::new();
            let type_check_result = type_checker.check_module(&module);

            // Add type errors to diagnostics
            for error in type_checker.errors() {
                if let Some(span) = error.span() {
                    let range = document.range_from_span(span.clone());
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("typhon-type-checker".to_string()),
                        message: error.to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                } else {
                    // For errors without a specific span, place at the beginning
                    diagnostics.push(Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("typhon-type-checker".to_string()),
                        message: error.to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
        }

        // Publish the diagnostics
        self.publish_diagnostics(uri.clone(), diagnostics).await;

        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TyphonLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
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
            document_manager.add_document(uri.clone(), text, version);
        }

        // Run diagnostics on the opened document
        if let Err(e) = self.run_diagnostics(&uri).await {
            self.log_error(format!("Error running diagnostics: {}", e))
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        {
            let mut document_manager = self.document_manager.write();
            for change in params.content_changes {
                if let Some(range) = change.range {
                    document_manager.update_document(&uri, range, change.text, version);
                } else {
                    document_manager.replace_document(&uri, change.text, version);
                }
            }
        }

        // Run diagnostics on the changed document
        if let Err(e) = self.run_diagnostics(&uri).await {
            self.log_error(format!("Error running diagnostics: {}", e))
                .await;
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
