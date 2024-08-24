mod message_handler;
use crate::{
    analysis::AnalysisState,
    lsp::{
        textdocument::TextDocumentItem, PublishDiagnosticsNotification, PublishDiagnosticsPrarams,
        TextDocumentContentChangeEvent,
    },
    rpc::{BaseMessage, Header},
};
use log::{error, info};
use message_handler::{collect_diagnostics, dispatch};

pub use message_handler::format_raw;

use std::{
    io::{self, BufReader, Read, Write},
    process::exit,
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Server {
    state: ServerState,
}

#[wasm_bindgen]
impl Server {
    pub fn new() -> Self {
        info!("Started LSP Server!!");
        Self {
            state: ServerState::new(),
        }
    }

    pub fn handle_message(&mut self, message: Vec<u8>) -> Option<String> {
        dispatch(&message, &mut self.state)
    }

    pub fn publish_diagnostic(&self, uri: String) -> String {
        let notification: PublishDiagnosticsNotification = PublishDiagnosticsNotification {
            base: BaseMessage::new("textDocument/publishDiagnostics".to_string()),
            params: PublishDiagnosticsPrarams {
                uri: uri.clone(),
                diagnostics: collect_diagnostics(&self.state.analysis_state, &uri).collect(),
            },
        };

        serde_json::to_string(&notification)
            .expect("Could not parse PublishDiagnosticsNotification")
    }

    pub fn listen_stdio(&mut self) {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);

        let mut bytes = reader.bytes();
        let mut buffer = vec![];

        loop {
            match bytes.next() {
                Some(Ok(byte)) => {
                    buffer.push(byte);
                }
                Some(Err(error)) => {
                    error!("Error while reading byte: {}", error);
                    panic!("{}", error);
                }
                None => {
                    error!("Stream ended unexpected while waiting for header, shutting down");
                    exit(1);
                }
            }
            if buffer.ends_with(b"\r\n\r\n") {
                let header = match Header::from_string(
                    String::from_utf8(buffer.clone()).expect("valid utf8 bytes"),
                ) {
                    Ok(header) => header,
                    Err(err) => {
                        error!("Received error while parsing header: {err}, clearing buffer");
                        buffer.clear();
                        continue;
                    }
                };
                buffer.clear();
                for ele in 0..header.content_length {
                    match bytes.next() {
                        Some(Ok(byte)) => {
                            buffer.push(byte);
                        }
                        Some(Err(err)) => {
                            error!(
                                "Error {} occured while reading byte {} of {}, clearing buffer",
                                err, ele, header.content_length
                            );
                            buffer.clear();
                            break;
                        }
                        None => {
                            error!(
                                "Byte stream endet after {} of {} bytes, clearing message buffer",
                                ele, header.content_length
                            );
                            buffer.clear();
                            break;
                        }
                    }
                }
                match self.handle_message(buffer.clone()) {
                    Some(response) => {
                        print!("Content-Length: {}\r\n\r\n{}", response.len(), response);
                        io::stdout().flush().expect("No IO errors or EOFs");
                    }
                    _ => {}
                }

                buffer.clear();
            }
        }
    }
}

#[derive(Debug)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

pub struct ServerState {
    pub status: ServerStatus,
    pub analysis_state: AnalysisState,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            analysis_state: AnalysisState::new(),
        }
    }

    pub fn add_document(&mut self, document: TextDocumentItem) {
        self.analysis_state.add_document(document);
    }

    pub(crate) fn change_document(
        &mut self,
        document_uri: String,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        self.analysis_state
            .change_document(document_uri, content_changes)
    }
}
