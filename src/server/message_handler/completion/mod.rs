use log::error;

use crate::server::{
    anaysis::get_all_variables,
    lsp::{CompletionRequest, CompletionResponse, CompletionTriggerKind},
    ServerState,
};

pub fn handel_completion_request(
    request: CompletionRequest,
    state: &mut ServerState,
) -> CompletionResponse {
    match request.get_completion_context().trigger_kind {
        // Completion was triggered by typing an identifier (24x7 code complete),
        // manual invocation (e.g Ctrl+Space) or via API.
        CompletionTriggerKind::Invoked => CompletionResponse::new(request.get_id()),
        // Completion was triggered by a trigger character specified by
        // the `triggerCharacters` properties of the `CompletionRegistrationOptions`.
        // i.e. "?"
        CompletionTriggerKind::TriggerCharacter => CompletionResponse::from_variables(
            request.get_id(),
            get_all_variables(&state, request.get_document_uri()),
        ),
        CompletionTriggerKind::TriggerForIncompleteCompletions => {
            error!("Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet");
            CompletionResponse::from_variables(request.get_id(), vec![])
        }
    }
}
