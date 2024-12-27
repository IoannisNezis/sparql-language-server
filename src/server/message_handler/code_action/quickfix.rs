use std::collections::HashMap;

use crate::server::{
    lsp::{
        base_types::LSPAny,
        errors::{ErrorCode, ResponseError},
        textdocument::{Range, TextEdit},
        CodeAction, CodeActionKind, WorkspaceEdit,
    },
    Server,
};

use super::{Diagnostic, DiagnosticCode};

pub(super) fn get_quickfix(
    server: &Server,
    document_uri: &String,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, ResponseError> {
    match diagnostic.code {
        Some(DiagnosticCode::String(ref diagnostic_code)) => match diagnostic_code.as_str() {
            "undeclared-prefix" => undeclared_prefix(server, document_uri, diagnostic),
            _ => {
                log::warn!("Unknown diagnostic code: {}", diagnostic_code);
                Ok(None)
            }
        },
        _ => Ok(None),
    }
}

fn undeclared_prefix(
    server: &Server,
    document_uri: &String,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, ResponseError> {
    if let Some(LSPAny::String(prefix)) = &diagnostic.data {
        if let Ok(record) = server.tools.uri_converter.find_by_prefix(&prefix) {
            Ok(Some(CodeAction {
                title: format!("Declare prefix \"{}\"", prefix),
                kind: Some(CodeActionKind::QuickFix),
                edit: WorkspaceEdit {
                    changes: HashMap::from([(
                        document_uri.clone(),
                        vec![TextEdit::new(
                            Range::new(0, 0, 0, 0),
                            &format!("PREFIX {}: <{}>\n", prefix, record.uri_prefix),
                        )],
                    )]),
                },
                diagnostics: vec![diagnostic],
            }))
        } else {
            Ok(None)
        }
    } else {
        Err(ResponseError::new(
            ErrorCode::InvalidParams,
            "expected prefix in undeclared-prefix data... was disapointed",
        ))
    }
}
