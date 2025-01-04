use log::error;

use crate::server::{
    common::{serde_parse, PublishDiagnosticsCommandAruments},
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
            let arguments = serde_parse(&request.params.arguments)?;
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
