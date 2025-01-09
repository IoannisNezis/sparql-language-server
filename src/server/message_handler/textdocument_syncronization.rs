use log::{error, info, warn};

use crate::server::{
    lsp::{
        errors::{ErrorCode, ResponseError},
        DidChangeTextDocumentNotification, DidOpenTextDocumentNotification,
        DidSaveTextDocumentNotification,
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
    let document = did_open_notification.get_text_document();
    let tree = server.tools.parser.parse(document.text.as_bytes(), None);
    server.state.add_document(document, tree);
    Ok(())
}

pub(super) fn handle_did_change_notification(
    server: &mut Server,
    did_change_notification: DidChangeTextDocumentNotification,
) -> Result<(), ResponseError> {
    let uri = &did_change_notification.params.text_document.base.uri;
    if let Some(document) = server
        .state
        .change_document(uri, did_change_notification.params.content_changes)
    {
        let bytes = document.text.as_bytes();
        // let old_tree = server.state.get_tree(&uri).ok();
        let new_tree = server.tools.parser.parse(bytes, None);
        if new_tree.is_none() {
            warn!("Could not build new parse-tree for \"{}\"", uri);
        }
        if let Err(err) = server.state.update_tree(uri, new_tree) {
            error!("{}", err.message);
            return Err(ResponseError::new(
                ErrorCode::InternalError,
                &format!("Error while building parse-tree:\n{}", err.message),
            ));
        }

        Ok(())
    } else {
        let message = format!("Did-Change request failed, document not found: \"{}\"", uri);
        error!("{}", message);
        Err(ResponseError::new(ErrorCode::InvalidRequest, &message))
    }
}

pub(super) fn handle_did_save_notification(
    _server: &mut Server,
    did_save_notification: DidSaveTextDocumentNotification,
) -> Result<(), ResponseError> {
    log::warn!(
        "saved text document (has no effect yet): \"{}\"",
        did_save_notification.params.text_document.uri
    );
    Ok(())
}
