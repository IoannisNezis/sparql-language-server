use streaming_iterator::StreamingIterator;

use std::collections::HashSet;

use tree_sitter::{Node, Query, QueryCursor};

use super::{
    lsp::{
        errors::{ErrorCode, ResponseError},
        textdocument::{Position, Range},
    },
    state::ServerState,
    Server,
};

fn build_query(query_str: &str) -> Result<Query, ResponseError> {
    Query::new(&tree_sitter_sparql::LANGUAGE.into(), query_str).map_err(|error| {
        ResponseError::new(
            ErrorCode::InternalError,
            &format!(
                "Building tree-sitter query failed:\n{}\n{}",
                query_str, error
            ),
        )
    })
}

fn collect_all_unique_captures(
    node: Node,
    query_str: &str,
    text: &String,
) -> Result<Vec<String>, ResponseError> {
    let query = build_query(query_str)?;
    let mut capture_set: HashSet<String> = HashSet::new();
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, node, text.as_bytes());
    while let Some((mat, capture_index)) = captures.next() {
        let node: Node = mat.captures[*capture_index].node;
        if node.end_byte() != node.start_byte() {
            capture_set.insert(node.utf8_text(text.as_bytes()).unwrap().to_string());
        }
    }
    Ok(capture_set.into_iter().collect())
}

pub fn get_all_variables(
    analyis_state: &ServerState,
    uri: &String,
) -> Result<Vec<String>, ResponseError> {
    let (document, tree) = analyis_state.get_state(uri)?;
    collect_all_unique_captures(tree.root_node(), "(VAR) @variable", &document.text)
}

pub fn get_kind_at_position(
    analyis_state: &ServerState,
    uri: &String,
    position: &Position,
) -> Result<&'static str, ResponseError> {
    let tree = analyis_state.get_tree(uri)?;
    let point = position.to_point();

    tree.root_node()
        .descendant_for_point_range(point, point)
        .ok_or(ResponseError::new(
            ErrorCode::InternalError,
            &format!("Could not get kind at position {} of {}", position, uri),
        ))
        .map(|node| node.kind())
}

pub fn namespace_is_declared(
    server_state: &ServerState,
    document_uri: &String,
    namespace: &str,
) -> Result<bool, ResponseError> {
    let declared_namespaces = get_declared_namespaces(server_state, document_uri)?;
    let set: HashSet<String> = HashSet::from_iter(
        declared_namespaces
            .into_iter()
            .map(|(namespace, _range)| namespace),
    );
    Ok(set.contains(&(namespace.to_string() + ":")))
}

pub fn get_all_uncompressed_uris(
    server: &Server,
    document_uri: &String,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = server.state.get_state(document_uri)?;
    let declared_uris = collect_all_unique_captures(
        tree.root_node(),
        "(PrefixDecl (IRIREF) @variable)",
        &document.text,
    )?;
    let prefix_set: HashSet<String> = HashSet::from_iter(declared_uris.into_iter());
    let all_uris = get_all_uris(&server.state, document_uri)?;
    Ok(all_uris
        .into_iter()
        .filter(|(uri, _range)| !prefix_set.contains(uri))
        .collect())
}

fn get_all_uris(
    analyis_state: &ServerState,
    document_uri: &String,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = analyis_state.get_state(document_uri)?;
    let query_str = "(IRIREF) @iri";
    let query = build_query(query_str)?;
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
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
    Ok(namespaces)
}

pub fn get_declared_namespaces(
    analyis_state: &ServerState,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = analyis_state.get_state(document_uri)?;
    let query = build_query("(PrefixDecl (PNAME_NS) @namespace)")?;
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
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
    Ok(namespaces)
}

pub fn get_used_namspaces(
    analyis_state: &ServerState,
    uri: &String,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = analyis_state.get_state(uri)?;
    let query = build_query("(PrefixedName (PNAME_NS) @namespace)")?;
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
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
    Ok(namespaces)
}

pub(crate) fn get_unused_prefixes(
    analysis_state: &ServerState,
    uri: &String,
) -> Result<impl Iterator<Item = (String, Range)>, ResponseError> {
    let declared_namespaces = get_declared_namespaces(analysis_state, uri)?;
    let declared_namespaces_set: HashSet<String> = declared_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let used_namespaces = get_used_namspaces(analysis_state, uri)?;
    let used_namespaces_set: HashSet<String> = used_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();

    Ok(declared_namespaces
        .iter()
        .filter_map(move |(declared_namespace, range)| {
            (&declared_namespaces_set - &used_namespaces_set)
                .contains(declared_namespace)
                .then_some((declared_namespace.clone(), range.clone()))
        })
        .collect::<Vec<(String, Range)>>()
        .into_iter())
}

pub(crate) fn get_undeclared_prefixes(
    analysis_state: &ServerState,
    uri: &String,
) -> Result<impl Iterator<Item = (String, Range)>, ResponseError> {
    let declared_namespaces = get_declared_namespaces(analysis_state, uri)?;
    let declared_namespaces_set: HashSet<String> = declared_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let used_namespaces = get_used_namspaces(analysis_state, uri)?;
    let used_namespaces_set: HashSet<String> = used_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();

    Ok(used_namespaces
        .iter()
        .filter_map(|(declared_namespace, range)| {
            (&used_namespaces_set - &declared_namespaces_set)
                .contains(declared_namespace)
                .then_some((declared_namespace.clone(), range.clone()))
        })
        .collect::<Vec<(String, Range)>>()
        .into_iter())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use tree_sitter::Parser;
    use tree_sitter_sparql::LANGUAGE;

    use crate::server::{
        anaysis::{
            get_declared_namespaces, get_undeclared_prefixes, get_unused_prefixes,
            get_used_namspaces,
        },
        lsp::textdocument::TextDocumentItem,
        state::ServerState,
    };

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let mut parser = Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        let document = TextDocumentItem::new("uri", text);
        let tree = parser.parse(&document.text, None);
        state.add_document(document, tree);
        return state;
    }

    #[test]
    fn declared_namespaces() {
        let state = setup_state(indoc!(
            "PREFIX wdt: <iri>
                 PREFIX wd: <iri>
                 PREFIX wdt: <iri>

                 SELECT * {}"
        ));
        let declared_namesapces = get_declared_namespaces(&state, "uri").unwrap();
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
        let state = setup_state(indoc!(
            "SELECT * {?a wdt:P32 ?b. ?a wd:p32 ?b. ?a wdt:P31 ?b}"
        ));
        let declared_namesapces = get_used_namspaces(&state, &"uri".to_string()).unwrap();
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
        let state = setup_state(indoc!("SELECT * {x:y y:p x:x}"));
        let declared_namesapces: Vec<String> = get_undeclared_prefixes(&state, &"uri".to_string())
            .unwrap()
            .map(|(namespace, _range)| namespace)
            .collect();
        assert_eq!(declared_namesapces, vec!["x:", "y:", "x:"]);
    }
    #[test]
    fn unused_namespaces() {
        let state = setup_state(indoc!(
            "PREFIX wdt: <>
             PREFIX wdt: <>
             SELECT * {}"
        ));
        let declared_namesapces: Vec<String> = get_unused_prefixes(&state, &"uri".to_string())
            .unwrap()
            .map(|(namespace, _range)| namespace)
            .collect();
        assert_eq!(declared_namesapces, vec!["wdt:", "wdt:"]);
    }
}
