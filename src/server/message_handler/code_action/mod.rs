mod quickfix;
use std::collections::HashSet;

use quickfix::get_quickfix;

use crate::server::{
    anaysis::{get_all_uncompacted_uris, get_declared_uri_prefixes},
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode},
        errors::ResponseError,
        textdocument::{Range, TextDocumentItem, TextEdit},
        CodeAction, CodeActionKind, CodeActionParams, CodeActionRequest, CodeActionResponse,
    },
    Server,
};

pub fn handle_codeaction_request(
    server: &mut Server,
    request: CodeActionRequest,
) -> Result<CodeActionResponse, ResponseError> {
    let mut code_action_response = CodeActionResponse::new(request.get_id());
    code_action_response.add_code_actions(generate_code_actions(server, &request.params)?);
    code_action_response.add_code_actions(
        request
            .params
            .context
            .diagnostics
            .into_iter()
            .filter_map(|diagnostic| {
                match get_quickfix(server, &request.params.text_document.uri, diagnostic) {
                    Ok(code_action) => code_action,
                    Err(err) => {
                        log::error!(
                            "Encountered Error while computing quickfix:\n{}\nDropping error!",
                            err.message
                        );
                        None
                    }
                }
            })
            .collect(),
    );
    Ok(code_action_response)
}

fn generate_code_actions(
    server: &Server,
    params: &CodeActionParams,
) -> Result<Vec<CodeAction>, ResponseError> {
    let document_uri = &params.text_document.uri;
    let (document, parse_tree) = server.state.get_state(document_uri)?;
    if let Some(node) = parse_tree
        .root_node()
        .descendant_for_point_range(params.range.start.to_point(), params.range.end.to_point())
    {
        if let Some(parent) = node.parent() {
            if node.kind() == "IRIREF"
                && parent.kind() != "PrefixDecl"
                && parent.kind() != "BaseDecl"
            {
                let mut code_actions = vec![];
                // if let Some(code_action) = shorten_uri(server, Range::from_node(node), &document) {
                //     code_actions.push(code_action);
                // }
                if let Some(code_action) = shorten_all_uris(server, &document) {
                    code_actions.push(code_action);
                }
                return Ok(code_actions);
            }
        }
    }
    Ok(vec![])
}

// TODO: Handle errors properly.
fn shorten_all_uris(server: &Server, document: &TextDocumentItem) -> Option<CodeAction> {
    let mut code_action = CodeAction::new("Shorten all URI's", Some(CodeActionKind::Refactor));
    let uncompacted_uris = get_all_uncompacted_uris(server, &document.uri).ok()?;
    let mut declared_uri_prefix_set: HashSet<String> =
        get_declared_uri_prefixes(&server.state, &document.uri)
            .ok()?
            .into_iter()
            .map(|(uri, _range)| uri[1..uri.len() - 1].to_string())
            .collect();

    uncompacted_uris.iter().for_each(|(uri, range)| {
        if let Some((prefix, uri_prefix, curie)) = server.shorten_uri(&uri[1..uri.len() - 1]) {
            code_action.add_edit(&document.uri, TextEdit::new(range.clone(), &curie));
            if !declared_uri_prefix_set.contains(&uri_prefix) {
                code_action.add_edit(
                    &document.uri,
                    TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, uri_prefix),
                    ),
                );
                declared_uri_prefix_set.insert(uri_prefix);
            }
        }
    });
    if !uncompacted_uris.is_empty() {
        return Some(code_action);
    }

    None
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use tree_sitter::Parser;
    use tree_sitter_sparql::LANGUAGE;

    use crate::server::{
        lsp::textdocument::{Range, TextDocumentItem, TextEdit},
        message_handler::code_action::shorten_all_uris,
        state::ServerState,
        Server,
    };

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let mut parser = Parser::new();
        if let Err(err) = parser.set_language(&LANGUAGE.into()) {
            log::error!("Could not initialize parser:\n{}", err)
        }
        let document = TextDocumentItem::new("uri", text);
        let tree = parser.parse(&document.text, None);
        state.add_document(document, tree);
        return state;
    }

    #[test]
    fn shorten_all_uris_undeclared() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "SELECT * {
               ?a <http://schema.org/name> ?b .
               ?c <http://schema.org/name> ?d
             }"
        ));
        server.state = state;
        let document = server.state.get_document("uri").unwrap();
        let code_action = shorten_all_uris(&server, document).unwrap();
        assert_eq!(
            code_action.edit.changes.get("uri").unwrap(),
            &vec![
                TextEdit::new(Range::new(1, 5, 1, 29), "schema:name"),
                TextEdit::new(
                    Range::new(0, 0, 0, 0),
                    "PREFIX schema: <http://schema.org/>\n"
                ),
                TextEdit::new(Range::new(2, 5, 2, 29), "schema:name"),
            ]
        );
    }

    #[test]
    fn shorten_all_uris_declared() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "PREFIX schema: <http://schema.org/>
             SELECT * {
               ?a <http://schema.org/name> ?b .
               ?c <http://schema.org/name> ?d
             }"
        ));
        server.state = state;
        let document = server.state.get_document("uri").unwrap();
        let code_action = shorten_all_uris(&server, document).unwrap();
        assert_eq!(
            code_action.edit.changes.get("uri").unwrap(),
            &vec![
                TextEdit::new(Range::new(2, 5, 2, 29), "schema:name"),
                TextEdit::new(Range::new(3, 5, 3, 29), "schema:name"),
            ]
        );
    }
}
