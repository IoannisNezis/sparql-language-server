use core::panic;
use std::{
    io::{self, BufReader, Read},
    process::exit,
};

use log::error;

pub fn listen_stdio(mut message_handler: impl FnMut(String) -> ()) {
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
            let cl_slice = buffer
                .get(16..(buffer.len() - 4))
                .expect("Header does not have a 'Content-Length: '");
            let cl_string =
                String::from_utf8(cl_slice.to_vec().clone()).expect("Invalid UTF-8 data");
            let content_length: u32 = cl_string.parse().expect("Failed to parse Content-Length");
            buffer.clear();
            for ele in 0..content_length {
                match bytes.next() {
                    Some(Ok(byte)) => {
                        buffer.push(byte);
                    }
                    Some(Err(err)) => {
                        error!(
                            "Error {} occured while reading byte {} of {}, clearing buffer",
                            err, ele, content_length
                        );
                        buffer.clear();
                        break;
                    }
                    None => {
                        error!(
                            "Byte stream endet after {} of {} bytes, clearing message buffer",
                            ele, content_length
                        );
                        buffer.clear();
                        break;
                    }
                }
            }
            (message_handler)(String::from_utf8(buffer.clone()).unwrap());
            buffer.clear();
        }
    }
}
