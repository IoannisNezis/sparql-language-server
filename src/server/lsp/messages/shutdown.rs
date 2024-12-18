use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    base_types::LSPAny,
    rpc::{RequestId, ResponseMessage},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ShutdownResponse {
    #[serde(flatten)]
    pub base: ResponseMessage,
    pub result: Option<LSPAny>,
}

impl ShutdownResponse {
    pub fn new(id: &RequestId) -> Self {
        Self {
            base: ResponseMessage::success(id),
            result: Some(LSPAny::Null),
        }
    }
}
