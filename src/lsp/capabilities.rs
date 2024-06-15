use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub text_document_sync: Option<TextDocumentSyncKind>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

#[cfg(test)]
mod tests {
    use super::{ServerCapabilities, TextDocumentSyncKind};

    #[test]
    fn test_serialization() {
        let sync_kind = TextDocumentSyncKind::Full;
        let server_capabilities = ServerCapabilities {
            text_document_sync: Some(sync_kind),
        };

        let serialized = serde_json::to_string(&server_capabilities).unwrap();

        assert_eq!(serialized, "{\"textDocumentSync\":1}");
    }
}
