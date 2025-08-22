use clap::{Parser, Subcommand};
use ollama_rs::{
    Ollama,
    coordinator::Coordinator,
    error,
    generation::chat::{
        ChatMessage, ChatMessageResponse, ChatMessageResponseStream, request::ChatMessageRequest,
    },
};

use reqwest::Url;
use tokio::fs;
use utils::git::{get_current_branch_name, get_project_struture};

use std::{
    io::{self, Write, stdin, stdout},
    sync::{Arc, Mutex},
};
use tokio_stream::StreamExt;

mod utils;
use crate::utils::config::{
    AppConfig, get_app_config_obejct, get_config_file_location, write_config_file,
};
use crate::utils::git::get_git_diff;

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Generate Commit Message
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generation Commit Messages
    Generate {
        /// Use the spedified model
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
    match &cli.command {
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
                            commit_message: app_config.commit_message.clone(),
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
                println!("commit_message: {:?}", app_config.commit_message);
            }
        }
        None => {}
    }
}

async fn genetate_commit_message(app_config: AppConfig, model: String) {
    println!("{}", model.clone());

    let mut input: String = String::new();
    let mut history = vec![];
    history.push(ChatMessage::system(app_config.system_prompts.join(". ")));

    history.push(ChatMessage::user(app_config.commit_message.join(". ")));

    history.push(ChatMessage::user(format!(
        "This is project folder structure: {}",
        get_project_struture().unwrap().join(",")
    )));

    let ollama = Ollama::from_url(Url::parse(&app_config.ollama_server).unwrap());

    let mut coordinator = Coordinator::new(ollama, model.clone(), history).add_tool(get_file);

    let diff_data = get_git_diff().join("");

    let mut messages: Vec<ChatMessage> = vec![];

    messages.push(ChatMessage::user(format!(
        "The branch name is {}",
        get_current_branch_name()
    )));

    messages.push(ChatMessage::user("Folllowing is the changes made that need the commit message".to_owned()));

    messages.push(ChatMessage::user(diff_data));

    loop {
        if !input.is_empty() {
            messages.push(ChatMessage::user(input));
        }
        let res = coordinator.chat(messages.clone()).await;
        match res {
            Ok(data) => {
                handle_ollama_response(data).await;
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        messages.clear();
        input = get_input("\n\"/bye\" to exit: ".to_owned());
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

fn get_input(input_prompt: String) -> String {
    let mut s = String::new();
    print!("{}", input_prompt);
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

async fn handle_ollama_response(stream: ChatMessageResponse) {
    print!("{}", stream.message.content.as_str());
    io::stdout().flush().unwrap();
}

/// Get file contents from a file path.
///
/// * file_path - The file path to read from.
#[ollama_rs::function]
async fn get_file(file_path: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    println!("file_path: {file_path}");
    let file_contents = fs::read_to_string(&file_path).await?;
    Ok(file_contents)
}
