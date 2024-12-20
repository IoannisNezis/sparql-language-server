use std::collections::HashMap;

use log::error;
use tree_sitter::{Parser, Tree};

use super::lsp::{
    errors::{ErrorCode, ResponseError},
    textdocument::{TextDocumentItem, TextEdit},
    TextDocumentContentChangeEvent, TraceValue,
};

#[derive(Debug, PartialEq)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

pub struct ServerState {
    pub status: ServerStatus,
    pub trace_value: TraceValue,
    documents: HashMap<String, (TextDocumentItem, Option<Tree>)>,
    parser: Parser,
}

impl ServerState {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
            Ok(()) => {}
            Err(err) => error!("Error while initializing parser: {}", err),
        };
        ServerState {
            status: ServerStatus::Initializing,
            trace_value: TraceValue::Off,
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

    pub(crate) fn get_state(
        &self,
        uri: &String,
    ) -> Result<(&TextDocumentItem, &Tree), ResponseError> {
        match self.documents.get(uri) {
            Some((document, Some(tree))) => Ok((document, tree)),
            Some((_document, None)) => Err(ResponseError::new(
                ErrorCode::InternalError,
                &format!("Could not find parse-tree for \"{}\"", uri),
            )),
            None => Err(ResponseError::new(
                ErrorCode::InternalError,
                &format!("Could not find document \"{}\"", uri),
            )),
        }
    }

    pub(crate) fn get_tree(&self, uri: &str) -> Result<&Tree, ResponseError> {
        match self.documents.get(uri) {
            Some((_document, Some(tree))) => Ok(tree),
            _ => Err(ResponseError::new(
                ErrorCode::InternalError,
                &format!("Could not find parse-tree for \"{}\"", uri),
            )),
        }
    }

    pub(crate) fn get_document(&self, uri: &str) -> Result<&TextDocumentItem, ResponseError> {
        Ok(&self
            .documents
            .get(uri)
            .ok_or(ResponseError::new(
                ErrorCode::InvalidRequest,
                &format!("Requested document \"{}\"could not be found", uri),
            ))?
            .0)
    }
}
