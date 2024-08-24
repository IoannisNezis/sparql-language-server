use nom::{
    bytes::complete::{tag, take_while},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub content_length: usize,
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BaseMessage {
    pub jsonrpc: String,
    pub method: String,
}

impl BaseMessage {
    pub fn new(method: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
        }
    }
}

fn parse_header(input: &str) -> IResult<&str, usize> {
    let (input, _) = tag("Content-Length: ")(input)?;
    let (input, number) = take_while(|c: char| c.is_digit(10))(input)?;
    Ok((input, number.parse().unwrap()))
}

impl Header {
    pub fn from_string(string: String) -> Result<Header, String> {
        let (_rest, content_length) = parse_header(string.as_str()).expect("Expected valid header");
        Ok(Header { content_length })
    }
}

pub fn decode_message(msg: &Vec<u8>) -> Result<BaseMessage, String> {
    let msg_string = String::from_utf8(msg.to_vec()).unwrap();
    let request: BaseMessage = serde_json::from_str(&msg_string).expect("A valid Message");
    return Ok(request);
}

#[cfg(test)]
mod tests {
    use crate::rpc::{BaseMessage, Header};

    use super::decode_message;

    #[test]
    fn header_parses() {
        let header_string: String = "Content-Length: 12345\r\n\r\n".to_owned();
        assert_eq!(
            Header::from_string(header_string),
            Ok(Header {
                content_length: 12345
            })
        );
    }

    #[test]
    fn test_decode() {
        let maybe_request = decode_message(
            &b"{\"jsonrpc\": \"2.0\",\"id\": 1, \"method\": \"initialize\", \"params\": {}}"
                .to_vec(),
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
