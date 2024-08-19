use crate::{
    lsp::{
        analysis::AnalysisState, textdocument::TextDocumentItem, TextDocumentContentChangeEvent,
    },
    message_handler::dispatch,
};

// use crate::message_handler::dispatch;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Server {
    state: ServerState,
}

#[wasm_bindgen]
impl Server {
    pub fn new() -> Self {
        Self {
            state: ServerState::new(),
        }
    }

    pub fn handle_message(&mut self, message: Vec<u8>) -> Option<String> {
        dispatch(&message, &mut self.state)
    }
}

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
