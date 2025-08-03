use std::fs::{exists, File};
use std::net::TcpStream;
use std::io::{Write};
use std::os::unix::fs::MetadataExt;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::io::{BufRead, BufReader};

#[derive(Parser)]
#[command(name = "dropdash")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add { path: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { path } => {
            let mut stream = TcpStream::connect("127.0.0.1:59123").expect("Could not connect to daemon");
            let _exist = exists(&path).expect("Failed to check if file exists");
            if !_exist {
                println!("File path does not exist.");
                return
            }
            let f = File::open(&path).expect("Failed to open file");
            let metadata = f.metadata().expect("Failed to get metadata");

            let msg = json!({ "cmd": "add", "path": path, "size": metadata.size() });
            writeln!(stream, "{msg}").expect("Failed to write to stream");
            
            let mut reader = BufReader::new(stream);
            let mut response = String::new();
            reader.read_line(&mut response).expect("Failed to read line");
            
            println!("Response from daemon: {response}");
        }
    }
}