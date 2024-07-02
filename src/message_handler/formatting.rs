use log::{error, info};
use tree_sitter::{Tree, TreeCursor};

use crate::{
    lsp::{
        textdocument::{TextDocumentItem, TextEdit},
        FormattingOptions, FormattingRequest, FormattingResponse,
    },
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

fn format_query(
    document: &TextDocumentItem,
    tree: &Tree,
    options: &FormattingOptions,
) -> Vec<TextEdit> {
    let range = document.get_full_range();
    let text = format_helper(document, &mut tree.walk(), 0, options);
    vec![TextEdit::new(range, text)]
}

fn format_helper(
    document: &TextDocumentItem,
    cursor: &mut TreeCursor,
    indentation_level: u8,
    options: &FormattingOptions,
) -> String {
    let mut result = String::new();
    if cursor.goto_first_child() {
        loop {
            match cursor.node().kind() {
                "unit" => {
                    result.push_str(&format_helper(document, cursor, indentation_level, options));
                }
                "prologue" => {
                    result.push_str(&format_helper(
                        document,
                        cursor,
                        indentation_level + 1,
                        options,
                    ));
                    result.push_str("\n");
                }
                "prefix_declaration" => {
                    // TODO: Fix trailing whitespace
                    result.push_str(&format_helper(document, cursor, indentation_level, options));
                    result.push_str("\n");
                }
                "PREFIX" => {
                    result.push_str("PREFIX ");
                }
                other => {
                    info!("hit unknown node kind: {}", other);
                    result.push_str(document.extract_node(cursor.node()).unwrap());
                    result.push(' ');
                }
            };

            if !cursor.goto_next_sibling() {
                cursor.goto_parent();
                break;
            }
        }
    }
    return result;
}
