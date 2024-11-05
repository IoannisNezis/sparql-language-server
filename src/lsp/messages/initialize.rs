use serde::{Deserialize, Serialize};

use crate::{
    lsp::capabilities::{self, ServerCapabilities},
    rpc::{RequestMessage, ResponseMessage},
    server::Server,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    // WARNING: This is not to Spec! It's optional
    pub client_info: ClientInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InitializeRequest {
    #[serde(flatten)]
    pub base: RequestMessage,
    pub params: InitializeParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InitializeResonse {
    #[serde(flatten)]
    pub base: ResponseMessage,
    pub result: InitializeResult,
}

impl InitializeResonse {
    pub fn new(id: u32, server: &Server) -> Self {
        InitializeResonse {
            base: ResponseMessage {
                jsonrpc: "2.0".to_string(),
                id,
            },
            result: InitializeResult {
                capabilities: server.capabilities.clone(),
                server_info: Some(server.server_info.clone()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lsp::ClientInfo,
        rpc::{BaseMessage, RequestMessage},
    };

    use super::{InitializeParams, InitializeRequest};

    #[test]
    fn deserialize() {
        let message = b"{\"jsonrpc\": \"2.0\",\"id\": 1, \"method\": \"initialize\", \"params\": { \"clientInfo\": {\"name\": \"dings\", \"version\": \"42.1\"}}}";
        let init_request: InitializeRequest = serde_json::from_slice(message).unwrap();
        assert_eq!(
            init_request,
            InitializeRequest {
                base: RequestMessage {
                    base: BaseMessage {
                        jsonrpc: "2.0".to_string(),
                        method: "initialize".to_string()
                    },
                    id: 1,
                },
                params: InitializeParams {
                    client_info: ClientInfo {
                        name: "dings".to_string(),
                        version: Some("42.1".to_string())
                    }
                }
            }
        );
    }
}
