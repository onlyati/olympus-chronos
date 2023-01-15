use std::{path::Path, collections::HashMap};
use std::io::{Read, BufReader, BufRead};

use crate::enums::command_output_type::CommandOutputType;
use crate::structs::command_output::CommandOutput;

use chrono::{Datelike, Timelike};

pub fn check_and_create_dir(dir_path: Option<&String>) -> i32 {
    match dir_path {
        Some(dir) => {
            let dir = Path::new(dir);
            if !dir.exists() || (dir.exists() && dir.is_file()) {
                println!("Directory '{}' does not exist, create it...", dir.display());
                if let Err(e) = std::fs::create_dir(dir) {
                    eprintln!("Failed to create '{}' directory: {}", dir.display(), e);
                    return 4;
                }
                println!("Directory '{}' is created!", dir.display());    
            }
        },
        None => {
            eprintln!("Property 'timer.all_dir' is not specified in config");
            return 4;
        }
    }

    return 0;
}

pub fn read_conf_files(dir_path: &String) -> Vec<HashMap<String, String>> {
    let mut configs: Vec<HashMap<String, String>> = Vec::new();

    let dir_path = Path::new(dir_path);
    for file in std::fs::read_dir(dir_path).unwrap() {
        let file = file.unwrap().path();
        if file.is_file() {
            let name = format!("{}", file.display());
            if name.ends_with(".conf") {
                let temp1 = name.split("/").collect::<Vec<&str>>();
                let temp1 = temp1.last().unwrap();
                let mut temp2 = temp1.split(".").collect::<Vec<&str>>();
                temp2.remove(temp2.len() - 1);
                let id = temp2.join(".");

                let mut file_conf = read_conf_file(name.as_str());
                match &mut file_conf {
                    Ok(conf) => {
                        conf.insert(String::from("id"), id);
                        configs.push(conf.clone());
                    }
                    Err(e) => eprintln!("{}", e),
                }
            }
        }
    }

    return configs;
}

pub fn read_conf_file(path: &str) -> Result<HashMap<String, String>, String> {
    verbose_println!("read_conf_files: Reading '{}'", path);
    let file_conf = match onlyati_config::read_config(path) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("Failed to read config '{}': {}", path, e));
        }
    };
    verbose_println!("read_conf_files: Config from '{}': {:?}", path, file_conf);

    return Ok(file_conf);
}

pub fn read_buffer<T: Read>(reader: &mut BufReader<T>, out_type: CommandOutputType) -> Vec<CommandOutput> {
    let mut line = String::new();
    let mut messages: Vec<CommandOutput> = Vec::new();

    while let Ok(size) = reader.read_line(&mut line) {
        if size == 0 {
            break;
        }

        messages.push(CommandOutput { 
            time: time_is_now(), 
            text: line.replace("\n", ""),
            r#type: out_type 
        });

        line = String::new();
    }

    return messages;
}

fn time_is_now() -> String {
    let now = chrono::Local::now();
    return format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
}
