use log::error;

use crate::server::{
    anaysis::get_all_variables,
    lsp::{
        errors::{ErrorCode, ResponseError},
        CompletionRequest, CompletionResponse, CompletionTriggerKind,
    },
    Server,
};

pub fn handle_completion_request(
    server: &mut Server,
    request: CompletionRequest,
) -> Result<CompletionResponse, ResponseError> {
    match request.get_completion_context().trigger_kind {
        // Completion was triggered by typing an identifier (24x7 code complete),
        // manual invocation (e.g Ctrl+Space) or via API.
        CompletionTriggerKind::Invoked => Ok(CompletionResponse::new(request.get_id())),
        // Completion was triggered by a trigger character specified by
        // the `triggerCharacters` properties of the `CompletionRegistrationOptions`.
        // i.e. "?"
        CompletionTriggerKind::TriggerCharacter => Ok(CompletionResponse::from_variables(
            request.get_id(),
            get_all_variables(&server.state, request.get_document_uri())?,
        )),
        CompletionTriggerKind::TriggerForIncompleteCompletions => {
            error!("Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet");
            Err(ResponseError::new(ErrorCode::InvalidRequest, "Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet"))
        }
    }
}
