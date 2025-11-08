// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-lsp/src/main.rs
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
//! Typhon Language Server
//!
//! LSP implementation for the Typhon programming language.

mod capabilities;
mod document;
mod handlers;
mod server;
mod utils;

use std::error::Error;

use tower_lsp::{
    LspService,
    Server,
};
use typhon_compiler as compiler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Typhon Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create a new language server instance
    let (service, socket) =
        LspService::build(|client| server::TyphonLanguageServer::new(client)).finish();

    // Start the language server
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
