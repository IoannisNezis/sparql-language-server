mod core;
mod utils;
use core::*;
use log::{error, info};

use crate::{
    lsp::{FormattingRequest, FormattingResponse},
    state::ServerState,
};

pub fn handle_format_request(
    request: FormattingRequest,
    state: &mut ServerState,
) -> FormattingResponse {
    let uri = request.get_document_uri();
    info!("Received formatting request for: {}", uri);
    match state.analysis_state.get_document(uri) {
        Some((document, Some(tree))) => {
            let options = request.get_options();
            let text_edits = format_query(document, tree, options);
            FormattingResponse::new(request.get_id(), text_edits)
        }
        _ => {
            error!("Requested formatting for unknown document: {}", uri);
            todo!()
        }
    }
}

#[cfg(test)]
mod tests;
