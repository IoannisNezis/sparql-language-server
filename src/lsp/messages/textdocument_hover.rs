use serde::{Deserialize, Serialize};

use crate::{
    lsp::textdocument::{Position, Range, TextDocumentIdentifier},
    rpc::{RequestMessage, ResponseMessage},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HoverRequest {
    #[serde(flatten)]
    base: RequestMessage,
    params: HoverParams,
}

impl HoverRequest {
    pub fn get_position(&self) -> &Position {
        &self.params.text_document_position.position
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.text_document_position.text_document.uri
    }

    pub(crate) fn get_id(&self) -> u32 {
        self.base.id
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HoverParams {
    #[serde(flatten)]
    text_document_position: TextDocumentPositionParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TextDocumentPositionParams {
    text_document: TextDocumentIdentifier,
    position: Position,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HoverResponse {
    #[serde(flatten)]
    base: ResponseMessage,
    result: HoverResult,
}

impl HoverResponse {
    pub fn new(id: u32) -> Self {
        HoverResponse {
            base: ResponseMessage {
                jsonrpc: "2.0".to_string(),
                id,
            },
            result: HoverResult {
                contents: HoverResultContents::MultipleMarkedString(vec![
                    MarkedString::Dings {
                        language: "language".to_string(),
                        value: "hover content 1".to_string(),
                    },
                    MarkedString::Dings {
                        language: "language".to_string(),
                        value: "hover content 2".to_string(),
                    },
                ]),
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
    //WARNING: This is not to spec, the hover.contents also support markup content
    //see: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum MarkedString {
    Dings { language: String, value: String },
}

#[cfg(test)]
mod tests {
    use crate::{
        lsp::{
            messages::textdocument_hover::{HoverParams, Position, TextDocumentPositionParams},
            textdocument::TextDocumentIdentifier,
        },
        rpc::{BaseMessage, RequestMessage},
    };

    use super::{HoverRequest, HoverResponse};

    #[test]
    fn decode_hover_request() {
        let message= b"{\"params\":{\"textDocument\":{\"uri\":\"file:\\/\\/\\/dings\"},\"position\":{\"character\":42,\"line\":3}},\"method\":\"textDocument\\/hover\",\"id\":2,\"jsonrpc\":\"2.0\"}";
        let hover_request: HoverRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            hover_request,
            HoverRequest {
                base: RequestMessage {
                    base: BaseMessage {
                        jsonrpc: "2.0".to_string(),
                        method: "textDocument/hover".to_string()
                    },
                    id: 2
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
    fn encode_hover_response() {
        let hover_response = HoverResponse::new(42);
        let expected_message = "{\"jsonrpc\":\"2.0\",\"id\":42,\"result\":{\"contents\":[{\"language\":\"language\",\"value\":\"hover content 1\"},{\"language\":\"language\",\"value\":\"hover content 2\"}]}}";
        assert_eq!(
            serde_json::to_string(&hover_response).unwrap(),
            expected_message
        );
    }
}
