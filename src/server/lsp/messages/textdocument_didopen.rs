use serde::{Deserialize, Serialize};

use crate::server::lsp::{rpc::NotificationMessage, textdocument::TextDocumentItem};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidOpenTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessage,
    pub params: DidOpenTextDocumentPrams,
}

impl DidOpenTextDocumentNotification {
    pub fn get_text_document(self) -> TextDocumentItem {
        self.params.text_document
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentPrams {
    pub text_document: TextDocumentItem,
}
