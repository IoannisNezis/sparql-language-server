use crate::{
    lsp::{CompletionRequest, CompletionResponse},
    server::ServerState,
};

pub fn handel_completion_request(
    request: CompletionRequest,
    state: &mut ServerState,
) -> CompletionResponse {
    let document_uri = request.get_document_uri();
    CompletionResponse::new(request.get_id())
}
