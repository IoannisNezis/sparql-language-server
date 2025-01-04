mod anaysis;
mod capabilities;
mod common;
mod configuration;
mod lsp;
mod state;
mod tools;

mod message_handler;

use capabilities::create_capabilities;
use configuration::Settings;
use log::{error, info};
use lsp::{
    rpc::{RecoverId, RequestIdOrNull, ResponseMessage},
    ServerInfo,
};
use message_handler::dispatch;

// WARNING: This is a temporary soloution to export the format function directly
// will remove soon (12.12.24)
#[allow(unused_imports)]
pub use message_handler::format_raw;

use state::ServerState;
use tools::Tools;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Server {
    pub(crate) state: ServerState,
    pub(crate) settings: Settings,
    pub(crate) capabilities: lsp::capabilities::ServerCapabilities,
    pub(crate) server_info: ServerInfo,
    tools: Tools,
    send_message_clusure: Box<dyn Fn(String)>,
}

impl Server {
    pub fn new(write_function: impl Fn(String) -> () + 'static) -> Server {
        let version = env!("CARGO_PKG_VERSION");
        info!("Started Language Server: Qlue-ls - version: {}", version);
        Self {
            state: ServerState::new(),
            settings: Settings::new(),
            capabilities: create_capabilities(),
            server_info: ServerInfo {
                name: "Qlue-ls".to_string(),
                version: Some(version.to_string()),
            },
            tools: Tools::initiate(),
            send_message_clusure: Box::new(write_function),
        }
    }

    pub fn get_version(&self) -> String {
        self.server_info
            .version
            .clone()
            .unwrap_or("not-specified".to_string())
    }

    pub fn handle_message(&mut self, message: String) {
        match dispatch(self, &message) {
            Ok(None) => {
                // NOTE: message was a notification -> Nothing to do
            }
            Ok(Some(response)) => self.send_message(response),
            Err(error) => {
                if let Some(id) = serde_json::from_str::<RecoverId>(&message)
                    .map(|msg| RequestIdOrNull::RequestId(msg.id))
                    .ok()
                {
                    let response = ResponseMessage::error(id, error);
                    match serde_json::to_string(&response) {
                        Ok(response_string) => {
                            self.send_message(response_string);
                        }
                        Err(error) => {
                            error!(
                            "CRITICAL: could not serialize error message (this very bad):\n{:?}\n{}",
                            response, error
                        )
                        }
                    }
                }
            }
        }
    }

    fn send_message(&self, message: String) {
        (self.send_message_clusure)(message);
    }

    /// Shortens a raw URI into its CURIE (Compact URI) form and retrieves related metadata.
    ///
    /// This method takes a raw URI as input, attempts to find its associated prefix and URI prefix
    /// from the `uri_converter`, and shorten the URI into its CURIE form. If successful, it
    /// returns a tuple containing:
    /// - The prefix associated with the URI.
    /// - The URI prefix corresponding to the namespace of the URI.
    /// - The CURIE representation of the URI.
    ///
    /// # Parameters
    /// - `uri`: A string slice representing the raw URI to be shortened.
    ///
    /// # Returns
    /// - `Some((prefix, uri_prefix, curie))` if the URI can be successfully compacted:
    ///   - `prefix`: A `String` representing the prefix associated with the URI.
    ///   - `uri_prefix`: A `String` representing the URI namespace prefix.
    ///   - `curie`: A `String` representing the compact CURIE form of the URI.
    /// - `None` if the URI cannot be found or shortened.
    ///
    /// # Example
    /// ```rust
    /// let uri = "http://example.com/resource";
    /// if let Some((prefix, uri_prefix, curie)) = instance.shorten_uri(uri) {
    ///     println!("Prefix: {}", prefix);
    ///     println!("URI Prefix: {}", uri_prefix);
    ///     println!("CURIE: {}", curie);
    /// } else {
    ///     println!("Failed to shorten URI.");
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns `None` if:
    /// - The `uri_converter` fails to find a record associated with the URI.
    /// - The `uri_converter` fails to shorten the URI into a CURIE.
    pub(crate) fn shorten_uri(&self, uri: &str) -> Option<(String, String, String)> {
        let record = self.tools.uri_converter.find_by_uri(uri).ok()?;
        let curie = self.tools.uri_converter.compress(uri).ok()?;
        Some((record.prefix.clone(), record.uri_prefix.clone(), curie))
    }
}

#[cfg(test)]
mod test {
    use super::Server;

    use super::message_handler::dispatch;

    #[test]
    fn initialize() {
        let mut server = Server::new(|_message| {});
        assert!(dbg!(dispatch(
            &mut server,
            &r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":1}}"#
                .to_string(),
        ))
        .is_ok());
        assert!(dbg!(dispatch(
            &mut server,
            &r#"{"jsonrpc":"2.0","method":"initialized"}"#.to_string(),
        )
        .is_ok()));
    }
}
