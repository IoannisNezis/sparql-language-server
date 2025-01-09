use serde::{Deserialize, Serialize};

use crate::server::lsp::{rpc::NotificationMessage, textdocument::TextDocumentItem};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidSaveTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessage,
    pub params: DidSaveTextDocumentParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidSaveTextDocumentParams {
    pub text_document: TextDocumentItem,
    pub text: Option<String>,
}
