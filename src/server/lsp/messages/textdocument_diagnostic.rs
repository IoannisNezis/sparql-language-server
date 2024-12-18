use serde::{Deserialize, Serialize};

use crate::server::lsp::rpc::{RequestId, RequestMessage, ResponseMessage};
use crate::server::lsp::textdocument::TextDocumentIdentifier;

use super::Diagnostic;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticRequest {
    #[serde(flatten)]
    pub base: RequestMessage,
    pub params: DocumentDiagnosticParams,
}

impl DiagnosticRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
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
    pub fn new(id: &RequestId, items: Vec<Diagnostic>) -> Self {
        Self {
            base: ResponseMessage::success(id),
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
