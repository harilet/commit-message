use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::Read,
};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub(crate) struct AppConfig {
    pub(crate) ollama_server: String,
    pub(crate) model: String,
    pub(crate) system_prompts: Vec<String>,
}

pub(crate) fn write_config_file(app_config: AppConfig) {
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

pub(crate) fn get_config_file_location() -> String {
    let exe_path = env::current_exe().expect("Failed to get current executable path");
    let binding = exe_path
        .parent()
        .expect("Failed to get executable directory")
        .to_path_buf()
        .join("config.json");
    let file_path = binding.to_str().unwrap();
    return file_path.to_owned();
}

pub(crate) fn get_app_config_obejct() -> AppConfig {
    let mut config_file = get_config_file();
    let mut app_config_string = String::new();
    config_file
        .read_to_string(&mut app_config_string)
        .expect("Config File Read Failure");
    return serde_json::from_str(&app_config_string)
        .expect("Config File JSON String To AppConfig Failure");
}
