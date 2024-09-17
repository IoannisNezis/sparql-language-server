mod analysis;
mod lsp;
mod rpc;
mod server;

use server::Server;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_language_server() -> Server {
    #[cfg(target_arch = "wasm32")]
    wasm_logger::init(wasm_logger::Config::default());
    return Server::new();
}
