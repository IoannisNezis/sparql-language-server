use crate::server::{
    anaysis::{get_all_uncompacted_uris, get_undeclared_prefixes, get_unused_prefixes},
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        errors::ResponseError,
        textdocument::TextDocumentItem,
        DiagnosticRequest, DiagnosticResponse,
    },
    Server,
};

pub fn handle_diagnostic_request(
    server: &mut Server,
    request: DiagnosticRequest,
) -> Result<DiagnosticResponse, ResponseError> {
    Ok(DiagnosticResponse::new(
        request.get_id(),
        collect_diagnostics(server, &request.params.text_document.uri)?.collect(),
    ))
}

pub fn collect_diagnostics<'a>(
    server: &'a Server,
    document_uri: &str,
) -> Result<impl Iterator<Item = Diagnostic> + use<'a>, ResponseError> {
    let document = server.state.get_document(document_uri)?;
    let unused_prefixes = unused_prefix(server, document)?;
    Ok(unused_prefixes
        .chain(undeclared_prefix(server, document)?)
        .chain(uncompacted_uris(server, document)?))
}

fn unused_prefix<'a>(
    server: &Server,
    document: &TextDocumentItem,
) -> Result<impl Iterator<Item = Diagnostic> + use<'a>, ResponseError> {
    let ununsed_prefixes = get_unused_prefixes(&server.state, &document.uri)?;
    Ok(ununsed_prefixes.map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Warning,
        source: Some("qlue-ls (unused-prefix)".to_string()),
        code: Some(DiagnosticCode::String("unused-prefix".to_string())),
        message: format!("'{}' is declared here, but was never used\n", prefix),
        data: None,
    }))
}

fn undeclared_prefix(
    server: &Server,
    document: &TextDocumentItem,
) -> Result<impl Iterator<Item = Diagnostic>, ResponseError> {
    let undeclared_prefixes = get_undeclared_prefixes(&server.state, &document.uri)?;
    Ok(undeclared_prefixes.map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Error,
        source: Some("qlue-ls (undeclared_prefix)".to_string()),
        code: Some(DiagnosticCode::String("undeclared-prefix".to_string())),
        message: format!("'{}' is used here, but was never delared\n", prefix),
        data: Some(LSPAny::String(prefix)),
    }))
}

fn uncompacted_uris<'a>(
    server: &'a Server,
    document: &TextDocumentItem,
) -> Result<impl Iterator<Item = Diagnostic> + use<'a>, ResponseError> {
    let uncompacted_uris = get_all_uncompacted_uris(server, &document.uri)?;
    let diagnostics = uncompacted_uris.into_iter().filter_map(|(uri, range)| {
        match server.shorten_uri(&uri[1..uri.len() - 1]) {
            Some((prefix, namespace, curie)) => Some(Diagnostic {
                source: Some("qlue-ls".to_string()),
                code: Some(DiagnosticCode::String("uncompacted-uri".to_string())),
                range,
                severity: DiagnosticSeverity::Information,
                message: format!("You might want to shorten this Uri\n{} -> {}", uri, curie),
                data: Some(LSPAny::LSPArray(vec![
                    LSPAny::String(prefix),
                    LSPAny::String(namespace),
                    LSPAny::String(curie),
                ])),
            }),
            None => None,
        }
    });
    Ok(diagnostics)
}
