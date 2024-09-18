mod analysis;
mod lsp;
mod rpc;
mod server;

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use camino::Utf8PathBuf;
use log::{info, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use server::{format_raw, Server};

use clap::{Parser, Subcommand};

/// fichu: An SPARQL language server and formatter
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

fn configure_logging() {
    let mut app_dir = dirs_next::data_dir().expect("Failed to find data directory");
    app_dir.push("fichu");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
    }
    let log_file_path = app_dir.join("fichu.log");
    //
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
        .build(log_file_path)
        .expect("Failed to create logfile");

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(logfile)))
        .build(Root::builder().appender("file").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).expect("Failed to configure logger");
    info!("{:?}", app_dir);
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    configure_logging();

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
