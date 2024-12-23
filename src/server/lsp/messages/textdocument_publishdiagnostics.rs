use serde::{Deserialize, Serialize};

use crate::server::lsp::rpc::NotificationMessageBase;

use super::diagnostic::Diagnostic;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublishDiagnosticsNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: PublishDiagnosticsPrarams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublishDiagnosticsPrarams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}
