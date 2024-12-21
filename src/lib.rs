mod server;

use log::error;
use server::Server;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;

fn send_message(writer: &web_sys::WritableStreamDefaultWriter, message: String) {
    let _future = JsFuture::from(writer.write_with_chunk(&message.into()));
}

#[wasm_bindgen]
pub fn init_language_server(writer: web_sys::WritableStreamDefaultWriter) -> Server {
    #[cfg(target_arch = "wasm32")]
    wasm_logger::init(wasm_logger::Config::default());
    Server::new(move |message| send_message(&writer, message))
}

async fn read_message(
    reader: &web_sys::ReadableStreamDefaultReader,
) -> Result<(String, bool), String> {
    match JsFuture::from(reader.read()).await {
        Ok(js_object) => {
            let value = js_sys::Reflect::get(&js_object, &"value".into())
                .map_err(|_| "\"value\" property not present in message")?
                .as_string()
                .ok_or("\"value\" is not a string")?;
            let done = js_sys::Reflect::get(&js_object, &"done".into())
                .map_err(|_| "\"done\" property not present in message")?
                .as_bool()
                .ok_or("\"done\" is not a boolean")?;
            Ok((value, done))
        }
        Err(_) => Err("Error while reading from input-stream".to_string()),
    }
}

#[wasm_bindgen]
impl Server {
    pub async fn listen(&mut self, reader: web_sys::ReadableStreamDefaultReader) {
        loop {
            match read_message(&reader).await {
                Ok((value, done)) => {
                    self.handle_message(value);
                    if done {
                        break;
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
    }
}
