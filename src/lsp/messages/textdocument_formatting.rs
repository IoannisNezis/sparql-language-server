use serde::{Deserialize, Serialize};

use crate::{
    lsp::textdocument::{TextDocumentIdentifier, TextEdit},
    rpc::{RequestMessage, ResponseMessage},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FormattingRequest {
    #[serde(flatten)]
    base: RequestMessage,
    params: DocumentFormattingParams,
}
impl FormattingRequest {
    pub(crate) fn get_id(&self) -> u32 {
        self.base.id
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.text_document.uri
    }

    pub(crate) fn get_options(&self) -> &FormattingOptions {
        &self.params.options
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DocumentFormattingParams {
    text_document: TextDocumentIdentifier,
    options: FormattingOptions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FormattingOptions {
    pub tab_size: u8,
    pub insert_spaces: bool,
    // TODO: further options
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FormattingResponse {
    #[serde(flatten)]
    base: ResponseMessage,
    result: Vec<TextEdit>,
}
impl FormattingResponse {
    pub(crate) fn new(id: u32, text_edits: Vec<TextEdit>) -> Self {
        Self {
            base: ResponseMessage::new(id),
            result: text_edits,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lsp::{
            messages::textdocument_formatting::{DocumentFormattingParams, FormattingOptions},
            textdocument::TextDocumentIdentifier,
        },
        rpc::{BaseMessage, RequestMessage},
    };

    use super::FormattingRequest;

    #[test]
    fn decode_formatting_request() {
        let message = br#"{"jsonrpc":"2.0","method":"textDocument/formatting","id":2,"params":{"textDocument":{"uri":"file:///dings"},"options":{"tabSize":2,"insertSpaces":true}}}"#;
        let request = serde_json::from_slice::<FormattingRequest>(message).unwrap();

        assert_eq!(
            request,
            FormattingRequest {
                base: RequestMessage {
                    base: BaseMessage {
                        jsonrpc: "2.0".to_string(),
                        method: "textDocument/formatting".to_string()
                    },
                    id: 2
                },
                params: DocumentFormattingParams {
                    text_document: TextDocumentIdentifier {
                        uri: "file:///dings".to_string()
                    },
                    options: FormattingOptions {
                        tab_size: 2,
                        insert_spaces: true
                    }
                }
            }
        );
    }

    #[test]
    fn encode_formatting_response() {
        todo!()
    }
}
