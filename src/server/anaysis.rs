use streaming_iterator::StreamingIterator;

use std::collections::HashSet;

use log::{error, info};

use tree_sitter::{Node, Query, QueryCursor};

use super::{
    lsp::textdocument::{Position, Range},
    state::ServerState,
    Server,
};

fn collect_all_unique_captures(node: Node, query_str: &str, text: &String) -> Vec<String> {
    match Query::new(&tree_sitter_sparql::LANGUAGE.into(), query_str) {
        Ok(query) => {
            let mut capture_set: HashSet<String> = HashSet::new();
            let mut query_cursor = QueryCursor::new();
            let mut captures = query_cursor.captures(&query, node, text.as_bytes());
            while let Some((mat, capture_index)) = captures.next() {
                let node: Node = mat.captures[*capture_index].node;
                if node.end_byte() != node.start_byte() {
                    capture_set.insert(node.utf8_text(text.as_bytes()).unwrap().to_string());
                }
            }
            capture_set.into_iter().collect()
        }

        Err(_) => {
            error!("Building a tree-sitter query failed: {}", query_str);
            vec![]
        }
    }
}

pub fn get_all_variables(analyis_state: &ServerState, uri: &String) -> Vec<String> {
    match analyis_state.get_state(uri) {
        Some((document, Some(tree))) => {
            collect_all_unique_captures(tree.root_node(), "(VAR) @variable", &document.text)
        }
        Some((_document, None)) => {
            info!("Could not compute variables for {}: No tree availible", uri);
            vec![]
        }
        None => {
            error!("Could not compute variables for {}: No such document", uri);
            vec![]
        }
    }
}

