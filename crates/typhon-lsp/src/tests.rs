//! Tests for the Typhon Language Server.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::RwLock;
    use tower_lsp::lsp_types::*;
    use url::Url;

    use crate::document::DocumentManager;
    use crate::handlers::{
        completion_handler,
        definition_handler,
        document_symbol_handler,
        hover_handler,
        references_handler,
    };

    // Helper function to create a test document manager
    fn setup_test_document_manager() -> Arc<RwLock<DocumentManager>> {
        let mut document_manager = DocumentManager::new();
        let uri = Url::parse("file:///test.ty").unwrap();
        let content = r#"
def add(a: int, b: int) -> int:
    return a + b

let x: int = 10
let y: int = 20
let result = add(x, y)
"#
        .to_string();
        document_manager.add_document(uri, content, 1);
        Arc::new(RwLock::new(document_manager))
    }

    #[test]
    fn test_document_manager() {
        let document_manager = setup_test_document_manager();
        let uri = Url::parse("file:///test.ty").unwrap();

        {
            let dm = document_manager.read();
            let doc = dm.get_document(&uri).unwrap();
            assert_eq!(doc.version(), 1);
            assert!(doc.text().contains("def add"));
        }

        // Test update
        {
            let mut dm = document_manager.write();
            dm.update_document(
                &uri,
                Range::new(Position::new(0, 0), Position::new(0, 0)),
                "# Test comment\n".to_string(),
                2,
            );
        }

        // Verify update
        {
            let dm = document_manager.read();
            let doc = dm.get_document(&uri).unwrap();
            assert_eq!(doc.version(), 2);
            assert!(doc.text().starts_with("# Test comment\n"));
        }
    }

    #[test]
    fn test_completion_handler() {
        let document_manager = setup_test_document_manager();
        let uri = Url::parse("file:///test.ty").unwrap();

        // Test completion for 'ad' (should suggest 'add')
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(6, 13), // Position just before 'd' in "add(x, y)"
        };

        let completions = completion_handler(&document_manager.read(), &params).unwrap();

        match completions {
            Some(CompletionResponse::Array(items)) => {
                // Should include at least 'add' in completions
                let has_add = items.iter().any(|item| item.label == "add");
                assert!(has_add, "Completion should include 'add'");
            }
            _ => panic!("Expected completion items"),
        }
    }

    #[test]
    fn test_hover_handler() {
        let document_manager = setup_test_document_manager();
        let uri = Url::parse("file:///test.ty").unwrap();

        // Test hover over 'def' keyword
        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(1, 2), // Position at 'def' keyword
        };

        let hover = hover_handler(&document_manager.read(), &params).unwrap();

        match hover {
            Some(hover) => {
                assert!(
                    hover.contents.to_string().contains("def"),
                    "Hover should provide info about 'def' keyword"
                );
            }
            None => panic!("Expected hover information"),
        }
    }

    #[test]
    fn test_document_symbol_handler() {
        let document_manager = setup_test_document_manager();
        let uri = Url::parse("file:///test.ty").unwrap();

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        let symbols =
            document_symbol_handler(&document_manager.read(), &params.text_document).unwrap();

        // We should get some symbols even if it's just the module itself
        assert!(symbols.is_some(), "Should return some symbols");
    }
}
