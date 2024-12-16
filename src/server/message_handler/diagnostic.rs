use crate::server::{
    anaysis::{get_undeclared_prefixes, get_unused_prefixes},
    lsp::{Diagnostic, DiagnosticSeverity},
    state::ServerState,
};

pub fn collect_diagnostics(state: &ServerState, uri: &String) -> impl Iterator<Item = Diagnostic> {
    unused_prefix(state, uri).chain(undeclared_prefix(state, uri))
}

fn unused_prefix(state: &ServerState, uri: &String) -> impl Iterator<Item = Diagnostic> {
    get_unused_prefixes(state, uri).map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Warning,
        source: "qlue-ls (unused_prefix)".to_string(),
        message: format!("'{}' is declared here, but was never used\n", prefix),
    })
}

fn undeclared_prefix(state: &ServerState, uri: &String) -> impl Iterator<Item = Diagnostic> {
    get_undeclared_prefixes(state, uri).map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Error,
        source: "qlue-ls (undeclared_prefix)".to_string(),
        message: format!("'{}' is used here, but was never delared\n", prefix),
    })
}
