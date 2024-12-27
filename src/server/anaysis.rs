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
    let declared_namespaces: HashSet<String> = get_declared_prefixes(server_state, document_uri)?
        .into_iter()
        .map(|(namespace, _range)| namespace)
        .collect();
    Ok(declared_namespaces.contains(namespace))
}

pub fn get_all_uncompacted_uris(
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

/// Extracts the declared namespaces from a SPARQL document.
///
/// This function parses the specified document to identify namespace declarations
/// (`PrefixDecl`) and returns a list of tuples, each containing the namespace prefix
/// and its corresponding range within the document.
///
/// # Arguments
///
/// * `analysis_state` - A reference to the `ServerState` object, which provides access
///   to the document and its syntax tree.
/// * `document_uri` - A string slice representing the URI of the document to analyze.
///
/// # Returns
///
/// * `Ok(Vec<(String, Range)>)` - A vector of tuples where each tuple consists of:
///   - A `String` representing the namespace prefix.
///   - A `Range` specifying the location of the prefix in the document.
/// * `Err(ResponseError)` - An error if the document or its syntax tree cannot be
///   retrieved, or if the query for namespace declarations fails.
///
/// # Errors
///
/// This function can return a `ResponseError` if:
/// * The document specified by `document_uri` cannot be found or loaded.
/// * The syntax tree for the document cannot be accessed.
/// * The query for extracting `PrefixDecl` fails to build or execute.
///
/// # Example
///
/// Given the following SPARQL query in the document located at `file://example.sparql`:
///
/// ```sparql
/// PREFIX ex: <http://example.org/>
/// PREFIX foaf: <http://xmlns.com/foaf/0.1/>
///
/// SELECT ?name WHERE {
///   ?person a foaf:Person .
///   ?person foaf:name ?name .
/// }
/// ```
///
/// Calling the function:
///
/// ```rust
/// let namespaces = get_declared_namespaces(&analysis_state, "file://example.sparql")?;
/// for (prefix, range) in namespaces {
///     println!("Found prefix: {} at range: {:?}", prefix, range);
/// }
/// ```
///
/// Would return:
///
/// ```text
/// Ok(vec![
///     ("ex".to_string(), Range { start: Position { line: 0, character: 7 }, end: Position { line: 0, character: 9 } }),
///     ("foaf".to_string(), Range { start: Position { line: 1, character: 7 }, end: Position { line: 1, character: 11 } }),
/// ])
/// ```
///
/// # Notes
///
/// The function assumes that the document is written in SPARQL syntax and uses
/// Tree-sitter for syntax tree traversal to locate namespace declarations.
pub(crate) fn get_declared_prefixes(
    server_state: &ServerState,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = server_state.get_state(document_uri)?;
    let query = build_query("(PrefixDecl (PNAME_NS (PN_PREFIX) @prefix))")?;
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

pub(crate) fn get_declared_uri_prefixes(
    server_state: &ServerState,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = server_state.get_state(document_uri)?;
    let query = build_query("(PrefixDecl (PNAME_NS (PN_PREFIX)) (IRIREF) @uri)")?;
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

pub fn get_used_prefixes(
    analyis_state: &ServerState,
    uri: &String,
) -> Result<Vec<(String, Range)>, ResponseError> {
    let (document, tree) = analyis_state.get_state(uri)?;
    let query = build_query("(PrefixedName (PNAME_NS (PN_PREFIX) @prefix))")?;
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
    let declared_namespaces = get_declared_prefixes(analysis_state, uri)?;
    let declared_namespaces_set: HashSet<String> = declared_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let used_namespaces_set: HashSet<String> = get_used_prefixes(analysis_state, uri)?
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let unused_prefixes = &declared_namespaces_set - &used_namespaces_set;

    Ok(declared_namespaces
        .iter()
        .filter_map(move |(declared_namespace, range)| {
            unused_prefixes
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
    let declared_namespaces_set: HashSet<String> = get_declared_prefixes(analysis_state, uri)?
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let used_namespaces = get_used_prefixes(analysis_state, uri)?;
    let used_namespaces_set: HashSet<String> = used_namespaces
        .iter()
        .map(|(namespace, _range)| namespace)
        .cloned()
        .collect();
    let undelclared_prefixes_set = &used_namespaces_set - &declared_namespaces_set;

    Ok(used_namespaces
        .iter()
        .filter_map(|(declared_namespace, range)| {
            undelclared_prefixes_set
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
            get_declared_prefixes, get_undeclared_prefixes, get_unused_prefixes, get_used_prefixes,
        },
        lsp::textdocument::TextDocumentItem,
        state::ServerState,
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
    fn declared_namespaces() {
        let state = setup_state(indoc!(
            "PREFIX wdt: <iri>
                 PREFIX wd: <iri>
                 PREFIX wdt: <iri>

                 SELECT * {}"
        ));
        let declared_namesapces = get_declared_prefixes(&state, "uri").unwrap();
        assert_eq!(
            declared_namesapces
                .iter()
                .map(|(namespace, _range)| namespace)
                .collect::<Vec<&String>>(),
            vec!["wdt", "wd", "wdt"]
        );
    }

    #[test]
    fn used_namespaces() {
        let state = setup_state(indoc!(
            "SELECT * {?a wdt:P32 ?b. ?a wd:p32 ?b. ?a wdt:P31 ?b}"
        ));
        let declared_namesapces = get_used_prefixes(&state, &"uri".to_string()).unwrap();
        assert_eq!(
            declared_namesapces
                .iter()
                .map(|(namespace, _range)| namespace)
                .collect::<Vec<&String>>(),
            vec!["wdt", "wd", "wdt"]
        );
    }

    #[test]
    fn undeclared_namespaces() {
        let state = setup_state(indoc!("PREFIX x: <> SELECT * {x:y y:p x:x}"));
        let declared_namesapces: Vec<String> = get_undeclared_prefixes(&state, &"uri".to_string())
            .unwrap()
            .map(|(namespace, _range)| namespace)
            .collect();
        assert_eq!(declared_namesapces, vec!["y"]);
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
        assert_eq!(declared_namesapces, vec!["wdt", "wdt"]);
    }
}
