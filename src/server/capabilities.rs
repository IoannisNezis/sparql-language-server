use super::lsp::capabilities::{
    CompletionOptions, DiagnosticOptions, DocumentFormattingOptions, ExecuteCommandOptions,
    ServerCapabilities, TextDocumentSyncKind, WorkDoneProgressOptions,
};

pub(super) fn create_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: TextDocumentSyncKind::Incremental,
        hover_provider: true,
        code_action_provider: true,
        execute_command_provider: ExecuteCommandOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: true,
            },
            commands: vec![String::from("publish diagnostics")],
        },
        diagnostic_provider: DiagnosticOptions {
            identifier: "qlue-ls".to_string(),
            inter_file_dependencies: false,
            workspace_diagnostics: false,
        },
        completion_provider: CompletionOptions {
            trigger_characters: vec!["?".to_string()],
        },
        document_formatting_provider: DocumentFormattingOptions {},
    }
}
