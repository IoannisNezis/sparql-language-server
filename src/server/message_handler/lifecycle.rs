use std::process::exit;

use log::info;

use crate::server::{
    lsp::{
        errors::{ErrorCode, ResponseError},
        rpc::{NotificationMessage, RequestMessage},
        InitializeRequest, InitializeResonse, ProgressNotification, ShutdownResponse,
    },
    state::ServerStatus,
    Server,
};

pub fn handle_shutdown_request(
    server: &mut Server,
    request: RequestMessage,
) -> Result<ShutdownResponse, ResponseError> {
    info!("Recieved shutdown request, preparing to shut down");
    match server.state.status {
        ServerStatus::Initializing => Err(ResponseError::new(
            ErrorCode::InvalidRequest,
            "The Server is not yet initialized",
        )),
        ServerStatus::ShuttingDown => Err(ResponseError::new(
            ErrorCode::InvalidRequest,
            "The Server is already shutting down",
        )),
        ServerStatus::Running => {
            server.state.status = ServerStatus::ShuttingDown;

            Ok(ShutdownResponse::new(&request.id))
        }
    }
}

pub(super) fn handle_initialize_request(
    server: &mut Server,
    initialize_request: InitializeRequest,
) -> Result<InitializeResonse, ResponseError> {
    match server.state.status {
        ServerStatus::Initializing => {
            if let Some(ref client_info) = initialize_request.params.client_info {
                info!(
                    "Connected to: {} {}",
                    client_info.name,
                    client_info
                        .version
                        .clone()
                        .unwrap_or("no version specified".to_string())
                );
            }
            if let Some(ref work_done_token) =
                initialize_request.params.progress_params.work_done_token
            {
                let init_progress_begin_notification = ProgressNotification::begin_notification(
                    work_done_token.clone(),
                    &format!("setup qlue-ls v{}", server.get_version()),
                    Some(false),
                    Some("init"),
                    Some(0),
                );
                server.send_message(
                    serde_json::to_string(&init_progress_begin_notification).unwrap(),
                );

                let progress_report_1 = ProgressNotification::report_notification(
                    work_done_token.clone(),
                    Some(false),
                    Some("testing availibility of endpoint"),
                    Some(30),
                );
                server.send_message(serde_json::to_string(&progress_report_1).unwrap());

                let progress_report_2 = ProgressNotification::report_notification(
                    work_done_token.clone(),
                    Some(false),
                    Some("request prefixes from endpoint"),
                    Some(60),
                );
                server.send_message(serde_json::to_string(&progress_report_2).unwrap());

                let init_progress_end_notification = ProgressNotification::end_notification(
                    work_done_token.clone(),
                    Some("qlue-ls initialized"),
                );

                server
                    .send_message(serde_json::to_string(&init_progress_end_notification).unwrap());
            }
            Ok(InitializeResonse::new(initialize_request.get_id(), server))
        }
        _ => Err(ResponseError::new(
            ErrorCode::InvalidRequest,
            "The Server is already initialized",
        )),
    }
}

pub(super) fn handle_initialized_notifcation(
    server: &mut Server,
    _initialized_notification: NotificationMessage,
) -> Result<(), ResponseError> {
    info!("initialization completed");
    server.state.status = ServerStatus::Running;
    Ok(())
}

pub(super) fn handle_exit_notifcation(
    _server: &mut Server,
    _initialized_notification: NotificationMessage,
) -> Result<(), ResponseError> {
    info!("Recieved exit notification, shutting down!");
    exit(0);
}
