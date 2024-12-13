use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BaseMessage {
    pub jsonrpc: String,
    pub method: String,
}

impl BaseMessage {
    pub fn new(method: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RequestMessage {
    #[serde(flatten)]
    pub base: BaseMessage,
    // WARNING: This is not to Spec! It could also be a string, or null
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ResponseMessage {
    pub jsonrpc: String,
    // WARNING: This is not to Spec! It could also be a string, or null
    // TODO: Is there a serde way to "omitempty" this field?
    pub id: u32,
}

impl ResponseMessage {
    pub fn new(id: u32) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ResponseError {
    code: u32,
    message: String,
    // WARNING: This is not to Spec! It could also be a number, bool, object, ...
    data: Option<String>,
}

pub fn decode_message(message: &String) -> Result<BaseMessage, String> {
    let request: BaseMessage = serde_json::from_str(&message).expect("A valid Message");
    return Ok(request);
}

#[cfg(test)]
mod tests {

    use crate::server::lsp::rpc::BaseMessage;

    use super::decode_message;

    #[test]
    fn test_decode() {
        let maybe_request = decode_message(
            &r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string(),
        );
        assert_eq!(
            maybe_request,
            Ok(BaseMessage {
                jsonrpc: "2.0".to_owned(),
                method: "initialize".to_owned(),
            })
        );
    }
}
