mod server;
mod stdio_reader;

use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, Write},
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
use stdio_reader::listen_stdio;

/// qlue-ls: An SPARQL language server and formatter
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
    Format {
        /// overwrite given file
        #[arg(short, long)]
        writeback: bool,
        path: Utf8PathBuf,
    },
    /// Watch the logs
    Logs,
}

fn get_logfile_path() -> PathBuf {
    let mut app_dir = dirs_next::data_dir().expect("Failed to find data directory");
    app_dir.push("qlue-ls");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("Failed to create app directory");
    }
    app_dir.join("qlue-ls.log")
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

fn send_message(message: String) {
    print!("Content-Length: {}\r\n\r\n{}", message.len(), message);
    io::stdout().flush().expect("No IO errors or EOFs");
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    configure_logging();

    let cli = Cli::parse();
    match cli.command {
        Command::Server => {
            // Start server and listen to stdio
            let mut server = Server::new(send_message);
            listen_stdio(|message| server.handle_message(message));
        }
        Command::Format { path, writeback } => {
            match File::open(path.clone()) {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)
                        .expect(&format!("Could not read file {}", path));
                    let formatted_contents = format_raw(contents);
                    if writeback {
                        let mut file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .append(false)
                            .open(path.clone())
                            .expect("Could not write to file");
                        file.write_all(formatted_contents.as_bytes())
                            .expect("Unable to write");
                        println!("Sucessfully formatted {path}");
                    } else {
                        println!("{}", formatted_contents);
                    }
                }
                Err(e) => {
                    panic!("Could not open file: {}", e)
                }
            };
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
