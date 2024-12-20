use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::Position,
};

use super::utils::TextDocumentPositionParams;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HoverRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    params: HoverParams,
}

impl HoverRequest {
    pub fn get_position(&self) -> &Position {
        &self.params.text_document_position.position
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.text_document_position.text_document.uri
    }

    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HoverParams {
    #[serde(flatten)]
    text_document_position: TextDocumentPositionParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HoverResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: HoverResult,
}

impl HoverResponse {
    pub fn new(id: &RequestId, content: String) -> Self {
        HoverResponse {
            base: ResponseMessageBase::success(id),
            result: HoverResult {
                contents: HoverResultContents::MarkupContent(MarkupContent::Content {
                    kind: Markupkind::Markdown,
                    value: content,
                }),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HoverResult {
    contents: HoverResultContents,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum HoverResultContents {
    SingleMarkedString(MarkedString),
    MultipleMarkedString(Vec<MarkedString>),
    MarkupContent(MarkupContent),
    //WARNING: This is not to spec, the hover.contents also support markup content
    //see: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum MarkedString {
    Content { language: String, value: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum MarkupContent {
    Content { kind: Markupkind, value: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Markupkind {
    Plaintext,
    Markdown,
}

#[cfg(test)]
mod tests {

    use crate::server::lsp::{
        messages::{textdocument_hover::HoverParams, utils::TextDocumentPositionParams},
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::{Position, TextDocumentIdentifier},
    };

    use super::{HoverRequest, HoverResponse};

    #[test]
    fn deserialize() {
        let message = br#"{"params":{"textDocument":{"uri":"file:///dings"},"position":{"character":42,"line":3}},"method":"textDocument/hover","id":2,"jsonrpc":"2.0"}"#;
        let hover_request: HoverRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            hover_request,
            HoverRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string(),
                    },
                    method: "textDocument/hover".to_string(),
                    id: RequestId::Integer(2)
                },
                params: HoverParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: "file:///dings".to_string()
                        },
                        position: Position::new(3, 42)
                    }
                }
            }
        )
    }

    #[test]
    fn serialize() {
        let hover_response =
            HoverResponse::new(&RequestId::Integer(42), "hover content".to_string());
        let expected_message = r#"{"jsonrpc":"2.0","id":42,"result":{"contents":{"kind":"markdown","value":"hover content"}}}"#;
        assert_eq!(
            serde_json::to_string(&hover_response).unwrap(),
            expected_message
        );
    }
}
