mod anaysis;
mod configuration;
mod lsp;
mod state;

mod message_handler;

use configuration::Settings;
use log::{debug, error, info};
use lsp::{
    capabilities,
    rpc::{BaseMessage, Header},
    PublishDiagnosticsNotification, PublishDiagnosticsPrarams, ServerInfo,
};
use message_handler::{collect_diagnostics, dispatch};

pub use message_handler::format_raw;
use state::ServerState;

use std::{
    io::{self, BufReader, Read, Write},
    process::exit,
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Server {
    pub(crate) state: ServerState,
    pub(crate) settings: Settings,
    pub(crate) capabilities: capabilities::ServerCapabilities,
    pub(crate) server_info: ServerInfo,
}

#[wasm_bindgen]
impl Server {
    pub fn new() -> Self {
        // Load configuration
        let config = config::Config::builder()
            .add_source(config::File::with_name("fichu").required(false))
            .build()
            .unwrap();
        let settings: Settings = config.try_deserialize().expect("could not load Settings");
        let capabilities = capabilities::ServerCapabilities {
            text_document_sync: capabilities::TextDocumentSyncKind::Incremental,
            hover_provider: true,
            diagnostic_provider: capabilities::DiagnosticOptions {
                identifier: "fichu".to_string(),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
            },
            completion_provider: capabilities::CompletionOptions {
                trigger_characters: vec!["?".to_string()],
            },
            document_formatting_provider: capabilities::DocumentFormattingOptions {},
        };
        let version = env!("CARGO_PKG_VERSION");
        info!("Started Language Server!!1!");
        info!("version: {}", version);
        debug!("Capabilities: {:?}", capabilities);
        debug!("Settings:\n{:?}", settings);
        Self {
            state: ServerState::new(),
            settings,
            capabilities,
            server_info: ServerInfo {
                name: "fichu".to_string(),
                version: Some(version.to_string()),
            },
        }
    }

    pub fn get_version(&self) -> String {
        self.server_info
            .version
            .clone()
            .unwrap_or("not-specified".to_string())
    }

    pub fn handle_message(&mut self, message: Vec<u8>) -> Option<String> {
        dispatch(self, &message)
    }

    fn send_message(&self, message: String) {
        info!("sending message: {}", message);
        print!("Content-Length: {}\r\n\r\n{}", message.len(), message);
        io::stdout().flush().expect("No IO errors or EOFs");
    }

    pub fn publish_diagnostic(&self, uri: String) -> String {
        let notification = PublishDiagnosticsNotification {
            base: BaseMessage::new("textDocument/publishDiagnostics"),
            params: PublishDiagnosticsPrarams {
                uri: uri.clone(),
                diagnostics: collect_diagnostics(&self.state, &uri).collect(),
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
                        self.send_message(response);
                    }
                    _ => {}
                }

                buffer.clear();
            }
        }
    }
}
