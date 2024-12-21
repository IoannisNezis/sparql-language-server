use std::{any::type_name, collections::HashMap};

use log::error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

impl RPCMessage {
    pub fn get_method(&self) -> Option<&str> {
        match self {
            RPCMessage::Notification(notification) => Some(&notification.method),
            RPCMessage::Request(request) => Some(&request.method),
            RPCMessage::Response(_) => None,
        }
    }

    pub fn parse<T>(&self) -> Result<T, ResponseError>
    where
        T: Serialize + DeserializeOwned,
    {
        match serde_json::to_string(self) {
            Ok(serialized_message) => serde_json::from_str(&serialized_message).map_err(|error| {
                ResponseError::new(
                    ErrorCode::ParseError,
                    &format!(
                        "Could not deserialize RPC-message \"{}\"\n\n{}",
                        type_name::<T>(),
                        error
                    ),
                )
            }),
            Err(error) => Err(ResponseError::new(
                ErrorCode::ParseError,
                &format!("Could not serialize RPC-message\n\n{}", error),
            )),
        }
    }
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

// NOTE: The only purpouse of thi struct is to recover
// the id of a message in case a error occurs
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RecoverId {
    pub id: RequestId,
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
    /**
     * The method's params.
     */
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RequestMessageBase {
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
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Integer(u32),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Params {
    Array(Vec<serde_json::Value>),
    Object(HashMap<String, serde_json::Value>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResponseMessage {
    #[serde(flatten)]
    pub base: Message,
    /**
     * The request id.
     */
    pub id: RequestIdOrNull,
    /**
     * The result of a request. This member is REQUIRED on success.
     * This member MUST NOT exist if there was an error invoking the method.
     */
    pub result: Option<LSPAny>,
    /**
     * The error object in case a request fails.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl ResponseMessage {
    pub fn error(id: RequestIdOrNull, error: ResponseError) -> Self {
        Self {
            base: Message::new(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum RequestIdOrNull {
    RequestId(RequestId),
    Null,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResponseMessageBase {
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

impl ResponseMessageBase {
    pub fn success(id: &RequestId) -> Self {
        Self {
            base: Message::new(),
            id: RequestIdOrNull::RequestId(id.clone()),
            error: None,
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
    /*
     * The notification's params.
     */
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NotificationMessageBase {
    #[serde(flatten)]
    pub base: Message,
    /**
     * The method to be invoked.
     */
    pub method: String,
}

impl NotificationMessageBase {
    pub(crate) fn new(method: &str) -> Self {
        Self {
            base: Message::new(),
            method: method.to_string(),
        }
    }
}

pub fn deserialize_message(message: &String) -> Result<RPCMessage, ResponseError> {
    serde_json::from_str(&message).map_err(|error| {
        error!(
            "Error while serializing message:\n{}-----------------------\n{}",
            error, message,
        );
        ResponseError::new(
            ErrorCode::ParseError,
            &format!("Could not serialize RPC-Message:\n\n{}", error),
        )
    })
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use serde_json::json;

    use crate::server::lsp::{
        base_types::LSPAny,
        rpc::{
            deserialize_message, Message, NotificationMessage, Params, RequestId, RequestIdOrNull,
            RequestMessage, ResponseMessage,
        },
    };

    use super::RPCMessage;

    #[test]
    fn serialize() {
        let message = RPCMessage::Request(RequestMessage {
            base: Message::new(),
            id: RequestId::Integer(1),
            method: "initialize".to_owned(),
            params: Some(Params::Array(vec![])),
        });
        let serialized = serde_json::to_string(&message).unwrap();
        assert_eq!(
            serialized,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":[]}"#
        );
    }

    #[test]
    fn deserialize_notification() {
        let maybe_initialized = deserialize_message(
            &r#"{"params":{"a":2},"jsonrpc":"2.0","method":"initialized"}"#.to_string(),
        )
        .unwrap();
        assert_eq!(
            maybe_initialized,
            RPCMessage::Notification(NotificationMessage {
                base: Message::new(),
                method: "initialized".to_owned(),
                params: Some(Params::Object(HashMap::from([("a".to_string(), json!(2))])))
            })
        );
    }

    #[test]
    fn deserialize_request() {
        let maybe_request = deserialize_message(
            &r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"a":[1,2,3]}}"#.to_string(),
        );
        assert_eq!(
            maybe_request,
            Ok(RPCMessage::Request(RequestMessage {
                base: Message::new(),
                id: RequestId::Integer(1),
                method: "initialize".to_owned(),
                params: Some(Params::Object(HashMap::from([(
                    "a".to_string(),
                    json!([1, 2, 3])
                )])))
            }))
        );
    }

    #[test]
    fn deserialize_response() {
        let maybe_response =
            deserialize_message(&r#"{"jsonrpc":"2.0","id":1,"result":{"a":1}}"#.to_string());
        assert_eq!(
            maybe_response,
            Ok(RPCMessage::Response(ResponseMessage {
                base: Message::new(),
                id: RequestIdOrNull::RequestId(RequestId::Integer(1)),
                error: None,
                result: Some(LSPAny::LSPObject(HashMap::from([(
                    "a".to_string(),
                    LSPAny::Uinteger(1)
                )])))
            }))
        );
    }
}
