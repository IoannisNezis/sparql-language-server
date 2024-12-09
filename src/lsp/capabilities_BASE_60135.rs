use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    text_document_sync: TextDocumentSyncKind,
    hover_provider: bool,
    completion_provider: CompletionOptions,
    document_formatting_provider: DocumentFormattingOptions,
    diagnostic_provider: DiagnosticOptions,
}

impl ServerCapabilities {
    pub fn new() -> Self {
        Self {
            text_document_sync: TextDocumentSyncKind::Full,
            hover_provider: true,
            completion_provider: CompletionOptions::new(),
            document_formatting_provider: DocumentFormattingOptions {},
            diagnostic_provider: DiagnosticOptions::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DiagnosticOptions {
    identifier: String,
    inter_file_dependencies: bool,
    workspace_diagnostics: bool,
}

impl DiagnosticOptions {
    fn new() -> Self {
        Self {
            identifier: "sparql-ls".to_string(),
            inter_file_dependencies: false,
            workspace_diagnostics: false,
        }
    }
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CompletionOptions {
    // WARNING: This is not to spec, there are more optional options:
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionOptions
    trigger_characters: Vec<String>,
}

impl CompletionOptions {
    fn new() -> Self {
        Self {
            trigger_characters: vec!["?".to_string()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentFormattingOptions {
    // WARNING: This could also inherit WorkDoneProgressOptions (not implemented yet).
}

#[cfg(test)]
mod tests {
    use super::ServerCapabilities;

    #[test]
    fn serialize() {
        let server_capabilities = ServerCapabilities::new();

        let serialized = serde_json::to_string(&server_capabilities).unwrap();

        assert_eq!(
            serialized,
            "{\"textDocumentSync\":1,\"hoverProvider\":true,\"completionProvider\":{\"triggerCharacters\":[\"?\"]},\"documentFormattingProvider\":{},\"diagnosticProvider\":{\"identifier\":\"sparql-ls\",\"inter_file_dependencies\":false,\"workspace_diagnostics\":false}}"
        );
    }
}
