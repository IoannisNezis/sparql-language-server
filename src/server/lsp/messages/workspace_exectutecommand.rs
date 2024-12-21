use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    base_types::LSPAny,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecuteCommandRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: ExecuteCommandParams,
}
impl ExecuteCommandRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecuteCommandParams {
    pub command: String,
    pub arguments: Option<Vec<LSPAny>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecuteCommandResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: LSPAny,
}
impl ExecuteCommandResponse {
    pub(crate) fn new(id: &RequestId) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: LSPAny::Null,
        }
    }
}
