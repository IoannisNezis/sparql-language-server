mod lsp;
mod rpc;
mod state;

use std::{
    io::{self, BufReader, Read},
    process::exit,
};

use log::{error, info, warn};
use serde::Serialize;
use state::ServerState;

use crate::{
    lsp::{analysis::AnalysisState, DidOpenTextDocumentNotification, InitializeResonse},
    rpc::{Header, ResponseMessage},
};

fn handle_message(bytes: &Vec<u8>, state: &mut ServerState) {
    // info!("{}", String::from_utf8(bytes.to_vec()).unwrap());
    if let Ok(message) = rpc::decode_message(bytes) {
        match message.method.as_str() {
            "initialize" => {
                let initialize_request =
                    serde_json::from_slice::<lsp::InitializeRequest>(bytes).unwrap();
                info!(
                    "Connected to: {} {}",
                    initialize_request.params.client_info.name,
                    initialize_request
                        .params
                        .client_info
                        .version
                        .unwrap_or("no version specified".to_string())
                );
                let initialize_response = InitializeResonse::new(initialize_request.base.id);
                send_message(&initialize_response);
            }
            "initialized" => {
                info!("initialization completed");
                state.status = state::ServerStatus::Running;
            }
            "shutdown" => {
                let shutdown_request =
                    serde_json::from_slice::<rpc::RequestMessage>(bytes).unwrap();
                info!("recieved shutdown request, preparing to shut down");
                let response = ResponseMessage {
                    jsonrpc: "2.0".to_string(),
                    id: shutdown_request.id,
                };
                send_message(&response);
                state.status = state::ServerStatus::ShuttingDown;
            }
            "exit" => {
                info!("recieved exit notification, shutting down!");
                exit(0);
            }
            "textDocument/didOpen" => {
                let did_open_notification: DidOpenTextDocumentNotification =
                    serde_json::from_slice(bytes).unwrap();
                info!(
                    "opened text document: \"{}\"\n{}",
                    did_open_notification.params.text_document.uri,
                    did_open_notification.params.text_document.text
                );
                state.add_document(did_open_notification.params);
            }
            method => {
                warn!(
                    "Received message with unknown method \"{method}\": {:?}",
                    String::from_utf8(bytes.to_vec()).unwrap()
                );
            }
        };
    } else {
        error!("An error occured while parsing the request content");
    }
}

// TODO: This trait should be narrowed down, Serialize is not enougth to be jsonrpc message.
fn send_message<T: Serialize>(message_body: &T) {
    let message_body_string = rpc::encode(&message_body);
    // info!("sending: {}", message_body_string);
    println!(
        "Content-Length: {}\r\n\r\n{}",
        message_body_string.len(),
        message_body_string
    );
}

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    info!("Started LSP Server!");

    let mut server_state = state::ServerState::new();
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
            handle_message(&buffer, &mut server_state);
            buffer.clear();
        }
    }
}
