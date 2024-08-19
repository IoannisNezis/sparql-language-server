mod completion;
mod formatting;
use std::process::exit;

use log::{error, info, warn};

use crate::{
    lsp::{
        analysis::get_token, textdocument::TextDocumentItem, CompletionRequest,
        DidChangeTextDocumentNotification, DidOpenTextDocumentNotification, FormattingRequest,
        HoverRequest, HoverResponse, InitializeRequest, InitializeResonse, ShutdownResponse,
    },
    message_handler::completion::handly_completion_request,
    rpc::{self, RequestMessage, ResponseMessage},
    server::{ServerState, ServerStatus},
};

use self::formatting::handle_format_request;

pub fn dispatch(bytes: &Vec<u8>, state: &mut ServerState) -> Option<String> {
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
                    return Some(serde_json::to_string(&initialize_response).unwrap());
                }
                Err(error) => {
                    error!("Could not parse initialize request: {:?}", error);
                    return None;
                }
            },
            "initialized" => {
                info!("initialization completed");
                state.status = ServerStatus::Running;
                return None;
            }
            "shutdown" => match serde_json::from_slice::<RequestMessage>(bytes) {
                Ok(shutdown_request) => {
                    info!("recieved shutdown request, preparing to shut down");
                    let response = ShutdownResponse::new(shutdown_request.id);
                    state.status = ServerStatus::ShuttingDown;
                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!("Could not parse shutdown request: {:?}", error);
                    return None;
                }
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
                        return None;
                    }
                    Err(error) => {
                        error!("Could not parse textDocument/didOpen request: {:?}", error);
                        return None;
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
                        );
                        return None;
                    }
                    Err(error) => {
                        error!(
                            "Could not parse textDocument/didChange notification: {:?}",
                            error
                        );
                        return None;
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
                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!("Could not parse textDocument/hover request: {:?}", error);
                    return None;
                }
            },
            "textDocument/completion" => match serde_json::from_slice::<CompletionRequest>(bytes) {
                Ok(completion_request) => {
                    info!(
                        "Received completion request for {} {}",
                        completion_request.get_document_uri(),
                        completion_request.get_position()
                    );
                    let response = handly_completion_request(completion_request, state);
                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!(
                        "Could not parse textDocument/completion request: {:?}",
                        error
                    );
                    return None;
                }
            },
            "textDocument/formatting" => match serde_json::from_slice::<FormattingRequest>(bytes) {
                Ok(formatting_request) => {
                    let response = handle_format_request(formatting_request, state);
                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!(
                        "Could not parse textDocument/formatting request: {:?}",
                        error
                    );
                    return None;
                }
            },
            unknown_method => {
                warn!(
                    "Received message with unknown method \"{}\": {:?}",
                    unknown_method,
                    String::from_utf8(bytes.to_vec()).unwrap()
                );
                return None;
            }
        };
    } else {
        error!("An error occured while parsing the request content");
        return None;
    }
}
