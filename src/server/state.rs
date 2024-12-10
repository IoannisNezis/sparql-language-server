use std::collections::HashMap;

use log::{error, info};
use tree_sitter::{Parser, Tree};

use super::lsp::{
    textdocument::{TextDocumentItem, TextEdit},
    TextDocumentContentChangeEvent,
};

#[derive(Debug)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

pub struct ServerState {
    pub status: ServerStatus,
    pub analysis_state: AnalysisState,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            analysis_state: AnalysisState::new(),
        }
    }

    pub fn add_document(&mut self, document: TextDocumentItem) {
        self.analysis_state.add_document(document);
    }

    pub(crate) fn change_document(
        &mut self,
        document_uri: String,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        self.analysis_state
            .change_document(document_uri, content_changes)
    }
}

pub struct AnalysisState {
    documents: HashMap<String, (TextDocumentItem, Option<Tree>)>,
    parser: Parser,
}

impl AnalysisState {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
            Ok(()) => {
                info!("Successfully initialized Parser")
            }
            Err(err) => error!("Error while initializing parser: {}", err),
        };
        Self {
            documents: HashMap::new(),
            parser,
        }
    }

    pub(crate) fn add_document(&mut self, text_document: TextDocumentItem) {
        let tree = self.parser.parse(&text_document.text, None);

        let uri = text_document.uri.clone();
        self.documents.insert(uri.clone(), (text_document, tree));
    }

    pub(crate) fn change_document(
        &mut self,
        uri: String,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        match self.documents.get_mut(&uri) {
            Some((text_document, old_tree)) => {
                text_document.apply_text_edits(
                    content_changes
                        .into_iter()
                        .map(|change_event| {
                            TextEdit::from_text_document_content_change_event(change_event)
                        })
                        .collect::<Vec<TextEdit>>(),
                );
                let tree = self.parser.parse(&text_document.text, None);
                *old_tree = tree;
            }
            None => {
                error!("Recived changes for unknown document: {}", uri);
            }
        }
    }

    // pub(crate) fn documents(&self) -> impl Iterator<Item = &String> {
    //     self.documents.keys()
    // }

    pub(crate) fn get_state(&self, uri: &String) -> Option<&(TextDocumentItem, Option<Tree>)> {
        self.documents.get(uri)
    }

    pub(crate) fn get_tree(&self, uri: &str) -> Option<&Tree> {
        self.documents.get(uri)?.1.as_ref()
    }
}
