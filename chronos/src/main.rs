use std::sync::{mpsc, RwLock, Mutex};
use std::collections::HashMap;
use std::process::exit;

#[macro_use]
mod macros;

mod enums;
mod structs;
mod services;

use structs::timer::Timer;

static VERSION: &str = "v.0.2.0";
static VERBOSE: RwLock<bool> = RwLock::new(true);
static TIMERS: Mutex<Vec<Timer>> = Mutex::new(Vec::new());

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
    /* Read startup timers and defined them                                                      */
    /*-------------------------------------------------------------------------------------------*/
    {
        let timer_configs = services::file::read_conf_files(config.get("timer.all_dir").unwrap());
        let mut timers = TIMERS.lock().unwrap();
        for config in timer_configs {
            match Timer::from_config(config) {
                Ok(timer) => timers.push(timer),
                Err(e) => eprintln!("Failed to parse timer: {}", e),
            };            
        }
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
                    panic!("Failed to allocate tokio runtime for timer trigger: {}", e);
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
                verbose_println!("Triggered second: {}", secs);
            }
            Err(e) => {
                eprintln!("Failed to receive trigger: {}", e);
                exit(8);
            }
        }
    }

}