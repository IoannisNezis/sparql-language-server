use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    base_types::LSPAny,
    rpc::{RequestId, ResponseMessageBase},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ShutdownResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: Option<LSPAny>,
}

impl ShutdownResponse {
    pub fn new(id: &RequestId) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(LSPAny::Null),
        }
    }
}
