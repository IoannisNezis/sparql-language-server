use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    lsp::textdocument::Position,
    rpc::{RequestMessage, ResponseMessage},
};

use super::utils::TextDocumentPositionParams;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletionRequest {
    #[serde(flatten)]
    base: RequestMessage,
    params: CompletionParams,
}

impl CompletionRequest {
    pub fn get_position(&self) -> &Position {
        &self.params.base.position
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.base.text_document.uri
    }

    pub(crate) fn get_id(&self) -> u32 {
        self.base.id
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletionParams {
    #[serde(flatten)]
    base: TextDocumentPositionParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletionResponse {
    #[serde(flatten)]
    base: ResponseMessage,
    result: CompletionResult,
}

impl CompletionResponse {
    pub fn new(id: u32) -> Self {
        CompletionResponse {
            base: ResponseMessage::new(id),
            result: CompletionResult {
                items: vec![
                    // CompletionItem {
                    //     label: "SELECT ${1:*} WHERE {\n  $0\n}".to_string(),
                    //     kind: CompletionItemKind::Snippet,
                    //     detail: "A template for a Select query".to_string(),
                    //     documentation: "doc string".to_string(),
                    //     insert_text_format: InsertTextFormat::Snippet,
                    // },
                    // CompletionItem {
                    //     label: "DELETE {\n ${1}\n}\n WHERE {\n $0\n}".to_string(),
                    //     kind: CompletionItemKind::Snippet,
                    //     detail: "A template for Delete update query".to_string(),
                    //     documentation: "doc string".to_string(),
                    //     insert_text_format: InsertTextFormat::Snippet,
                    // },
                    // CompletionItem {
                    //     label: "namespace:pred ".to_string(),
                    //     kind: CompletionItemKind::Text,
                    //     detail: "Property".to_string(),
                    //     documentation: "".to_string(),
                    //     insert_text_format: InsertTextFormat::PlainText,
                    // },
                ],
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CompletionResult {
    items: Vec<CompletionItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CompletionItem {
    label: String,
    kind: CompletionItemKind,
    detail: String,
    documentation: String,
    insert_text_format: InsertTextFormat,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
enum InsertTextFormat {
    PlainText = 1,
    Snippet = 2,
}

#[cfg(test)]
mod tests {
    use crate::{
        lsp::{
            messages::utils::TextDocumentPositionParams,
            textdocument::{Position, TextDocumentIdentifier},
            CompletionParams,
        },
        rpc::{BaseMessage, RequestMessage},
    };

    use super::{CompletionRequest, CompletionResponse};

    #[test]
    fn deserialize() {
        let message = br#"{"id":4,"params":{"position":{"line":0,"character":0},"context":{"triggerKind":1},"textDocument":{"uri":"file:///dings"}},"jsonrpc":"2.0","method":"textDocument/completion"}"#;
        let completion_request: CompletionRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            completion_request,
            CompletionRequest {
                base: RequestMessage {
                    base: BaseMessage {
                        jsonrpc: "2.0".to_string(),
                        method: "textDocument/completion".to_string()
                    },
                    id: 4
                },
                params: CompletionParams {
                    base: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: "file:///dings".to_string()
                        },
                        position: Position::new(0, 0)
                    }
                }
            }
        )
    }

    #[test]
    fn serialize() {
        let completion_response = CompletionResponse::new(1337);
        let expected_message = r#"{"jsonrpc":"2.0","id":1337,"result":{"items":[]}}"#;
        let actual_message = serde_json::to_string(&completion_response).unwrap();
        assert_eq!(actual_message, expected_message);
    }
}
