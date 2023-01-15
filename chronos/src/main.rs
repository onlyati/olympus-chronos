use std::sync::{mpsc, RwLock, Mutex};
use std::collections::HashMap;
use std::process::exit;
use std::io::Write;

#[macro_use]
mod macros;

mod enums;
mod structs;
mod services;

use structs::timer::Timer;

use crate::enums::timer_types::TimerType;

static VERSION: &str = "v.0.2.0";
static VERBOSE: RwLock<bool> = RwLock::new(false);
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
    /* Get verbous output value                                                                  */
    /*-------------------------------------------------------------------------------------------*/
    if let Some(v) = config.get("defaults.verbose") {
        let mut verbose = VERBOSE.write().unwrap();
        if v == "yes" {
            *verbose = true;
        }
        else {
            *verbose = false;
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Check that directories are exist                                                          */
    /*-------------------------------------------------------------------------------------------*/
    if services::file::check_and_create_dir(config.get("timer.all_dir")) != 0 {
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
    /* Start gRPC server                                                                         */
    /*-------------------------------------------------------------------------------------------*/
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let config2 = config.clone();
    std::thread::spawn(move || {
        rt.block_on(async move {
            services::grpc::start_server(&config2).await.expect("Failed to start gRPC server");
        })
    });

    /*-------------------------------------------------------------------------------------------*/
    /* Allocate a tokio runtime and start Hermes client if required                              */
    /*-------------------------------------------------------------------------------------------*/
    let (hermes_sender, hermes_receiver) = mpsc::channel::<(String, String)>();
    if let Some(ena) = config.get("hermes.enable") {
        if ena == "yes" && config.get("hermes.grpc.address").is_some() && config.get("hermes.table").is_some() {
            println!("Corresponse properties are set to yes, so start Hermes client");
            let config2 = config.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let rt = match rt {
                    Ok(rt) => rt,
                    Err(e) => panic!("Failed to allocated runtime for Hermes client: {}", e),
                };

                

                rt.block_on(async move {
                    loop {
                        let _ = services::hermes_client::start_hermes_client(&config2, &hermes_receiver).await;
                        eprintln!("Hermes client has failed, try to restart 30 sec later");
                        tokio::time::sleep(tokio::time::Duration::new(30, 0)).await;
                    }
                })
            });
        }
    }


    /*-------------------------------------------------------------------------------------------*/
    /* Allocate runtime to run timer commands                                                    */
    /*-------------------------------------------------------------------------------------------*/
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    /*-------------------------------------------------------------------------------------------*/
    /* Start main part of the program, which executes timers accordingly                         */
    /*-------------------------------------------------------------------------------------------*/
    loop {
        match rx.recv() {
            Ok(secs) => {
                verbose_println!("Triggered second: {}", secs);
                {
                    let mut timers = TIMERS.lock().unwrap();
                    let mut remove_index_list: Vec<usize> = Vec::new();
                    let mut remove_index: usize = 0;
                    for timer in timers.iter_mut() {
                        if timer.should_run(secs) {
                            println!("Execute: {}", timer.id);
                            let timer2 = timer.clone();
                            let log_dir = config.get("timer.log_dir").unwrap().clone();
                            let hermes_sender = hermes_sender.clone();
                            rt.spawn(async move {
                                let output = timer2.execute().await;
                                let log_file = format!("{}/{}.log", log_dir, timer2.id);
                                let mut file = match std::fs::OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .append(true)
                                    .open(&log_file) {
                                        Ok(f) => f,
                                        Err(e) => {
                                            eprintln!("Failed to open file '{}' to write: {}", log_file, e);
                                            return;
                                        }
                                    };
                                
                                let output = match output {
                                    Some(o) => o,
                                    None => return,
                                };

                                for line in output.0 {
                                    writeln!(&mut file, "{} {} {}", line.time, line.r#type, line.text).unwrap();
                                }

                                if output.1 == 0 {
                                    let _ = hermes_sender.send((format!("{}", timer2.id), String::from("OK")));
                                }
                                else {
                                    let _ = hermes_sender.send((format!("{}", timer2.id), String::from("NOK")));
                                }
                            });

                            if timer.r#type == TimerType::OneShot {
                                remove_index_list.push(remove_index);
                            }
                            else {
                                timer.calculate_next_hit();
                            }
                        }
                        remove_index += 1;
                    }

                    for index in remove_index_list {
                        verbose_println!("main: {}: Type is oneshot so it purged", timers[index].id);
                        timers.remove(index);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to receive trigger: {}", e);
                exit(8);
            }
        }
    }

}