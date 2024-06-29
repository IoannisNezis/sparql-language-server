use serde::{Deserialize, Serialize};

use crate::lsp::textdocument::{Position, TextDocumentIdentifier};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}
