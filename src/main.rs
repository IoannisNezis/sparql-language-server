#[cfg(all(feature = "wasm", feature = "native"))]
compile_error!("feature \"wasm\" and feature \"native\" cannot be enabled at the same time");

mod lsp;
mod message_handler;
mod rpc;
mod server;

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use camino::Utf8PathBuf;
use message_handler::format_raw;
use server::Server;

use clap::{Parser, Subcommand};

/// monza: An SPARQL language server and formatter
#[derive(Debug, Parser)]
#[command(version, about, long_about= None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the language server
    Server,
    /// Run the formatter on a given file
    Format { path: Utf8PathBuf },
}

fn main() {
    // Initialize logging
    #[cfg(feature = "native")]
    log4rs::init_file("/home/ianni/code/sparql-lsp/log4rs.yml", Default::default()).unwrap();

    // Parse command line arguments
    let cli = Cli::parse();
    match cli.command {
        Command::Server => {
            // Start server and listen to stdio
            let mut server = Server::new();
            server.listen_stdio();
        }
        Command::Format { path } => {
            match File::open(path.clone()) {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)
                        .expect("Could not read file");
                    let formatted_contents = format_raw(contents);
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(false)
                        .open(path.clone())
                        .expect("Could not write to file");
                    file.write_all(formatted_contents.as_bytes())
                        .expect("Unable to write");
                }
                Err(e) => {
                    panic!("Could not open file: {}", e)
                }
            };
            println!("Sucessfully formatted {path}");
        }
    };
}
