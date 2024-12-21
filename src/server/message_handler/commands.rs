use core::fmt;
use std::any::type_name;

use log::error;
use serde::{de::DeserializeOwned, Serialize};

use crate::server::{
    commands::PublishDiagnosticsCommandAruments,
    lsp::{
        errors::{ErrorCode, ResponseError},
        rpc::NotificationMessageBase,
        ExecuteCommandRequest, ExecuteCommandResponse, PublishDiagnosticsNotification,
        PublishDiagnosticsPrarams,
    },
    message_handler::collect_diagnostics,
    state::ServerStatus,
    Server,
};

pub fn handle_execute_command_request(
    server: &mut Server,
    request: ExecuteCommandRequest,
) -> Result<ExecuteCommandResponse, ResponseError> {
    match request.params.command.as_str() {
        "publishDiagnostics" => {
            let arguments = parse_command_arguments(&request.params.arguments)?;
            publish_diagnostic(server, &arguments);
            Ok(ExecuteCommandResponse::new(request.get_id()))
        }
        unknown_command => {
            error!("Received unknown Command request: {}", unknown_command);
            Err(ResponseError::new(
                ErrorCode::InvalidRequest,
                &format!("Received unknown Command request: {}", unknown_command),
            ))
        }
    }
}

fn parse_command_arguments<T, O>(rpc_message: O) -> Result<T, ResponseError>
where
    T: Serialize + DeserializeOwned,
    O: Serialize + fmt::Debug,
{
    match serde_json::to_string(&rpc_message) {
        Ok(serialized_message) => serde_json::from_str(&serialized_message).map_err(|error| {
            error!(
                "Error while deserializing message:\n{}-----------------------\n{:?}",
                error, rpc_message,
            );
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

fn publish_diagnostic(server: &Server, args: &PublishDiagnosticsCommandAruments) {
    let uri = &args.0 .0;
    if server.state.status == ServerStatus::Running {
        match collect_diagnostics(server, uri) {
            Ok(diagnostics) => {
                let diagnostic_notification = PublishDiagnosticsNotification {
                    base: NotificationMessageBase::new("textDocument/publishDiagnistics"),
                    params: PublishDiagnosticsPrarams {
                        uri: uri.to_string(),
                        diagnostics: diagnostics.collect(),
                    },
                };
                let message = serde_json::to_string(&diagnostic_notification).unwrap();
                server.send_message(message);
            }
            Err(error) => {
                error!(
                    "Error occured while publishing diagnostics:\n\n{}",
                    error.message
                )
            }
        }
    }
}
