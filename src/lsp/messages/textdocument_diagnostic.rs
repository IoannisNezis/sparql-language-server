use serde::{Deserialize, Serialize};

use crate::{
    lsp::textdocument::TextDocumentIdentifier,
    rpc::{RequestMessage, ResponseMessage},
};

use super::Diagnostic;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticRequest {
    #[serde(flatten)]
    pub base: RequestMessage,
    pub params: DocumentDiagnosticParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDiagnosticParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticResponse {
    #[serde(flatten)]
    pub base: ResponseMessage,
    pub result: DocumentDiagnosticReport,
}

impl DiagnosticResponse {
    pub fn new(id: u32, items: Vec<Diagnostic>) -> Self {
        Self {
            base: ResponseMessage::new(id),
            result: DocumentDiagnosticReport {
                kind: DocumentDiagnosticReportKind::Full,
                items,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentDiagnosticReport {
    kind: DocumentDiagnosticReportKind,
    pub items: Vec<Diagnostic>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentDiagnosticReportKind {
    Full,
    Unchanged,
}
