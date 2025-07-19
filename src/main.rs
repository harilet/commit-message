use std::{
    env,
    fs::{self, File},
    io::Read,
};

use bytes::Bytes;
use clap::Parser;
use git2::{DiffFormat, DiffOptions, Repository};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
struct AppConfig {
    ollama_server: String,
    model: String,
    system_prompts: Vec<String>,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Config Management
    #[arg(short, long)]
    config: bool,

    /// Generate Commit Message
    #[arg(short, long)]
    generate: bool,
}

#[tokio::main]
async fn main() {
    let app_config: AppConfig = get_app_config_obejct();
    let client = reqwest::Client::new();

    let cli = Cli::parse();

    if cli.config {
        println!("ollama_server: {}", app_config.ollama_server);
        println!("model: {}", app_config.model);
        println!("system_prompts: {:?}", app_config.system_prompts);
    }

    if cli.generate {
        genetate_commit_message(app_config.clone(), client).await;
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

async fn genetate_commit_message(app_config: AppConfig, client: Client) {
    println!("{}", app_config.model);
    let diff_data = get_git_diff().join("");

    let body = serde_json::json!({
    "model": app_config.model,
    "prompt": diff_data,
    "system": app_config.system_prompts.join(". ")
    });
    let res = client
        .post(format!("{}/api/generate", app_config.ollama_server))
        .body(body.to_string())
        .send()
        .await;
    match res {
        Ok(mut data) => {
            let mut last_chunk: Vec<u8> = vec![];
            while let Some(chunk) = data.chunk().await.expect("Data chunk error") {
                if chunk.len() < 8186 {
                    if last_chunk.is_empty() {
                        let json_chunk: Result<Value, serde_json::Error> =
                            serde_json::from_slice(&chunk);
                        match json_chunk {
                            Ok(data) => {
                                print!(
                                    "{}",
                                    data["response"].as_str().expect("Response string error")
                                );
                            }
                            Err(e) => {
                                println!("\nError:{:#?}\non message:{:?}", e, chunk);
                            }
                        }
                    } else {
                        last_chunk.append(&mut chunk.to_vec());
                        let complete_chunk = Bytes::from(last_chunk.clone());
                        let json_chunk: Result<Value, serde_json::Error> =
                            serde_json::from_slice(&complete_chunk);
                        match json_chunk {
                            Ok(data) => {
                                print!(
                                    "{}",
                                    data["response"].as_str().expect("Response string error")
                                );
                            }
                            Err(e) => {
                                println!("\nError:{:#?}\non message:{:?}", e, complete_chunk);
                            }
                        }
                        last_chunk.clear();
                    }
                } else {
                    if last_chunk.is_empty() {
                        last_chunk = chunk.to_vec();
                    } else {
                        last_chunk.append(&mut chunk.to_vec());
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

fn get_git_diff() -> Vec<std::string::String> {
    let current_dir = env::current_dir().expect("Error getting env::current_dir()");
    let location = current_dir.as_path();

    let repo = Repository::open(location).expect("Open Repository Failure");
    let mut diff_opts = DiffOptions::new();
    let old_tree = repo
        .head()
        .expect("Failed to get HEAD")
        .peel_to_tree()
        .expect("Head is not a tree");

    let mut diff_data: Vec<String> = vec![];

    repo.diff_tree_to_index(
        Some(&old_tree),
        Some(&repo.index().expect("Failed to index files")),
        Some(&mut diff_opts),
    )
    .expect("Error creating diff")
    .print(DiffFormat::Patch, |_d, _h, l| {
        let content = str::from_utf8(l.content())
            .expect("Content is not utf-8")
            .to_string();
        diff_data.push(format!("{}:{}", l.origin(), content));
        true
    })
    .expect("Error printing diff");
    return diff_data;
}

fn get_app_config_obejct() -> AppConfig {
    let mut config_file = get_config_file();
    let mut app_config_string = String::new();
    config_file
        .read_to_string(&mut app_config_string)
        .expect("Config File Read Failure");
    return serde_json::from_str(&app_config_string)
        .expect("Config File JSON String To AppConfig Failure");
}
