mod code_action;
mod commands;
mod completion;
mod diagnostic;
mod formatting;
mod hovering;
mod lifecycle;

use code_action::handle_codeaction_request;
use commands::handle_execute_command_request;
use completion::handle_completion_request;
use hovering::handle_hover_request;
use lifecycle::{handle_initialize_request, handle_shutdown_request};
use log::{error, info, warn};
use serde::{de::DeserializeOwned, Serialize};
use std::{any::type_name, process::exit};

pub use diagnostic::*;
pub use formatting::format_raw;

use crate::server::lsp::errors::ErrorCode;

use self::formatting::handle_format_request;

use super::{
    lsp::{
        errors::ResponseError,
        rpc::{self, RPCMessage, RequestMessage},
        textdocument::TextDocumentItem,
        DidChangeTextDocumentNotification, DidOpenTextDocumentNotification, SetTraceNotification,
    },
    state::ServerStatus,
    Server,
};

fn parse_rpc_message<T, O>(rpc_message: O) -> Result<T, ResponseError>
where
    T: Serialize + DeserializeOwned,
    O: Serialize,
{
    match serde_json::to_string(&rpc_message) {
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

fn handle_request<T, R>(
    server: &mut Server,
    request: RequestMessage,
    mut handler: impl FnMut(&mut Server, T) -> Result<R, ResponseError>,
) -> Result<String, ResponseError>
where
    T: Serialize + DeserializeOwned,
    R: Serialize,
{
    serde_json::to_string(&handler(
        server,
        parse_rpc_message::<T, RequestMessage>(request)?,
    )?)
    .map_err(|error| {
        ResponseError::new(
            ErrorCode::ParseError,
            &format!(
                "Could not serialize response \"{}\"\n\n{}",
                type_name::<R>(),
                error
            ),
        )
    })
}

pub fn dispatch(
    server: &mut Server,
    message_string: &String,
) -> Result<Option<String>, ResponseError> {
    match rpc::deserialize_message(message_string)? {
        RPCMessage::Response(_response) => {
            warn!("got unknown response");
            Ok(None)
        }
        RPCMessage::Request(request) => Ok(Some(match request.method.as_str() {
            "initialize" => handle_request(server, request, handle_initialize_request)?,
            "shutdown" => handle_request(server, request, handle_shutdown_request)?,
            "textDocument/formatting" => handle_request(server, request, handle_format_request)?,
            "textDocument/diagnostic" => {
                handle_request(server, request, handle_diagnostic_request)?
            }
            "textDocument/codeAction" => {
                handle_request(server, request, handle_codeaction_request)?
            }
            "textDocument/hover" => handle_request(server, request, handle_hover_request)?,
            "textDocument/completion" => {
                handle_request(server, request, handle_completion_request)?
            }
            "workspace/executeCommand" => {
                handle_request(server, request, handle_execute_command_request)?
            }
            unknown_method => {
                warn!(
                    "Received notification with unknown method \"{}\"",
                    unknown_method
                );
                return Err(ResponseError::new(
                    ErrorCode::MethodNotFound,
                    &format!("Method \"{}\" currently not supported", unknown_method),
                ));
            }
        })),
        RPCMessage::Notification(notification) => {
            match notification.method.as_str() {
                "initialized" => {
                    info!("initialization completed");
                    server.state.status = ServerStatus::Running;
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
                        }
                        Err(error) => {
                            error!("Could not parse textDocument/didOpen request: {:?}", error);
                        }
                    }
                }
                "textDocument/didChange" => {
                    match serde_json::from_str::<DidChangeTextDocumentNotification>(&message_string)
                    {
                        Ok(did_change_notification) => {
                            server.state.change_document(
                                did_change_notification.params.text_document.base.uri,
                                did_change_notification.params.content_changes,
                            );
                        }
                        Err(error) => {
                            error!(
                                "Could not parse textDocument/didChange notification: {:?}",
                                error
                            );
                        }
                    }
                }
                "$/setTrace" => match serde_json::from_str::<SetTraceNotification>(&message_string)
                {
                    Ok(set_trace_notification) => {
                        info!(
                            "Setting trace value to: {:?}",
                            set_trace_notification.params.value
                        );
                        server.state.trace_value = set_trace_notification.params.value;
                    }
                    Err(error) => {
                        error!("Could not parse setTrace Notification: {:?}", error);
                    }
                },
                unknown_method => {
                    // TODO: Return LSP Method unknown method
                    warn!(
                        "Received notification with unknown method \"{}\"",
                        unknown_method
                    );
                }
            };
            Ok(None)
        }
    }
}
