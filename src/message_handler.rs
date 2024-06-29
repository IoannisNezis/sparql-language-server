use std::process::exit;

use log::{error, info, warn};
use serde::Serialize;

use crate::{
    lsp::{
        analysis::get_token, textdocument::TextDocumentItem, CompletionRequest, CompletionResponse,
        DidChangeTextDocumentNotification, DidOpenTextDocumentNotification, HoverRequest,
        HoverResponse, InitializeRequest, InitializeResonse,
    },
    rpc::{self, RequestMessage, ResponseMessage},
    state::{ServerState, ServerStatus},
};

pub fn handle_message(bytes: &Vec<u8>, state: &mut ServerState) {
    if let Ok(message) = rpc::decode_message(bytes) {
        match message.method.as_str() {
            "initialize" => match serde_json::from_slice::<InitializeRequest>(bytes) {
                Ok(initialize_request) => {
                    info!(
                        "Connected to: {} {}",
                        initialize_request.params.client_info.name,
                        initialize_request
                            .params
                            .client_info
                            .version
                            .unwrap_or("no version specified".to_string())
                    );
                    let initialize_response = InitializeResonse::new(initialize_request.base.id);
                    send_message(&initialize_response);
                }
                Err(error) => error!("Could not parse initialize request: {:?}", error),
            },
            "initialized" => {
                info!("initialization completed");
                state.status = ServerStatus::Running;
            }
            "shutdown" => match serde_json::from_slice::<RequestMessage>(bytes) {
                Ok(shutdown_request) => {
                    info!("recieved shutdown request, preparing to shut down");
                    let response = ResponseMessage {
                        jsonrpc: "2.0".to_string(),
                        id: shutdown_request.id,
                    };
                    send_message(&response);
                    state.status = ServerStatus::ShuttingDown;
                }
                Err(error) => error!("Could not parse shutdown request: {:?}", error),
            },
            "exit" => {
                info!("recieved exit notification, shutting down!");
                exit(0);
            }
            "textDocument/didOpen" => {
                match serde_json::from_slice::<DidOpenTextDocumentNotification>(bytes) {
                    Ok(did_open_notification) => {
                        info!(
                            "opened text document: \"{}\"\n{}",
                            did_open_notification.params.text_document.uri,
                            did_open_notification.params.text_document.text
                        );
                        let text_document: TextDocumentItem =
                            did_open_notification.get_text_document();
                        state.add_document(text_document);
                    }
                    Err(error) => {
                        error!("Could not parse textDocument/didOpen request: {:?}", error)
                    }
                }
            }
            "textDocument/didChange" => {
                match serde_json::from_slice::<DidChangeTextDocumentNotification>(bytes) {
                    Ok(did_change_notification) => {
                        info!(
                            "text document changed: {}",
                            did_change_notification.params.text_document.base.uri
                        );
                        state.change_document(
                            did_change_notification.params.text_document.base.uri,
                            did_change_notification.params.content_changes,
                        )
                    }
                    Err(error) => {
                        error!(
                            "Could not parse textDocument/didChange notification: {:?}",
                            error
                        )
                    }
                }
            }
            "textDocument/hover" => match serde_json::from_slice::<HoverRequest>(bytes) {
                Ok(hover_request) => {
                    info!(
                        "recieved hover request for {} {}",
                        hover_request.get_document_uri(),
                        hover_request.get_position()
                    );
                    let response_content = get_token(
                        &state.analysis_state,
                        hover_request.get_document_uri(),
                        hover_request.get_position(),
                    );

                    let response = HoverResponse::new(hover_request.get_id(), response_content);
                    send_message(&response);
                }
                Err(error) => error!("Could not parse textDocument/hover request: {:?}", error),
            },
            "textDocument/completion" => match serde_json::from_slice::<CompletionRequest>(bytes) {
                Ok(completion_request) => {
                    info!("{}", String::from_utf8(bytes.to_vec()).unwrap());
                    info!(
                        "Received completion request for {} {}",
                        completion_request.get_document_uri(),
                        completion_request.get_position()
                    );
                    let response = CompletionResponse::new(completion_request.get_id());
                    send_message(&response);
                }
                Err(error) => error!(
                    "Could not parse textDocument/completion request: {:?}",
                    error
                ),
            },
            unknown_method => {
                warn!(
                    "Received message with unknown method \"{}\": {:?}",
                    unknown_method,
                    String::from_utf8(bytes.to_vec()).unwrap()
                );
            }
        };
    } else {
        error!("An error occured while parsing the request content");
    }
}

// TODO: This trait should be narrowed down, Serialize is not enougth to be jsonrpc message.
fn send_message<T: Serialize>(message_body: &T) {
    let message_body_string = rpc::encode(&message_body);
    // info!("sending: {}", message_body_string);
    println!(
        "Content-Length: {}\r\n\r\n{}",
        message_body_string.len(),
        message_body_string
    );
}
