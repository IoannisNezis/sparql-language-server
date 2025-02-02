mod core;
mod utils;
use core::*;

use tree_sitter::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::server::{
    configuration::Settings,
    lsp::{
        errors::ResponseError, textdocument::TextDocumentItem, FormattingOptions,
        FormattingRequest, FormattingResponse,
    },
    Server,
};

pub(super) fn handle_format_request(
    server: &mut Server,
    request: FormattingRequest,
) -> Result<FormattingResponse, ResponseError> {
    let (document, tree) = server.state.get_state(request.get_document_uri())?;
    let edits = format_document(
        &document,
        tree,
        request.get_options(),
        &server.settings.format,
    )?;
    Ok(FormattingResponse::new(request.get_id(), edits))
}

#[wasm_bindgen]
pub fn format_raw(text: String) -> Result<String, String> {
    let mut parser = Parser::new();
    let settings = Settings::new();
    match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
        Ok(()) => {
            let tree = parser
                .parse(text.as_bytes(), None)
                .expect("Input should be parsed successfully");
            let mut document = TextDocumentItem::new("tmp", &text);
            let edits = format_document(
                &document,
                &tree,
                &FormattingOptions {
                    tab_size: 2,
                    insert_spaces: true,
                },
                &settings.format,
            )
            .map_err(|err| err.message)?;
            document.apply_text_edits(edits);
            return Ok(document.text);
        }
        Err(_) => panic!("Could not setup parser"),
    }
}

#[cfg(test)]
mod tests;