pub fn get_kind_at_position(
    analyis_state: &ServerState,
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

pub fn namespace_is_declared(
    server_state: &ServerState,
    document_uri: &String,
    namespace: &str,
) -> bool {
    let declared_namespaces = get_declared_namespaces(server_state, document_uri);
    let set: HashSet<String> = HashSet::from_iter(
        declared_namespaces
            .into_iter()
            .map(|(namespace, _range)| namespace),
    );
    set.contains(&(namespace.to_string() + ":"))
}

pub fn get_all_uncompressed_uris(server: &Server, document_uri: &String) -> Vec<(String, Range)> {
    match server.state.get_state(document_uri) {
        Some((document, Some(tree))) => {
            let declared_uris = collect_all_unique_captures(
                tree.root_node(),
                "(PrefixDecl (IRIREF) @variable)",
                &document.text,
            );
            let prefix_set: HashSet<String> = HashSet::from_iter(declared_uris.into_iter());
            let all_uris = get_all_uris(&server.state, document_uri);
            all_uris
                .into_iter()
                .filter(|(uri, _range)| !prefix_set.contains(uri))
                .collect()
        }
        _ => {
            vec![]
        }
    }
}

fn get_all_uris(analyis_state: &ServerState, document_uri: &String) -> Vec<(String, Range)> {
    match analyis_state.get_state(document_uri) {
        Some((document, Some(tree))) => {
            match Query::new(&tree_sitter_sparql::LANGUAGE.into(), "(IRIREF) @iri") {
                Ok(query) => {
                    let mut query_cursor = QueryCursor::new();
                    let mut captures =
                        query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
                    let mut namespaces: Vec<(String, Range)> = Vec::new();
                    while let Some((mat, capture_index)) = captures.next() {
                        let node = mat.captures[*capture_index].node;
                        namespaces.push((
                            node.utf8_text(document.text.as_bytes())
                                .unwrap()
                                .to_string(),
                            Range::from_node(node),
                        ));
                    }
                    return namespaces;
                }
                Err(_) => {
                    error!(
                        "Could not compute declared namspaces for {}: Query could not be build",
                        document_uri
                    );
                    vec![]
                }
            }
        }
        Some((_document, None)) => {
            info!(
                "Could not compute declared namespaces for {}: No tree availible",
                document_uri
            );
            vec![]
        }
        None => {
            error!(
                "Could not compute declared namspaces for {}: No such document",
                document_uri
            );
            vec![]
        }
    }
}

pub fn get_declared_namespaces(
    analyis_state: &ServerState,
    document_uri: &String,
) -> Vec<(String, Range)> {
    match analyis_state.get_state(document_uri) {
        Some((document, Some(tree))) => {
            match Query::new(
                &tree_sitter_sparql::LANGUAGE.into(),
                "(PrefixDecl (PNAME_NS) @namespace)",
            ) {
                Ok(query) => {
                    let mut query_cursor = QueryCursor::new();
                    let mut captures =
                        query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
                    let mut namespaces: Vec<(String, Range)> = Vec::new();
                    while let Some((mat, capture_index)) = captures.next() {
                        let node = mat.captures[*capture_index].node;
                        namespaces.push((
                            node.utf8_text(document.text.as_bytes())
                                .unwrap()
                                .to_string(),
                            Range::from_node(node),
                        ));
                    }
                    return namespaces;
                }
                Err(_) => {
                    error!(
                        "Could not compute declared namspaces for {}: Query could not be build",
                        document_uri
                    );
                    vec![]
                }
            }
        }
        Some((_document, None)) => {
            info!(
                "Could not compute declared namespaces for {}: No tree availible",
                document_uri
            );
            vec![]
        }
        None => {
            error!(
                "Could not compute declared namspaces for {}: No such document",
                document_uri
            );
            vec![]
        }
    }
}

pub fn get_used_namspaces(analyis_state: &ServerState, uri: &String) -> Vec<(String, Range)> {
    match analyis_state.get_state(uri) {
        Some((document, Some(tree))) => {
            match Query::new(
                &tree_sitter_sparql::LANGUAGE.into(),
                "(PrefixedName (PNAME_NS) @namespace)",
            ) {
                Ok(query) => {
                    let mut query_cursor = QueryCursor::new();
                    let mut captures =
                        query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
                    let mut namespaces: Vec<(String, Range)> = Vec::new();
                    while let Some((mat, capture_index)) = captures.next() {
                        let node = mat.captures[*capture_index].node;
                        namespaces.push((
                            node.utf8_text(document.text.as_bytes())
                                .unwrap()
                                .to_string(),
                            Range::from_node(node),
                        ));
                    }
                    return namespaces;
                }

                Err(_) => {
                    error!(
                        "Could not compute declared namspaces for {}: Query could not be build",
                        uri
                    );
                    vec![]
                }
            }
        }
        Some((_document, None)) => {
            info!(
                "Could not compute declared namespaces for {}: No tree availible",
                uri
            );
            vec![]
        }
        None => {
            error!(
                "Could not compute declared namspaces for {}: No such document",
                uri
            );
            vec![]
        }
    }
}

pub(crate) fn get_unused_prefixes(
    analysis_state: &ServerState,
    uri: &String,
) -> impl Iterator<Item = (String, Range)> {
    let declared_namespaces = get_declared_namespaces(analysis_state, uri);
    let declared_namespaces_set: HashSet<String> = declared_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let used_namespaces = get_used_namspaces(analysis_state, uri);
    let used_namespaces_set: HashSet<String> = used_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();

    declared_namespaces
        .iter()
        .filter_map(move |(declared_namespace, range)| {
            (&declared_namespaces_set - &used_namespaces_set)
                .contains(declared_namespace)
                .then_some((declared_namespace.clone(), range.clone()))
        })
        .collect::<Vec<(String, Range)>>()
        .into_iter()
}

pub(crate) fn get_undeclared_prefixes(
    analysis_state: &ServerState,
    uri: &String,
) -> impl Iterator<Item = (String, Range)> {
    let declared_namespaces = get_declared_namespaces(analysis_state, uri);
    let declared_namespaces_set: HashSet<String> = declared_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let used_namespaces = get_used_namspaces(analysis_state, uri);
    let used_namespaces_set: HashSet<String> = used_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();

    used_namespaces
        .iter()
        .filter_map(|(declared_namespace, range)| {
            (&used_namespaces_set - &declared_namespaces_set)
                .contains(declared_namespace)
                .then_some((declared_namespace.clone(), range.clone()))
        })
        .collect::<Vec<(String, Range)>>()
        .into_iter()
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::server::{
        anaysis::{
            get_declared_namespaces, get_undeclared_prefixes, get_unused_prefixes,
            get_used_namspaces,
        },
        lsp::textdocument::TextDocumentItem,
        state::ServerState,
    };

    #[test]
    fn declared_namespaces() {
        let mut state = ServerState::new();
        state.add_document(TextDocumentItem::new(
            "uri",
            indoc!(
                "PREFIX wdt: <iri>
                 PREFIX wd: <iri>
                 PREFIX wdt: <iri>

                 SELECT * {}"
            ),
        ));
        let declared_namesapces = get_declared_namespaces(&state, &"uri".to_string());
        assert_eq!(
            declared_namesapces
                .iter()
                .map(|(namespace, _range)| namespace)
                .collect::<Vec<&String>>(),
            vec!["wdt:", "wd:", "wdt:"]
        );
    }

    #[test]
    fn used_namespaces() {
        let mut state = ServerState::new();
        state.add_document(TextDocumentItem::new(
            "uri",
            indoc!("SELECT * {?a wdt:P32 ?b. ?a wd:p32 ?b. ?a wdt:P31 ?b}"),
        ));
        let declared_namesapces = get_used_namspaces(&state, &"uri".to_string());
        assert_eq!(
            declared_namesapces
                .iter()
                .map(|(namespace, _range)| namespace)
                .collect::<Vec<&String>>(),
            vec!["wdt:", "wd:", "wdt:"]
        );
    }

    #[test]
    fn undeclared_namespaces() {
        let mut state = ServerState::new();
        state.add_document(TextDocumentItem::new(
            "uri",
            indoc!("SELECT * {x:y y:p x:x}"),
        ));
        let declared_namesapces: Vec<String> = get_undeclared_prefixes(&state, &"uri".to_string())
            .map(|(namespace, _range)| namespace)
            .collect();
        assert_eq!(declared_namesapces, vec!["x:", "y:", "x:"]);
    }
    #[test]
    fn unused_namespaces() {
        let mut state = ServerState::new();
        state.add_document(TextDocumentItem::new(
            "uri",
            indoc!(
                "PREFIX wdt: <>
                 PREFIX wdt: <>

                 SELECT * {}"
            ),
        ));
        let declared_namesapces: Vec<String> = get_unused_prefixes(&state, &"uri".to_string())
            .map(|(namespace, _range)| namespace)
            .collect();
        assert_eq!(declared_namesapces, vec!["wdt:", "wdt:"]);
    }
}
