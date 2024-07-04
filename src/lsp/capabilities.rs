use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    text_document_sync: TextDocumentSyncKind,
    hover_provider: bool,
    completion_provider: CompletionOptions,
    document_formatting_provider: DocumentFormattingOptions,
}

impl ServerCapabilities {
    pub fn new() -> Self {
        Self {
            text_document_sync: TextDocumentSyncKind::Full,
            hover_provider: true,
            completion_provider: CompletionOptions {},
            document_formatting_provider: DocumentFormattingOptions {},
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
struct CompletionOptions {
    // WARNING: This is not to spec, there are multiple optional options:
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionOptions
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
            "{\"textDocumentSync\":1,\"hoverProvider\":true,\"completionProvider\":{},\"documentFormattingProvider\":{}}"
        );
    }
}
