use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub text_document_sync: TextDocumentSyncKind,
    pub hover_provider: bool,
    pub completion_provider: CompletionOptions,
    pub document_formatting_provider: DocumentFormattingOptions,
    pub diagnostic_provider: DiagnosticOptions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DiagnosticOptions {
    pub identifier: String,
    pub inter_file_dependencies: bool,
    pub workspace_diagnostics: bool,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionOptions {
    // WARNING: This is not to spec, there are more optional options:
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionOptions
    pub trigger_characters: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DocumentFormattingOptions {
    // WARNING: This could also inherit WorkDoneProgressOptions (not implemented yet).
}

#[cfg(test)]
mod tests {
    use crate::lsp::capabilities::{
        CompletionOptions, DiagnosticOptions, DocumentFormattingOptions, TextDocumentSyncKind,
    };

    use super::ServerCapabilities;

    #[test]
    fn serialize() {
        let server_capabilities = ServerCapabilities {
            text_document_sync: TextDocumentSyncKind::Full,
            hover_provider: true,
            completion_provider: CompletionOptions {
                trigger_characters: vec!["?".to_string()],
            },
            document_formatting_provider: DocumentFormattingOptions {},
            diagnostic_provider: DiagnosticOptions {
                identifier: "my-ls".to_string(),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
            },
        };

        let serialized = serde_json::to_string(&server_capabilities).unwrap();

        assert_eq!(
            serialized,
            "{\"textDocumentSync\":1,\"hoverProvider\":true,\"completionProvider\":{\"triggerCharacters\":[\"?\"]},\"documentFormattingProvider\":{},\"diagnosticProvider\":{\"identifier\":\"my-ls\",\"inter_file_dependencies\":false,\"workspace_diagnostics\":false}}"
        );
    }
}
