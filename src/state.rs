use crate::lsp::{analysis::AnalysisState, DidOpenTextDocumentNotification};

#[derive(Debug)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

pub struct ServerState {
    pub status: ServerStatus,
    analysis_state: AnalysisState,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            analysis_state: AnalysisState::new(),
        }
    }

    pub fn handle_did_open(&mut self, message: DidOpenTextDocumentNotification) {
        self.analysis_state
            .add_document(message.params.text_document);
    }

    pub(crate) fn handle_did_change(
        &mut self,
        did_change_notification: crate::lsp::DidChangeTextDocumentNotification,
    ) {
        self.analysis_state.change_document(
            did_change_notification.params.text_document.base.uri,
            did_change_notification.params.content_changes,
        )
    }
}
