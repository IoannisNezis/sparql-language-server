#[cfg(all(feature = "native", feature = "wasm"))]
compile_error!("feature \"native\" and feature \"wasm\" cannot be enabled at the same time");

mod lsp;
mod message_handler;
mod rpc;
mod server;

use server::Server;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() -> Server {
    #[cfg(feature = "wasm")]
    wasm_logger::init(wasm_logger::Config::default());
    return Server::new();
}
