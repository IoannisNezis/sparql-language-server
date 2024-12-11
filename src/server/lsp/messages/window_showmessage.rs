use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::rpc::BaseMessage;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ShowMessageNotification {
    #[serde(flatten)]
    base: BaseMessage,
    params: ShowMessageParams,
}

impl ShowMessageNotification {
    pub fn new(message: String, kind: MessageType) -> Self {
        Self {
            base: BaseMessage::new("window/showMessage".to_string()),
            params: ShowMessageParams { kind, message },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ShowMessageParams {
    #[serde(rename = "type")]
    kind: MessageType,
    message: String,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum MessageType {
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
    Debug = 5,
}
