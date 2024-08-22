use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{lsp::textdocument::Range, rpc::BaseMessage};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublishDiagnosticsNotification {
    #[serde(flatten)]
    pub base: BaseMessage,
    pub params: PublishDiagnosticsPrarams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublishDiagnosticsPrarams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    // code: u32 | String,
    // codeDescription: CodeDescription
    pub source: String,
    pub message: String,
    // tags
    // relatedInformation
    // data
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticSeverity
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}
