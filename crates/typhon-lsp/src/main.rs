//! Typhon Language Server
//!
//! LSP implementation for the Typhon programming language.

mod server;
mod handlers;
mod utils;
mod document;
mod capabilities;

use std::error::Error;
use tower_lsp::{LspService, Server};
use typhon_compiler as compiler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Typhon Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create a new language server instance
    let (service, socket) = LspService::build(|client| {
        server::TyphonLanguageServer::new(client)
    })
    .finish();

    // Start the language server
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
