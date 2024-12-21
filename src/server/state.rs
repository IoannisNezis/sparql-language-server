use std::collections::HashMap;

use tree_sitter::Tree;

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
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            trace_value: TraceValue::Off,
            documents: HashMap::new(),
        }
    }

    pub(super) fn add_document(&mut self, text_document: TextDocumentItem, tree: Option<Tree>) {
        self.documents
            .insert(text_document.uri.clone(), (text_document, tree));
    }

    pub(super) fn change_document(
        &mut self,
        uri: &String,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Option<&TextDocumentItem> {
        let document = &mut self.documents.get_mut(uri)?.0;
        document.apply_text_edits(
            content_changes
                .into_iter()
                .map(|change_event| TextEdit::from_text_document_content_change_event(change_event))
                .collect::<Vec<TextEdit>>(),
        );
        return Some(document);
    }

    pub(super) fn get_state(&self, uri: &str) -> Result<(&TextDocumentItem, &Tree), ResponseError> {
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

    pub(super) fn get_tree(&self, uri: &str) -> Result<&Tree, ResponseError> {
        match self.documents.get(uri) {
            Some((_document, Some(tree))) => Ok(tree),
            _ => Err(ResponseError::new(
                ErrorCode::InternalError,
                &format!("Could not find parse-tree for \"{}\"", uri),
            )),
        }
    }

    pub(super) fn get_document(&self, uri: &str) -> Result<&TextDocumentItem, ResponseError> {
        Ok(&self
            .documents
            .get(uri)
            .ok_or(ResponseError::new(
                ErrorCode::InvalidRequest,
                &format!("Requested document \"{}\"could not be found", uri),
            ))?
            .0)
    }

    pub(super) fn update_tree(
        &mut self,
        uri: &str,
        new_tree: Option<Tree>,
    ) -> Result<(), ResponseError> {
        if self.documents.get(uri).is_some() {
            self.documents
                .entry(uri.to_string())
                .and_modify(|(_document, old_tree)| *old_tree = new_tree);
            Ok(())
        } else {
            Err(ResponseError::new(
                ErrorCode::InternalError,
                &format!("Could not update parse-tree, no entry for \"{}\"", uri),
            ))
        }
    }
}
