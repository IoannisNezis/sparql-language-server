use log::info;

use crate::server::{
    lsp::{InitializeRequest, InitializeResonse, ProgressNotification},
    Server,
};

pub(super) fn handle_initialize_request(
    server: &Server,
    initialize_request: InitializeRequest,
) -> InitializeResonse {
    info!(
        "Connected to: {} {}",
        initialize_request.params.client_info.name,
        initialize_request
            .params
            .client_info
            .version
            .unwrap_or("no version specified".to_string())
    );
    if let Some(work_done_token) = initialize_request.params.progress_params.work_done_token {
        let init_progress_begin_notification = ProgressNotification::begin_notification(
            work_done_token.clone(),
            &format!("setup qlue-ls v{}", server.get_version()),
            Some(false),
            Some("init"),
            Some(0),
        );
        server.send_message(serde_json::to_string(&init_progress_begin_notification).unwrap());

        // TODO: implement server side requests
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

        server.send_message(serde_json::to_string(&init_progress_end_notification).unwrap());
    }
    InitializeResonse::new(initialize_request.base.id, server)
}
