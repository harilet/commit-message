use std::{
    fs::{self, File},
    io::Read,
};

use clap::Parser;
use serde_json::Value;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Show Config
    #[arg(short, long)]
    config: bool,

    /// Generate Commit Message
    #[arg(short, long)]
    generate: bool,
}

struct OllamaResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
}

#[tokio::main]
async fn main() {
    let mut config_file = get_config_file();
    let client = reqwest::Client::new();

    let cli = Cli::parse();

    if cli.config {
        let mut buffer = String::new();
        config_file
            .read_to_string(&mut buffer)
            .expect("Failed to read file");

        println!("{}", buffer);
    }

    if cli.generate {
        let body = serde_json::json!({
        "model": "codellama:7b",
        "prompt": "Why is the sky blue?"
        });
        let res = client
            .post("http://localhost:11434/api/generate")
            .body(body.to_string())
            .send()
            .await;
        match res {
            Ok(mut data) => {
                while let Some(chunk) = data.chunk().await.unwrap() {
                    let v: Value = serde_json::from_slice(&chunk).unwrap();
                    println!("{}", v["response"]);
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

fn get_config_file() -> File {
    let file_path = "./config.json";
    if fs::metadata(file_path).is_ok() {
        match File::open(file_path) {
            Ok(file) => {
                return file;
            }
            Err(_) => {
                panic!("Config File Open Error")
            }
        }
    } else {
        match File::create(file_path) {
            Ok(file) => {
                return file;
            }
            Err(_) => {
                panic!("Config File Creation Error")
            }
        }
    }
}
