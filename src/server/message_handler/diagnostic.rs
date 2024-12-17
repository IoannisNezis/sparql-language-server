use log::error;

use crate::server::{
    anaysis::{get_all_uncompressed_uris, get_undeclared_prefixes, get_unused_prefixes},
    lsp::{textdocument::TextDocumentItem, Diagnostic, DiagnosticSeverity},
    Server,
};

pub fn collect_diagnostics<'a>(
    server: &'a Server,
    document_uri: &str,
) -> Option<impl Iterator<Item = Diagnostic> + use<'a>> {
    if let Some(document) = server.state.get_document(document_uri) {
        Some(
            unused_prefix(server, document)
                .chain(undeclared_prefix(server, document))
                .chain(uncompressed_uris(server, document)),
        )
    } else {
        error!(
            "collecting diagnostics for {} failed, document not found!",
            document_uri
        );
        None
    }
}

fn unused_prefix<'a>(
    server: &Server,
    document: &TextDocumentItem,
) -> impl Iterator<Item = Diagnostic> + use<'a> {
    get_unused_prefixes(&server.state, &document.uri).map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Warning,
        source: "qlue-ls (unused_prefix)".to_string(),
        message: format!("'{}' is declared here, but was never used\n", prefix),
    })
}

fn undeclared_prefix(
    server: &Server,
    document: &TextDocumentItem,
) -> impl Iterator<Item = Diagnostic> {
    get_undeclared_prefixes(&server.state, &document.uri).map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Error,
        source: "qlue-ls (undeclared_prefix)".to_string(),
        message: format!("'{}' is used here, but was never delared\n", prefix),
    })
}

fn uncompressed_uris<'a>(
    server: &'a Server,
    document: &TextDocumentItem,
) -> impl Iterator<Item = Diagnostic> + use<'a> {
    let uncompressed_uris = get_all_uncompressed_uris(server, &document.uri);
    uncompressed_uris.into_iter().filter_map(|(uri, range)| {
        match server.compress_uri(&uri[1..uri.len() - 1]) {
            Some((_prefix, _namespace, curie)) => Some(Diagnostic {
                source: "dings".to_string(),
                range,
                severity: DiagnosticSeverity::Information,
                message: format!("You might want to compress this Uri\n{} -> {}", uri, curie),
            }),
            None => None,
        }
    })
}
