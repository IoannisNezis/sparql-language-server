mod lsp;
mod message_handler;
mod rpc;
mod state;

use std::{
    io::{self, BufReader, Read},
    process::exit,
};

use log::{error, info};

use crate::{message_handler::dispatch, rpc::Header};

fn main() {
    // Initialize logging
    log4rs::init_file("/home/ianni/code/sparql-lsp/log4rs.yml", Default::default()).unwrap();
    info!("Started LSP Server!");

    // Initialize input stream
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut bytes = reader.bytes();
    let mut buffer = vec![];

    // Initialize the server state
    let mut server_state = state::ServerState::new();

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
            dispatch(&buffer, &mut server_state);
            buffer.clear();
        }
    }
}
