use crate::{
    analysis::{get_undeclared_prefixes, get_unused_prefixes, AnalysisState},
    lsp::{Diagnostic, DiagnosticSeverity},
};

pub fn collect_diagnostics(
    state: &AnalysisState,
    uri: &String,
) -> impl Iterator<Item = Diagnostic> {
    unused_prefix(state, uri).chain(undeclared_prefix(state, uri))
}

fn unused_prefix(state: &AnalysisState, uri: &String) -> impl Iterator<Item = Diagnostic> {
    get_unused_prefixes(state, uri).map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Warning,
        source: "monza (unused_prefix)".to_string(),
        message: format!("'{}' is declared here, but was never used\n", prefix),
    })
}

fn undeclared_prefix(state: &AnalysisState, uri: &String) -> impl Iterator<Item = Diagnostic> {
    get_undeclared_prefixes(state, uri).map(|(prefix, range)| Diagnostic {
        range: range.clone(),
        severity: DiagnosticSeverity::Warning,
        source: "monza (undeclared_prefix)".to_string(),
        message: format!("'{}' is used here, but was never delared\n", prefix),
    })
}

