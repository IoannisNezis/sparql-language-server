mod core;
mod utils;
use core::*;

use tree_sitter::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::server::{
    configuration::Settings,
    lsp::{errors::ResponseError, FormattingRequest, FormattingResponse},
    Server,
};

pub(super) fn handle_format_request(
    server: &mut Server,
    request: FormattingRequest,
) -> Result<FormattingResponse, ResponseError> {
    let (document, _tree) = server.state.get_state(request.get_document_uri())?;
    let edits = format_pipeline(
        &document,
        &mut server.tools.parser,
        request.get_options(),
        &server.settings.format,
    );
    Ok(FormattingResponse::new(request.get_id(), edits))
}

#[wasm_bindgen]
pub fn format_raw(text: String) -> String {
    let mut parser = Parser::new();
    let settings = Settings::new();
    match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
        Ok(()) => {
            let tree = parser
                .parse(text.as_bytes(), None)
                .expect("could not parse");
            let formatted_text =
                format_parse1(&text, &mut tree.walk(), 0, "  ", "", &settings.format) + "\n";
            return formatted_text;
        }
        Err(_) => panic!("Could not setup parser"),
    }
}

#[cfg(test)]
mod tests;
