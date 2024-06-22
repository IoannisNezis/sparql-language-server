use serde::{Deserialize, Serialize};

use crate::{lsp::textdocument::VersionedTextDocumentIdentifier, rpc::BaseMessage};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidChangeTextDocumentNotification {
    #[serde(flatten)]
    base: BaseMessage,
    pub params: DidChangeTextDocumentParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TextDocumentContentChangeEvent {
    // WARNING: This is not to spec, this could also be a incremental diff.
    // See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentContentChangeEvent
    pub text: String,
}
