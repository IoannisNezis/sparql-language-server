use crate::lsp::{analysis::AnalysisState, DidOpenTextDocumentPrams};

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

    pub fn add_document(&mut self, params: DidOpenTextDocumentPrams) {
        self.analysis_state.add_document(params.text_document);
    }
}
