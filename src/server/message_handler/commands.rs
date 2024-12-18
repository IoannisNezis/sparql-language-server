use std::any::type_name;

use log::warn;
use serde::{de::DeserializeOwned, Serialize};

use crate::server::{
    commands::PublishDiagnosticsCommandAruments,
    lsp::{
        base_types::LSPAny, rpc::NotificationMessage, ExecuteCommandParams,
        PublishDiagnosticsNotification, PublishDiagnosticsPrarams,
    },
    message_handler::collect_diagnostics,
    state::ServerStatus,
    Server,
};

pub fn handle_command(server: &Server, params: ExecuteCommandParams) -> Result<LSPAny, String> {
    match params.command.as_str() {
        "publishDiagnostics" => {
            let arguments = parse_command_arguments(&params.arguments)
                .map_err(to_parse_error::<PublishDiagnosticsCommandAruments>)?;
            publish_diagnostic(server, &arguments);
        }
        unknown_command => {
            warn!("Received unknown Command: {}", unknown_command);
        }
    }
    // TODO: Return LSP ERROR
    return Ok(LSPAny::Null);
}

fn to_parse_error<T>(error: serde_json::Error) -> String {
    format!(
        "Could not parse Command arguments \"{}\"\n\n{}",
        type_name::<T>(),
        error
    )
}

fn parse_command_arguments<T>(arguments: &Option<Vec<LSPAny>>) -> Result<T, serde_json::Error>
where
    T: Serialize + DeserializeOwned,
{
    let serialized_arguments = serde_json::to_string(&arguments)?;
    serde_json::from_str(&serialized_arguments)
}

fn publish_diagnostic(server: &Server, args: &PublishDiagnosticsCommandAruments) {
    let uri = &args.0 .0;
    if server.state.status == ServerStatus::Running {
        if let Some(diagnostics) = collect_diagnostics(server, uri) {
            let diagnostic_notification = PublishDiagnosticsNotification {
                base: NotificationMessage::new("textDocument/publishDiagnistics"),
                params: PublishDiagnosticsPrarams {
                    uri: uri.to_string(),
                    diagnostics: diagnostics.collect(),
                },
            };
            let message = serde_json::to_string(&diagnostic_notification).unwrap();
            server.send_message(message);
        }
    }
}
