mod core;
mod utils;
use core::*;
use log::{error, info};

use tree_sitter::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    lsp::{FormattingRequest, FormattingResponse},
    server::{
        configuration::{FormatSettings, Settings},
        ServerState,
    },
};

pub fn handle_format_request(
    request: FormattingRequest,
    state: &mut ServerState,
    settings: &Settings,
) -> FormattingResponse {
    let uri = request.get_document_uri();
    info!("Received formatting request for: {}", uri);
    match state.analysis_state.get_state(uri) {
        Some((document, Some(tree))) => {
            let options = request.get_options();
            let text_edits = format_textdoument(document, tree, &settings.format, options);
            FormattingResponse::new(request.get_id(), text_edits)
        }
        _ => {
            error!("Requested formatting for unknown document: {}", uri);
            todo!()
        }
    }
}

#[wasm_bindgen]
pub fn format_raw(text: String) -> String {
    let mut parser = Parser::new();
    // TODO: look for user configuration
    let format_settings = FormatSettings::default();
    match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
        Ok(()) => {
            let tree = parser.parse(text.clone(), None).expect("could not parse");
            let formatted_text =
                format_helper(&text, &mut tree.walk(), 0, "  ", "", &format_settings);
            return formatted_text;
        }
        Err(_) => panic!("Could not setup parser"),
    }
}

#[cfg(test)]
mod tests;
