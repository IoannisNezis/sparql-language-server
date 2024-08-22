mod state;

use log::info;
pub use state::*;

use tree_sitter_c2rust::{Query, QueryCursor};

use crate::lsp::textdocument::{Position, Range};

pub fn get_kind_at_position(
    analyis_state: &AnalysisState,
    uri: &String,
    position: &Position,
) -> Option<&'static str> {
    match analyis_state.get_tree(uri) {
        Some(tree) => {
            let point = position.to_point();
            Some(
                tree.root_node()
                    .descendant_for_point_range(point, point)?
                    .kind(),
            )
        }
        None => None,
    }
}

pub(crate) fn get_unused_prefixes(
    analysis_state: &AnalysisState,
) -> Vec<(String, Vec<(String, Range)>)> {
    analysis_state
        .documents()
        .map(|uri| {
            (
                uri.clone(),
                match analysis_state.get_state(uri) {
                    Some((textdocument, Some(tree))) => {
                        let mut declared_namespaces: Vec<(String, Range)> = vec![];
                        let mut used_namespaces: Vec<(String, Range)> = vec![];
                        match Query::new(
                            &tree_sitter_sparql::language(),
                            "(PrefixDecl (PNAME_NS) @decl)(PrefixedName (PNAME_NS) @used)",
                        ) {
                            Ok(query) => {
                                // Step 1: Get all prefix strs and their lengths.
                                // NOTE: Here a `HashSet` is used to avoid douplication of prefixes.
                                let mut query_cursor = QueryCursor::new();
                                let captures = query_cursor.captures(
                                    &query,
                                    tree.root_node(),
                                    textdocument.text.as_bytes(),
                                );
                                captures.for_each(|(query_match, capture_index)| {
                                    let node = query_match.captures[capture_index].node;
                                    match query.capture_names()[query_match.pattern_index] {
                                        "decl" => declared_namespaces.push((
                                            node.utf8_text(textdocument.text.as_bytes())
                                                .unwrap()
                                                .to_string(),
                                            Range::new(
                                                node.start_position().row as u32,
                                                node.start_position().column as u32,
                                                node.end_position().row as u32,
                                                node.end_position().column as u32,
                                            ),
                                        )),
                                        "used" => used_namespaces.push((
                                            node.utf8_text(textdocument.text.as_bytes())
                                                .unwrap()
                                                .to_string(),
                                            Range::new(
                                                node.start_position().row as u32,
                                                node.start_position().column as u32,
                                                node.end_position().row as u32,
                                                node.end_position().column as u32,
                                            ),
                                        )),
                                        _ => {}
                                    }
                                });

                                let unused: Vec<(String, Range)> = declared_namespaces
                                    .iter()
                                    .filter(|decl| {
                                        used_namespaces
                                            .iter()
                                            .filter(|used| decl.0 == used.0)
                                            .count()
                                            == 0
                                    })
                                    .map(|(prefix, range)| (prefix.clone(), range.clone()))
                                    .collect();
                                unused
                            }

                            Err(_) => vec![],
                        }
                    }
                    _ => {
                        vec![]
                    }
                },
            )
        })
        .collect()
}
