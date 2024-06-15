pub mod state {
    use std::collections::HashMap;

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
    }
}

pub use state::*;
