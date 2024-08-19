use crate::rpc::ResponseMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ShutdownResponse {
    #[serde(flatten)]
    pub base: ResponseMessage,
    pub result: Option<()>,
}

impl ShutdownResponse {
    pub fn new(id: u32) -> Self {
        Self {
            base: ResponseMessage {
                jsonrpc: "2.0".to_string(),
                id,
            },
            result: None,
        }
    }
}
