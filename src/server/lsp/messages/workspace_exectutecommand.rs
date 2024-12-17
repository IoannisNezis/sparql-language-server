use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    base_types::LSPAny,
    rpc::{RequestMessage, ResponseMessage},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecuteCommandRequest {
    #[serde(flatten)]
    pub base: RequestMessage,
    pub params: ExecuteCommandParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecuteCommandParams {
    pub command: String,
    pub arguments: Option<Vec<LSPAny>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecuteCommandResponse {
    #[serde(flatten)]
    pub base: ResponseMessage,
    pub result: LSPAny,
}
