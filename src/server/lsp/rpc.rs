use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    base_types::LSPAny,
    errors::{ErrorCode, ResponseError},
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum RPCMessage {
    Request(RequestMessage),
    Response(ResponseMessage),
    Notification(NotificationMessage),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    pub jsonrpc: String,
}

impl Message {
    pub fn new() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RequestMessage {
    #[serde(flatten)]
    pub base: Message,
    /**
     * The request id.
     */
    pub id: RequestId,
    /**
     * The method to be invoked.
     */
    pub method: String,
    // The method's params.
    // NOTE: This is omited due to the flatten serde mechanism
    // pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Integer(u32),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Params {
    Array(Vec<LSPAny>),
    Object(HashMap<String, LSPAny>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResponseMessage {
    #[serde(flatten)]
    pub base: Message,
    /**
     * The request id.
     */
    pub id: RequestIdOrNull,
    //The result of a request. This member is REQUIRED on success.
    // This member MUST NOT exist if there was an error invoking the method.
    // NOTE: This is omited due to the flatten serde mechanism
    // pub result: Option<LSPAny>,
    /**
     * The error object in case a request fails.
     */

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum RequestIdOrNull {
    RequestId(RequestId),
    Null,
}

impl ResponseMessage {
    pub fn success(id: &RequestId) -> Self {
        Self {
            base: Message::new(),
            id: RequestIdOrNull::RequestId(id.clone()),
            error: None,
        }
    }

    pub(crate) fn error(id: &RequestId, error_code: ErrorCode, message: &str) -> Self {
        Self {
            base: Message::new(),
            id: RequestIdOrNull::RequestId(id.clone()),
            error: Some(ResponseError::new(error_code, message)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NotificationMessage {
    #[serde(flatten)]
    pub base: Message,
    /**
     * The method to be invoked.
     */
    pub method: String,
    // The notification's params.
    //pub params: Option<Params>,
}
impl NotificationMessage {
    pub(crate) fn new(method: &str) -> Self {
        Self {
            base: Message::new(),
            method: method.to_string(),
        }
    }
}

pub fn deserialize_message(message: &String) -> Result<Message, String> {
    let request: Message = serde_json::from_str(&message).expect("A valid Message");
    return Ok(request);
}

pub fn deserialize_request(message: &String) -> Result<RPCMessage, String> {
    let request = serde_json::from_str(&message).map_err(|_error| "Parse failed".to_string())?;
    return Ok(request);
}

#[cfg(test)]
mod tests {

    use crate::server::lsp::rpc::{
        deserialize_request, Message, NotificationMessage, RequestId, RequestMessage,
    };

    use super::RPCMessage;

    #[test]
    fn serialize() {
        let message = RPCMessage::Request(RequestMessage {
            base: Message::new(),
            id: RequestId::Integer(1),
            method: "initialize".to_owned(),
        });
        assert_eq!(
            serde_json::to_string(&message).unwrap(),
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#
        );
    }

    #[test]
    fn deserialize() {
        let maybe_initialized = deserialize_request(
            &r#"{"params":{},"jsonrpc":"2.0","method":"initialized"}"#.to_string(),
        );
        assert_eq!(
            maybe_initialized,
            Ok(RPCMessage::Notification(NotificationMessage {
                base: Message::new(),
                method: "initialized".to_owned(),
            }))
        );
        let maybe_request = deserialize_request(
            &r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string(),
        );
        assert_eq!(
            maybe_request,
            Ok(RPCMessage::Request(RequestMessage {
                base: Message::new(),
                id: RequestId::Integer(1),
                method: "initialize".to_owned(),
            }))
        );
    }
}
