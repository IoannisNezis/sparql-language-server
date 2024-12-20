use std::collections::HashSet;

use crate::server::{
    anaysis::{get_all_uncompressed_uris, get_declared_namespaces, namespace_is_declared},
    lsp::{
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
    let code_actions = generate_code_actions(server, &request.params)?;
    code_action_response.add_code_actions(code_actions);
    Ok(code_action_response)
}

pub fn generate_code_actions(
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
                if let Some(code_action) = compress_uri(server, Range::from_node(node), &document) {
                    code_actions.push(code_action);
                }
                if let Some(code_action) = compress_all_uris(server, &document) {
                    code_actions.push(code_action);
                }
                return Ok(code_actions);
            }
        }
    }
    Ok(vec![])
}

// TODO: Handle errors properly.
fn compress_uri(server: &Server, range: Range, document: &TextDocumentItem) -> Option<CodeAction> {
    let mut code_action = CodeAction::new("Compress URI", Some(CodeActionKind::Refactor));
    let mut uri = &document.text[range.to_byte_index_range(&document.text)?];
    uri = &uri[1..uri.len() - 1];
    if let Some((prefix, uri_prefix, curie)) = server.compress_uri(uri) {
        code_action.add_edit(&document.uri, TextEdit::new(range, &curie));
        if !namespace_is_declared(&server.state, &document.uri, &prefix).ok()? {
            code_action.add_edit(
                &document.uri,
                TextEdit::new(
                    Range::new(0, 0, 0, 0),
                    &format!("PREFIX {}: <{}>\n", prefix, uri_prefix),
                ),
            );
        }
        return Some(code_action);
    }
    None
}

// TODO: Handle errors properly.
fn compress_all_uris(server: &Server, document: &TextDocumentItem) -> Option<CodeAction> {
    let mut code_action = CodeAction::new("Compress all URI", Some(CodeActionKind::Refactor));
    let uncompressed_uris = get_all_uncompressed_uris(server, &document.uri).ok()?;
    let declared_uri_prefix = get_declared_namespaces(&server.state, &document.uri).ok()?;
    let mut set: HashSet<String> = HashSet::from_iter(
        declared_uri_prefix
            .into_iter()
            .map(|(namespace, _range)| namespace[..namespace.len() - 1].to_string()),
    );

    uncompressed_uris.iter().for_each(|(uri, range)| {
        if let Some((prefix, uri_prefix, curie)) = server.compress_uri(&uri[1..uri.len() - 1]) {
            code_action.add_edit(&document.uri, TextEdit::new(range.clone(), &curie));

            if !set.contains(&prefix) {
                code_action.add_edit(
                    &document.uri,
                    TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, uri_prefix),
                    ),
                );
                set.insert(prefix);
            }
        }
    });
    if !uncompressed_uris.is_empty() {
        return Some(code_action);
    }

    None
}
