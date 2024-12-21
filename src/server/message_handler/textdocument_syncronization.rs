use log::info;

use crate::server::{
    lsp::{
        errors::ResponseError, DidChangeTextDocumentNotification, DidOpenTextDocumentNotification,
    },
    Server,
};

pub(super) fn handle_did_open_notification(
    server: &mut Server,
    did_open_notification: DidOpenTextDocumentNotification,
) -> Result<(), ResponseError> {
    info!(
        "opened text document: \"{}\"",
        did_open_notification.params.text_document.uri
    );
    server
        .state
        .add_document(did_open_notification.get_text_document());
    Ok(())
}

pub(super) fn handle_did_change_notification(
    server: &mut Server,
    did_change_notification: DidChangeTextDocumentNotification,
) -> Result<(), ResponseError> {
    server.state.change_document(
        did_change_notification.params.text_document.base.uri,
        did_change_notification.params.content_changes,
    );
    Ok(())
}
