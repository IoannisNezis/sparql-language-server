mod code_action;
mod commands;
mod completion;
mod diagnostic;
mod formatting;
mod hovering;
mod initialize;

use code_action::generate_code_actions;
use commands::handle_command;
use completion::handel_completion_request;
use hovering::handle_hover_request;
use initialize::handle_initialize_request;
use log::{debug, error, info, warn};
use std::process::exit;

pub use diagnostic::*;
pub use formatting::format_raw;

use crate::server::lsp::errors::ErrorCode;

use self::formatting::handle_format_request;

use super::{
    lsp::{
        rpc::{self, Message, RPCMessage, RequestIdOrNull, RequestMessage, ResponseMessage},
        textdocument::TextDocumentItem,
        CodeActionRequest, CodeActionResponse, CompletionRequest, DiagnosticRequest,
        DiagnosticResponse, DidChangeTextDocumentNotification, DidOpenTextDocumentNotification,
        ExecuteCommandRequest, ExecuteCommandResponse, FormattingRequest, FormattingResponse,
        HoverRequest, InitializeRequest, SetTraceNotification, ShutdownResponse,
    },
    state::ServerStatus,
    Server,
};

pub fn dispatch(server: &mut Server, message_string: String) -> Option<String> {
    match rpc::deserialize_request(&message_string) {
        Err(error) => {
            error!(
                "Error while parsing message:\n{}-----------------------\n{}",
                error, message_string,
            );
            return None;
        }
        Ok(RPCMessage::Response(_response)) => {
            warn!("got unknown response");
            return None;
        }
        Ok(RPCMessage::Request(response)) => match response.method.as_str() {
            "initialize" => match serde_json::from_str::<InitializeRequest>(&message_string) {
                Ok(initialize_request) => {
                    let response = handle_initialize_request(&server, initialize_request);
                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!("Could not parse initialize request: {:?}", error);
                    return None;
                }
            },
            "shutdown" => match serde_json::from_str::<RequestMessage>(&message_string) {
                Ok(shutdown_request) => {
                    info!("Recieved shutdown request, preparing to shut down");
                    let response = ShutdownResponse::new(&shutdown_request.id);
                    server.state.status = ServerStatus::ShuttingDown;
                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!("Could not parse shutdown request: {:?}", error);
                    return None;
                }
            },
            "textDocument/formatting" => {
                match serde_json::from_str::<FormattingRequest>(&message_string) {
                    Ok(formatting_request) => {
                        let response = match handle_format_request(
                            &formatting_request,
                            &mut server.state,
                            &server.settings,
                        ) {
                            Ok(text_edits) => {
                                let response = FormattingResponse::new(
                                    formatting_request.get_id(),
                                    text_edits,
                                );
                                serde_json::to_string(&response).unwrap()
                            }
                            Err(error) => {
                                let response = ResponseMessage {
                                    base: Message::new(),
                                    id: RequestIdOrNull::RequestId(
                                        formatting_request.get_id().clone(),
                                    ),
                                    error: Some(error),
                                };
                                serde_json::to_string(&response).unwrap()
                            }
                        };
                        // TODO: Fix format response
                        return Some(serde_json::to_string(&response).unwrap());
                    }
                    Err(error) => {
                        error!(
                            "Could not parse textDocument/formatting request: {:?}",
                            error
                        );
                        return None;
                    }
                }
            }
            "textDocument/diagnostic" => {
                match serde_json::from_str::<DiagnosticRequest>(&message_string) {
                    Ok(diagnostic_request) => {
                        info!("Recieved diagnostic request");
                        if let Some(diagnostics) = collect_diagnostics(
                            &server,
                            &diagnostic_request.params.text_document.uri,
                        ) {
                            let resonse = DiagnosticResponse::new(
                                diagnostic_request.get_id(),
                                diagnostics.collect(),
                            );
                            return Some(serde_json::to_string(&resonse).unwrap());
                        }
                        return None;
                    }
                    Err(error) => {
                        error!(
                            "Could not parse textDocument/diagnostic request: {:?}",
                            error
                        );
                        return None;
                    }
                }
            }
            "textDocument/codeAction" => {
                match serde_json::from_str::<CodeActionRequest>(&message_string) {
                    Ok(codeaction_request) => {
                        let mut code_action_response =
                            CodeActionResponse::new(codeaction_request.get_id());
                        let code_actions =
                            generate_code_actions(server, &codeaction_request.params)
                                .unwrap_or(vec![]);
                        code_action_response.add_code_actions(code_actions);
                        let response = serde_json::to_string(&code_action_response).unwrap();
                        return Some(response);
                    }
                    Err(error) => {
                        error!(
                            "Could not parse textDocument/codeAction request: {:?}",
                            error
                        );
                        return None;
                    }
                }
            }
            "textDocument/hover" => match serde_json::from_str::<HoverRequest>(&message_string) {
                Ok(hover_request) => {
                    debug!(
                        "recieved hover request for {} {}",
                        hover_request.get_document_uri(),
                        hover_request.get_position()
                    );
                    let response = handle_hover_request(&hover_request, &server.state);

                    return Some(serde_json::to_string(&response).unwrap());
                }
                Err(error) => {
                    error!("Could not parse textDocument/hover request: {:?}", error);
                    return None;
                }
            },
            "textDocument/completion" => {
                match serde_json::from_str::<CompletionRequest>(&message_string) {
                    Ok(completion_request) => {
                        debug!(
                            "Received completion request for {} {}",
                            completion_request.get_document_uri(),
                            completion_request.get_position()
                        );
                        let response =
                            handel_completion_request(completion_request, &mut server.state);
                        return Some(serde_json::to_string(&response).unwrap());
                    }
                    Err(error) => {
                        error!(
                            "Could not parse textDocument/completion request: {:?}",
                            error
                        );
                        return None;
                    }
                }
            }
            "workspace/executeCommand" => {
                match serde_json::from_str::<ExecuteCommandRequest>(&message_string) {
                    Ok(execute_command_request) => {
                        let execute_command_response = ExecuteCommandResponse {
                            base: ResponseMessage::success(execute_command_request.get_id()),
                            result: handle_command(server, execute_command_request.params).unwrap(),
                        };
                        // TODO: unifi this next two lines & remove unwrap
                        let response = serde_json::to_string(&execute_command_response).unwrap();
                        return Some(response);
                    }
                    Err(error) => {
                        error!(
                            "Could not parse workspace/executeCommand request: {:?}",
                            error
                        );
                        return None;
                    }
                }
            }

            unknown_method => {
                warn!(
                    "Received notification with unknown method \"{}\"",
                    unknown_method
                );
                let response = ResponseMessage::error(
                    &response.id,
                    ErrorCode::MethodNotFound,
                    r#"Method "{}" currently not supported"#,
                );
                return Some(serde_json::to_string(&response).unwrap());
            }
        },
        Ok(RPCMessage::Notification(notification)) => match notification.method.as_str() {
            "initialized" => {
                info!("initialization completed");
                server.state.status = ServerStatus::Running;
                return None;
            }

            "exit" => {
                info!("Recieved exit notification, shutting down!");
                exit(0);
            }
            "textDocument/didOpen" => {
                match serde_json::from_str::<DidOpenTextDocumentNotification>(&message_string) {
                    Ok(did_open_notification) => {
                        info!(
                            "opened text document: \"{}\"",
                            did_open_notification.params.text_document.uri
                        );
                        let text_document: TextDocumentItem =
                            did_open_notification.get_text_document();
                        server.state.add_document(text_document);
                        return None;
                    }
                    Err(error) => {
                        error!("Could not parse textDocument/didOpen request: {:?}", error);
                        return None;
                    }
                }
            }
            "textDocument/didChange" => {
                match serde_json::from_str::<DidChangeTextDocumentNotification>(&message_string) {
                    Ok(did_change_notification) => {
                        server.state.change_document(
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
            "$/setTrace" => match serde_json::from_str::<SetTraceNotification>(&message_string) {
                Ok(set_trace_notification) => {
                    info!(
                        "Setting trace value to: {:?}",
                        set_trace_notification.params.value
                    );
                    server.state.trace_value = set_trace_notification.params.value;
                    return None;
                }
                Err(error) => {
                    error!("Could not parse setTrace Notification: {:?}", error);
                    return None;
                }
            },
            unknown_method => {
                // TODO: Return LSP Method unknown method
                warn!(
                    "Received notification with unknown method \"{}\"",
                    unknown_method
                );
                return None;
            }
        },
    };
}
