pub mod state {
    use std::collections::HashMap;

    use log::error;

    use crate::lsp::textdocument::TextDoucmentItem;

    #[derive(Debug)]
    pub struct AnalysisState {
        pub documents: HashMap<String, TextDoucmentItem>,
    }

    impl AnalysisState {
        pub fn new() -> Self {
            Self {
                documents: HashMap::new(),
            }
        }

        pub(crate) fn add_document(&mut self, text_document: TextDoucmentItem) {
            self.documents
                .insert(text_document.uri.clone(), text_document);
        }

        pub(crate) fn change_document(
            &mut self,
            uri: String,
            content_changes: Vec<crate::lsp::TextDocumentContentChangeEvent>,
        ) {
            match self.documents.get_mut(&uri) {
                Some(text_document) => text_document.apply_changes(content_changes),
                None => {
                    error!("Recived changes for unknown document: {}", uri);
                }
            }
        }
    }
}

pub use state::*;
