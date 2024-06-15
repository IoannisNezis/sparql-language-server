use serde::{Deserialize, Serialize};

use crate::{lsp::textdocument::TextDoucmentItem, rpc::BaseMessage};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidOpenTextDocumentNotification {
    #[serde(flatten)]
    base: BaseMessage,
    pub params: DidOpenTextDocumentPrams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentPrams {
    pub text_document: TextDoucmentItem,
}
