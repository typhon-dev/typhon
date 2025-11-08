// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-lsp/src/capabilities.rs
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
//! LSP capabilities of the Typhon Language Server.

use tower_lsp::lsp_types::*;

/// Returns the server capabilities that the Typhon Language Server supports.
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF8),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                save: Some(SaveOptions::default().into()),
            },
        )),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string()]),
            completion_item: None,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(false),
            },
            all_commit_characters: None,
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        signature_help_provider: None,
        declaration_provider: None,
        definition_provider: Some(OneOf::Left(true)),
        type_definition_provider: None,
        implementation_provider: None,
        references_provider: Some(OneOf::Left(true)),
        document_highlight_provider: None,
        document_symbol_provider: Some(OneOf::Left(true)),
        code_action_provider: None,
        code_lens_provider: None,
        document_formatting_provider: None,
        document_range_formatting_provider: None,
        document_on_type_formatting_provider: None,
        selection_range_provider: None,
        folding_range_provider: None,
        rename_provider: None,
        linked_editing_range_provider: None,
        document_link_provider: None,
        color_provider: None,
        workspace_symbol_provider: None,
        call_hierarchy_provider: None,
        semantic_tokens_provider: None,
        moniker_provider: None,
        workspace: None,
        experimental: None,
        execute_command_provider: None,
        position_encoding: Some(PositionEncodingKind::UTF8),
        inlay_hint_provider: None,
        inline_value_provider: None,
        diagnostic_provider: None,
    }
}
