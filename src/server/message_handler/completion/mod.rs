use log::{error, warn};

use crate::server::{
    anaysis::get_all_variables,
    lsp::{
        errors::{ErrorCode, ResponseError},
        CompletionItem, CompletionItemKind, CompletionRequest, CompletionResponse,
        CompletionTriggerKind, InsertTextFormat,
    },
    Server,
};

pub fn handle_completion_request(
    server: &mut Server,
    request: CompletionRequest,
) -> Result<CompletionResponse, ResponseError> {
    match request.get_completion_context().trigger_kind {
        // Completion was triggered by typing an trigger character
        CompletionTriggerKind::TriggerCharacter  => Ok(
            CompletionResponse::new(request.get_id(), collect_completions_triggered(server, &request)?))
        ,
        // Completion was triggered by typing an identifier (24x7 code complete),
        // manual invocation (e.g Ctrl+Space) or via API.
        CompletionTriggerKind::Invoked  => Ok(
            CompletionResponse::new(request.get_id(), collect_completions(server, &request)?),
        ),
        CompletionTriggerKind::TriggerForIncompleteCompletions => {
            error!("Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet");
            Err(ResponseError::new(ErrorCode::InvalidRequest, "Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet"))
        }
    }
}

fn variable_completions(
    server: &Server,
    request: &CompletionRequest,
    triggered: bool,
) -> Result<impl Iterator<Item = CompletionItem>, ResponseError> {
    Ok(get_all_variables(
        &server.state,
        &request.get_text_position().text_document.uri,
    )?
    .into_iter()
    .map(move |variable| {
        CompletionItem::new(
            &variable,
            "variable",
            match triggered {
                true => &variable[1..],
                false => &variable,
            },
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        )
    }))
}

fn graph_pattern_not_triples_completions(
    _server: &Server,
    _request: &CompletionRequest,
) -> Result<impl Iterator<Item = CompletionItem>, ResponseError> {
    Ok([
        CompletionItem::new(
            "FILTER",
            "Filter the results",
            "FILTER ( $0 )",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "BIND",
            "Bind a new variable",
            "BIND ($1 AS ?$0)",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "VALUES",
            "Inline data definition",
            "VALUES ?$1 { $0 }",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "SERVICE",
            "Collect data from a fedarated SPARQL endpoint",
            "SERVICE <$1> {\n  $0\n}",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "MINUS",
            "Subtract data",
            "MINUS { $0 }",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "OPTIONAL",
            "Optional graphpattern",
            "OPTIONAL { $0 }",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "UNION",
            "Union of two results",
            "{\n  $1\n}\nUNION\n{\n  $0\n}",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
    ]
    .into_iter())
}

fn collect_completions_triggered(
    server: &Server,
    request: &CompletionRequest,
) -> Result<Vec<CompletionItem>, ResponseError> {
    let trigger_character =
        request
            .params
            .context
            .trigger_character
            .to_owned()
            .ok_or(ResponseError::new(
                ErrorCode::InvalidParams,
                "triggered completion request has no trigger character",
            ))?;
    Ok(match trigger_character.as_str() {
        "?" => variable_completions(server, &request, true)?.collect(),
        other => {
            warn!(
                "Completion request triggered by unknown trigger character: \"{}\"",
                other
            );
            vec![]
        }
    })
}

fn collect_completions(
    server: &Server,
    request: &CompletionRequest,
) -> Result<Vec<CompletionItem>, ResponseError> {
    let tree = server
        .state
        .get_tree(&request.get_text_position().text_document.uri)
        .unwrap();
    let position = &request.get_text_position().position;
    let node = tree
        .root_node()
        .descendant_for_point_range(position.to_point(), position.to_point())
        .unwrap();
    Ok(match node.kind() {
        "unit" => vec![
            CompletionItem::new(
                "SELECT",
                "Select query",
                "SELECT ${1:*} WHERE {\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "PREFIX",
                "Declare a namespace",
                "PREFIX ${1:namespace}: <${0:iri}>",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "ORDER BY",
                "Sort the results",
                "ORDER BY ${1|ASC,DESC|} ( $0 )",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "BASE",
                "Set the Base URI",
                "BASE <${0}>",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
        ],
        "GroupGraphPattern" => variable_completions(server, request, false)?
            .chain(graph_pattern_not_triples_completions(server, request)?)
            .collect(),
        _ => vec![],
    })
}
