mod analysis;
mod lsp;
mod rpc;
mod server;

use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, Write},
    path::PathBuf,
    sync::mpsc::channel,
};

use camino::Utf8PathBuf;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
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
    /// Watch the logs
    Logs,
}

fn get_logfile_path() -> PathBuf {
    let mut app_dir = dirs_next::data_dir().expect("Failed to find data directory");
    app_dir.push("fichu");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
    }
    app_dir.join("fichu.log")
}

fn configure_logging() {
    let logfile_path = get_logfile_path();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
        .build(logfile_path)
        .expect("Failed to create logfile");

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(logfile)))
        .build(Root::builder().appender("file").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).expect("Failed to configure logger");
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
        Command::Logs => {
            let logfile_path = get_logfile_path();
            // Open the file and seek to the end (to mimic `tail -f` behavior)
            let mut file = File::open(&logfile_path).expect("Could not open file");
            let mut pos = std::fs::metadata(&logfile_path)
                .expect("could not read file metatdaa")
                .len();

            let (tx, rx) = channel();

            // Create a file watcher
            let mut watcher: RecommendedWatcher =
                Watcher::new(tx, notify::Config::default()).expect("Could not create watcher");

            // Start watching the file
            watcher
                .watch(logfile_path.as_ref(), RecursiveMode::NonRecursive)
                .expect("Could not watch file");

            for res in rx {
                match res {
                    Ok(_event) => {
                        // ignore any event that didn't change the pos
                        if file.metadata().unwrap().len() == pos {
                            continue;
                        }

                        // read from pos to end of file
                        file.seek(std::io::SeekFrom::Start(pos)).unwrap();

                        // update post to end of file
                        pos = file.metadata().unwrap().len();

                        let reader = BufReader::new(&file);
                        for line in reader.lines() {
                            println!("{}", line.unwrap());
                        }
                    }
                    Err(error) => println!("{error:?}"),
                }
            }
        }
    };
}
