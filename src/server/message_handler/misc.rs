use log::info;

use crate::server::{
    lsp::{errors::ResponseError, SetTraceNotification},
    Server,
};

pub(super) fn handle_set_trace_notifcation(
    server: &mut Server,
    set_trace_notification: SetTraceNotification,
) -> Result<(), ResponseError> {
    info!("Trace set to: {:?}", set_trace_notification.params.value);
    server.state.trace_value = set_trace_notification.params.value;
    Ok(())
}
