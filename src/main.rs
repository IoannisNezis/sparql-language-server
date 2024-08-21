mod lsp;
mod rpc;

use std::{
    io::{self, BufReader, Read},
    process,
};

use log::{error, info, warn};

use crate::{lsp::InitializeResonse, rpc::Header};

fn handle_message(bytes: &Vec<u8>) {
    // info!("{}", String::from_utf8(bytes.to_vec()).unwrap());
    if let Ok(message) = rpc::decode_message(bytes) {
        match message.method.as_str() {
            "initialize" => {
                let init_request = serde_json::from_slice::<lsp::InitializeRequest>(bytes).unwrap();
                info!(
                    "Connected to: {} {}",
                    init_request.params.client_info.name,
                    init_request
                        .params
                        .client_info
                        .version
                        .unwrap_or("no version specified".to_string())
                );
                let init_response = InitializeResonse::new(init_request.base.id);

                // NOTE: This could be done in rpc, maybe with traits (serialize)?
                // TODO: remove unwrap
                let response_string = serde_json::to_string(&init_response).unwrap();
                println!(
                    "Content-Length: {}\r\n\r\n{}",
                    response_string.len(),
                    response_string
                );
            }
            "shutdown" => {
                info!("recieved shutdown notification, shutting down");
                process::exit(0);
            }
            method => {
                warn!(
                    "Received message with unknown method \"{method}\": {:?}",
                    serde_json::to_string(&message).unwrap()
                );
            }
        };
    } else {
        error!("An error occured while parsing the request content");
    }
}

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    info!("Started LSP Server!");

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
                // TODO: Handle more gracefully
                error!("Stream ended unexpected while waiting for header, shutting down");
                break;
                continue;
            }
        }
        if buffer.ends_with(b"\r\n\r\n") {
            // TODO: Handle error better
            let header = match Header::from_string(
                String::from_utf8(buffer.clone()).expect("valid utf9 bytes"),
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
            handle_message(&buffer);
            buffer.clear();
        }
    }
    info!("Shutting down server");
}
