#[cfg(all(feature = "native", feature = "wasm"))]
compile_error!("feature \"native\" and feature \"wasm\" cannot be enabled at the same time");

mod lsp;
mod message_handler;
mod rpc;
mod server;
// use message_handler::*;
