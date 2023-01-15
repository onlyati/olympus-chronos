use std::sync::{mpsc, mpsc::{Sender, Receiver}};
use std::collections::HashMap;
use std::process::exit;

mod enums;
mod services;

static VERSION: &str = "v.0.2.0";

fn main() {
    println!("Version {} is starting...", VERSION);
    
    /*-------------------------------------------------------------------------------------------*/
    /* Read and parse config parameters                                                          */
    /*-------------------------------------------------------------------------------------------*/
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Config file is not specified as parameter");
        exit(2);
    }

    let config: HashMap<String, String> = match onlyati_config::read_config(args[1].as_str()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read '{}' config: {}", args[1], e);
            exit(2);
        }
    };

    println!("Configuration:");
    for (property, value) in &config {
        println!("- {} -> {}", property, value);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Check that directories are exist                                                          */
    /*-------------------------------------------------------------------------------------------*/
    if services::file::check_and_create_dir(config.get("timer.all_dir")) != 0 {
        exit(4);
    }

    if services::file::check_and_create_dir(config.get("timer.startup_dir")) != 0 {
        exit(4);
    }

    if services::file::check_and_create_dir(config.get("timer.log_dir")) != 0 {
        exit(4);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Start a thread which send trigger in every 1 second                                       */
    /*-------------------------------------------------------------------------------------------*/
    let (tx, rx) = mpsc::channel::<u64>();
    std::thread::spawn(move || {
        let rt =  match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build() {
                Ok(rt) => rt,
                Err(e) => {
                    panic!("Failed to allocate tokio runtime for timer trigger");
                }
            };
        rt.block_on(async move {
            services::timing::start_trigger_timer(tx).await;
        });
    });

    /*-------------------------------------------------------------------------------------------*/
    /* Start main part of the program, which executes timers accordingly                         */
    /*-------------------------------------------------------------------------------------------*/
    loop {
        match rx.recv() {
            Ok(secs) => {
                println!("Seconds: {}", secs);
            }
            Err(e) => {
                eprintln!("Failed to receive trigger: {}", e);
                exit(8);
            }
        }
    }

}