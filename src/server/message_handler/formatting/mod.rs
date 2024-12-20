mod core;
mod utils;
use core::*;

use tree_sitter::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::server::{
    configuration::FormatSettings,
    lsp::{errors::ResponseError, FormattingRequest, FormattingResponse},
    Server,
};

pub fn handle_format_request(
    server: &mut Server,
    request: FormattingRequest,
) -> Result<FormattingResponse, ResponseError> {
    let (document, tree) = server.state.get_state(request.get_document_uri())?;
    let options = request.get_options();
    let text_edits = format_textdoument(document, tree, &server.settings.format_settings, options);
    Ok(FormattingResponse::new(request.get_id(), text_edits))
}

#[wasm_bindgen]
pub fn format_raw(text: String) -> String {
    let mut parser = Parser::new();
    // TODO: use user configuration
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
