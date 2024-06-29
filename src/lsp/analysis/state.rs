use std::collections::HashMap;

use log::{error, info};
use tree_sitter::{Parser, Tree};

use crate::lsp::textdocument::TextDocumentItem;

pub struct AnalysisState {
    documents: HashMap<String, TextDocumentItem>,
    trees: HashMap<String, Option<Tree>>,
    parser: Parser,
}

impl AnalysisState {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_sparql::language()) {
            Ok(()) => {
                info!("Successfully initialized Parser")
            }
            Err(err) => error!("Error while initializing parser: {}", err),
        };
        Self {
            documents: HashMap::new(),
            trees: HashMap::new(),
            parser,
        }
    }

    pub(crate) fn add_document(&mut self, text_document: TextDocumentItem) {
        let tree = self.parser.parse(&text_document.text, None);

        let uri = text_document.uri.clone();
        self.trees.insert(uri.clone(), tree);
        self.documents.insert(uri, text_document);
    }

    pub(crate) fn change_document(
        &mut self,
        uri: String,
        content_changes: Vec<crate::lsp::TextDocumentContentChangeEvent>,
    ) {
        match self.documents.get_mut(&uri) {
            Some(text_document) => {
                text_document.apply_changes(content_changes);
                let tree = self.parser.parse(&text_document.text, None);
                self.trees.insert(uri.clone(), tree);
            }
            None => {
                error!("Recived changes for unknown document: {}", uri);
            }
        }
    }

    pub(crate) fn get_tree(&self, uri: &str) -> Option<&Tree> {
        self.trees.get(uri)?.as_ref()
    }
}
