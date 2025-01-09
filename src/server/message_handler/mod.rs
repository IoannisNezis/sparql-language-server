mod code_action;
mod commands;
mod completion;
mod diagnostic;
mod formatting;
mod hovering;
mod lifecycle;
mod misc;
mod textdocument_syncronization;

use code_action::handle_codeaction_request;
use commands::handle_execute_command_request;
use completion::handle_completion_request;
use hovering::handle_hover_request;
use lifecycle::{
    handle_exit_notifcation, handle_initialize_request, handle_initialized_notifcation,
    handle_shutdown_request,
};
use log::warn;
use misc::handle_set_trace_notifcation;
use serde::{de::DeserializeOwned, Serialize};
use std::any::type_name;
use textdocument_syncronization::{
    handle_did_change_notification, handle_did_open_notification, handle_did_save_notification,
};

pub use diagnostic::*;
pub use formatting::format_raw;

use crate::server::lsp::errors::ErrorCode;

use self::formatting::handle_format_request;

use super::{
    lsp::{
        errors::ResponseError,
        rpc::{deserialize_message, RPCMessage},
    },
    Server,
};

fn handle_request<T, R>(
    server: &mut Server,
    request: RPCMessage,
    mut handler: impl FnMut(&mut Server, T) -> Result<R, ResponseError>,
) -> Result<Option<String>, ResponseError>
where
    T: Serialize + DeserializeOwned,
    R: Serialize,
{
    let response = handler(server, request.parse::<T>()?)?;
    match type_name::<R>() {
        "()" => Ok(None),
        _ => Ok(Some(serde_json::to_string(&response).map_err(|error| {
            ResponseError::new(
                ErrorCode::ParseError,
                &format!(
                    "Could not serialize response \"{}\"\n\n{}",
                    type_name::<R>(),
                    error
                ),
            )
        })?)),
    }
}

pub fn dispatch(
    server: &mut Server,
    message_string: &String,
) -> Result<Option<String>, ResponseError> {
    let message = deserialize_message(message_string)?;
    let method = message.get_method().unwrap_or("");
    macro_rules! link {
        ($handler:ident) => {
            handle_request(server, message, $handler)?
        };
    }
    Ok(match method {
        // Requests
        "initialize" => link!(handle_initialize_request),
        "shutdown" => link!(handle_shutdown_request),
        "textDocument/formatting" => link!(handle_format_request),
        "textDocument/diagnostic" => link!(handle_diagnostic_request),
        "textDocument/codeAction" => link!(handle_codeaction_request),
        "textDocument/hover" => link!(handle_hover_request),
        "textDocument/completion" => link!(handle_completion_request),
        "workspace/executeCommand" => link!(handle_execute_command_request),
        // Notifications
        "initialized" => link!(handle_initialized_notifcation),
        "exit" => link!(handle_exit_notifcation),
        "textDocument/didOpen" => link!(handle_did_open_notification),
        "textDocument/didChange" => link!(handle_did_change_notification),
        "textDocument/didSave" => link!(handle_did_save_notification),
        "$/setTrace" => link!(handle_set_trace_notifcation),
        unknown_method => {
            warn!(
                "Received message with unknown method \"{}\"",
                unknown_method
            );
            return Err(ResponseError::new(
                ErrorCode::MethodNotFound,
                &format!("Method \"{}\" currently not supported", unknown_method),
            ));
        }
    })
}
