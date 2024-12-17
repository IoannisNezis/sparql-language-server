mod anaysis;
mod commands;
mod configuration;
mod lsp;
mod state;

mod message_handler;

use configuration::Settings;
use curies::{Converter, Record};
use log::{debug, error, info};
use lsp::{
    capabilities::{self, ExecuteCommandOptions, WorkDoneProgressOptions},
    ServerInfo,
};
use message_handler::dispatch;

// WARNING: This is a temporary soloution to export the format function directly
// will remove soon (12.12.24)
#[allow(unused_imports)]
pub use message_handler::format_raw;

use state::ServerState;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Server {
    pub(crate) state: ServerState,
    pub(crate) settings: Settings,
    pub(crate) capabilities: capabilities::ServerCapabilities,
    pub(crate) server_info: ServerInfo,
    uri_converter: Converter,
    send_message_clusure: Box<dyn Fn(String)>,
}

impl Server {
    pub fn new(write_function: impl Fn(String) -> () + 'static) -> Server {
        let config = config::Config::builder()
            .add_source(config::File::with_name("qlue-ls").required(false))
            .build()
            .unwrap();
        let settings: Settings = config.try_deserialize().expect("could not load Settings");
        let capabilities = capabilities::ServerCapabilities {
            text_document_sync: capabilities::TextDocumentSyncKind::Incremental,
            hover_provider: true,
            code_action_provider: true,
            execute_command_provider: ExecuteCommandOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: true,
                },
                commands: vec!["publish diagnostics".to_string()],
            },
            diagnostic_provider: capabilities::DiagnosticOptions {
                identifier: "qlue-ls".to_string(),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
            },
            completion_provider: capabilities::CompletionOptions {
                trigger_characters: vec!["?".to_string()],
            },
            document_formatting_provider: capabilities::DocumentFormattingOptions {},
        };
        let mut uri_converter = Converter::new(":");
        uri_converter.add_record(Record::new("schema", "http://schema.org/"));
        uri_converter.add_record(Record::new(
            "envTopic",
            "https://environment.ld.admin.ch/foen/nfi/Topic/",
        ));
        uri_converter.add_record(Record::new(
            "envCube",
            "https://environment.ld.admin.ch/foen/nfi/nfi_C-20/cube/",
        ));

        uri_converter.add_record(Record::new("cube", "https://cube.link/"));
        uri_converter.add_record(Record::new(
            "env",
            "https://environment.ld.admin.ch/foen/nfi/",
        ));
        uri_converter.add_record(Record::new("country", "https://ld.admin.ch/country/"));

        let version = env!("CARGO_PKG_VERSION");
        info!("Started Language Server: Qlue-ls - version: {}", version);
        Self {
            state: ServerState::new(),
            settings,
            capabilities,
            server_info: ServerInfo {
                name: "Qlue-ls".to_string(),
                version: Some(version.to_string()),
            },
            uri_converter,
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
        match dispatch(self, message) {
            Some(response) => self.send_message(response),
            None => {}
        }
    }

    fn send_message(&self, message: String) {
        (self.send_message_clusure)(message);
    }

    /// Compresses a raw URI into its CURIE (Compact URI) form and retrieves related metadata.
    ///
    /// This method takes a raw URI as input, attempts to find its associated prefix and URI prefix
    /// from the `uri_converter`, and compresses the URI into its CURIE form. If successful, it
    /// returns a tuple containing:
    /// - The prefix associated with the URI.
    /// - The URI prefix corresponding to the namespace of the URI.
    /// - The compressed CURIE representation of the URI.
    ///
    /// # Parameters
    /// - `uri`: A string slice representing the raw URI to be compressed.
    ///
    /// # Returns
    /// - `Some((prefix, uri_prefix, curie))` if the URI can be successfully compressed:
    ///   - `prefix`: A `String` representing the prefix associated with the URI.
    ///   - `uri_prefix`: A `String` representing the URI namespace prefix.
    ///   - `curie`: A `String` representing the compressed CURIE form of the URI.
    /// - `None` if the URI cannot be found or compressed.
    ///
    /// # Example
    /// ```rust
    /// let uri = "http://example.com/resource";
    /// if let Some((prefix, uri_prefix, curie)) = instance.compress_uri(uri) {
    ///     println!("Prefix: {}", prefix);
    ///     println!("URI Prefix: {}", uri_prefix);
    ///     println!("CURIE: {}", curie);
    /// } else {
    ///     println!("Failed to compress URI.");
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns `None` if:
    /// - The `uri_converter` fails to find a record associated with the URI.
    /// - The `uri_converter` fails to compress the URI into a CURIE.
    pub(crate) fn compress_uri(&self, uri: &str) -> Option<(String, String, String)> {
        let record = self.uri_converter.find_by_uri(uri).ok()?;
        let curie = self.uri_converter.compress(uri).ok()?;
        Some((record.prefix.clone(), record.uri_prefix.clone(), curie))
    }
}
