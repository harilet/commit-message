use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write, stdin, stdout},
    sync::{Arc, Mutex},
};
use clap::{Parser, Subcommand};
use git2::{DiffFormat, DiffOptions, Repository};
use ollama_rs::error;
use ollama_rs::{
    Ollama,
    generation::chat::{ChatMessage, ChatMessageResponseStream, request::ChatMessageRequest},
};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

#[derive(Deserialize, Debug, Clone, Serialize)]
struct AppConfig {
    ollama_server: String,
    model: String,
    system_prompts: Vec<String>,
}

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Generate Commit Message
    #[command(subcommand)]
    generate: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generation Commit Messages
    Generate {
        /// lists test values
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Config Managet
    Config {
        /// To Set Config
        /// e.g model=qwen3:8b
        #[arg(short, long)]
        set_config: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let app_config: AppConfig = get_app_config_obejct();

    let cli = Cli::parse();
    match &cli.generate {
        Some(Commands::Generate { model }) => {
            let use_model: String;
            match model {
                Some(model_string) => {
                    use_model = model_string.to_string();
                }
                None => {
                    use_model = app_config.model.clone();
                }
            }
            genetate_commit_message(app_config.clone(), use_model).await;
        }
        Some(Commands::Config { set_config }) => {
            let mut show_config = true;
            match set_config {
                Some(set_config_string) => {
                    show_config = false;
                    let set_config_list: Vec<&str> = set_config_string.split("=").collect();
                    if *set_config_list.get(0).unwrap() == "model" {
                        write_config_file(AppConfig {
                            ollama_server: app_config.ollama_server.clone(),
                            model: set_config_list.get(1).unwrap().to_string(),
                            system_prompts: app_config.system_prompts.clone(),
                        });
                    }
                }
                None => {}
            }
            if show_config {
                println!("Config File: {}", get_config_file_location());
                println!("");
                println!("ollama_server: {}", app_config.ollama_server);
                println!("model: {}", app_config.model);
                println!("system_prompts: {:?}", app_config.system_prompts);
            }
        }
        None => {}
    }
}

fn write_config_file(app_config: AppConfig) {
    let config_file = get_config_file();
    match serde_json::to_writer_pretty(config_file, &app_config) {
        Ok(_) => {
            println!("Config Updated");
        }
        Err(e) => {
            println!("Config Updated Failed: {:#?}", e);
        }
    };
}

fn get_config_file() -> File {
    let file_path = get_config_file_location();
    if fs::metadata(&file_path).is_ok() {
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

fn get_config_file_location() -> String {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let binding = exe_path
        .parent()
        .expect("Failed to get executable directory")
        .to_path_buf()
        .join("config.json");
    let file_path = binding.to_str().unwrap();
    return file_path.to_owned();
}

async fn genetate_commit_message(app_config: AppConfig, model: String) {
    println!("{}", model.clone());

    let mut input: String = String::new();
    let history: Arc<Mutex<Vec<ChatMessage>>> = Arc::new(Mutex::new(vec![]));
    history
        .lock()
        .unwrap()
        .push(ChatMessage::system(app_config.system_prompts.join(". ")));

    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    let diff_data = get_git_diff().join("");

    let mut messages: Vec<ChatMessage> = vec![];

    messages.push(ChatMessage::user(diff_data));

    loop {
        if !input.is_empty() {
            messages.push(ChatMessage::user(input));
        }
        let res = sent_message(&ollama, &history, &model, &messages).await;
        match res {
            Ok(data) => {
                handle_ollama_response(data).await;
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        messages.clear();
        input = get_input();
        if input == "/bye" {
            break;
        }
    }
}

async fn sent_message(
    ollama: &Ollama,
    history: &Arc<Mutex<Vec<ChatMessage>>>,
    model: &String,
    messages: &Vec<ChatMessage>,
) -> error::Result<ChatMessageResponseStream> {
    let temp_history = history.to_owned();
    let res = ollama
        .send_chat_messages_with_history_stream(
            temp_history,
            ChatMessageRequest::new(model.to_owned(), messages.to_owned()),
        )
        .await;

    return res;
}

fn get_input() -> String {
    let mut s = String::new();
    print!("\n\"/bye\" to exit: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    return s;
}

async fn handle_ollama_response(mut stream: ChatMessageResponseStream) {
    while let Some(Ok(data)) = stream.next().await {
        print!("{}", data.message.content.as_str());
        io::stdout().flush().unwrap();
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
