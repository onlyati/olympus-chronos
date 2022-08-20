use std::env;
use std::mem::size_of;
use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;
use std::process::exit;

use chrono::{Local, Datelike};
use chrono::Weekday;

mod types;
use crate::types::Timer;
use crate::types::TimerType;

static TIMERS: Mutex<Vec<Timer>> = Mutex::new(Vec::new());
static HERMES_ADDR: Mutex<Option<String>> = Mutex::new(None);

mod files;
mod process;
mod comm;

fn main() {
    println!("Version 0.1.2 is starting...");

    /*-------------------------------------------------------------------------------------------*/
    /* Argument verification                                                                     */
    /* =====================                                                                     */
    /*                                                                                           */
    /* Verify that config member has been passed as argument:                                    */
    /* - If it does, try to parse it into a HashMap                                              */
    /* - Else return with error                                                                  */
    /*-------------------------------------------------------------------------------------------*/
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Chronos config must be defined!");
        return;
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Read the configuration from main.conf member                                              */
    /*-------------------------------------------------------------------------------------------*/
    let config: HashMap<String, String> = match onlyati_config::read_config(&args[1]) {
        Ok(r) => r,
        Err(e) => {
            println!("Error during config reading: {}", e);
            return;
        }
    };

    println!("Configuration:");
    for (setting, value) in &config {
        println!("{} -> {}", setting, value);
    }

    if let Some(addr) = config.get("hermes_addr") {
        let mut hermes_addr = HERMES_ADDR.lock().unwrap();
        *hermes_addr = Some(addr.clone());
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Set working directory                                                                     */
    /* =====================                                                                     */
    /*                                                                                           */
    /* To make file functions more transparent, work directory is changed to there where every   */
    /* files can be located.                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    let work_dir = Path::new(config.get("work_dir").expect("work_dir does not found config file"));
    if !work_dir.exists() {
        println!("Working directory does not exist: {}", work_dir.display());
        exit(1);
    }

    if let Err(e) = env::set_current_dir(work_dir) {
        println!("Work directory change to {} has failed: {:?}", work_dir.display(), e);
        exit(1);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Verify file structure                                                                     */
    /* =====================                                                                     */
    /*                                                                                           */
    /* Root directory is that which is passed as 'timer_location' in the config file. From this  */
    /* point file system should look:                                                            */
    /* root                                                                                      */
    /* |-> all_timers                                                                            */
    /* |-> startup_timers                                                                        */
    /* '-> logs                                                                                  */
    /*                                                                                           */
    /* If any of them does not exist, program will try to create them. If creation is failed then*/
    /* program make an exit.                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    match files::check_and_build_dirs() {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to create directories in work dir: {}", e);
            exit(1);
        },
    };

    /*-------------------------------------------------------------------------------------------*/
    /* Read active timers                                                                        */
    /* ==================                                                                        */
    /*                                                                                           */
    /* Read active timers from startup_timers directory. This directory contains links which are */
    /* point to the file in all_timers directory.                                                */
    /*                                                                                           */
    /* If any timer file parse has failed, then program makes a warning, but does not exit.      */
    /*                                                                                           */
    /* This function also starts a background process which will watch the startup_timers        */
    /* directory and remove/add timer dynamically for *.conf file changes                        */
    /*-------------------------------------------------------------------------------------------*/
    let socket = config.get("socket_name").expect("socket_name is not specified in config");
    let socket = Path::new(socket);

    match process::start_timer_refresh() {
        Ok(_) => println!("Timers are read"),
        Err(e) => {
            println!("{}", e);
            exit(1);
        },
    }

    match process::start_unix_socket(socket) {
        Ok(_) => println!("UNIX socket thread has started"),
        Err(e) => {
            println!("{}", e);
            exit(1);
        },
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Start timers                                                                              */
    /* ============                                                                              */
    /*                                                                                           */
    /* Create a Channel clone the transmitter to every single timer. Timers will run as threads  */
    /* which are sleeping until interval has expired. After interval has expired, it sends a sig-*/
    /* nal via Channel and main program starts the command belongs to timer on another thread.   */
    /*-------------------------------------------------------------------------------------------*/
    let (tx, rx): (Sender<u64>, Receiver<u64>) = mpsc::channel();

    match process::set_every_timer(tx) {
        Ok(_) => println!("Timer thread started..."),
        Err(s) => {
            println!("{}", s);
            return;
        },
    }

    loop {
        match rx.recv() {
            Ok(s) => {
                let mut timers = TIMERS.lock().unwrap();
                let mut purged_timers: Vec<usize> = Vec::with_capacity(10 * size_of::<usize>());
                let mut index: usize = 0;

                for timer in timers.iter_mut() {
                    if timer.next_hit == s && timer.days[num_of_today()] {
                        println!("{} has expired", timer.name);
                        let _ = process::exec_command(timer.command.clone(), timer.name.clone());

                        if timer.kind == TimerType::Every {
                            timer.next_hit = s + timer.interval.as_secs();
                            if timer.next_hit >= 86400 {
                                timer.next_hit = timer.next_hit - 86400;
                            }
                        }

                        if timer.kind == TimerType::OneShot {
                            purged_timers.push(index);
                        }
                    }
                    index += 1;
                }

                for i in purged_timers {
                    timers.remove(i);
                }
            },
            Err(_) => println!("Error during receive"),
        }
    }
}

fn num_of_today() -> usize {
    let now = Local::now();
    let mut day_map: HashMap<Weekday, usize> = HashMap::new();
    day_map.insert(Weekday::Mon, 0);
    day_map.insert(Weekday::Tue, 1);
    day_map.insert(Weekday::Wed, 2);
    day_map.insert(Weekday::Thu, 3);
    day_map.insert(Weekday::Fri, 4);
    day_map.insert(Weekday::Sat, 5);
    day_map.insert(Weekday::Sun, 6);

    return *day_map.get(&now.weekday()).unwrap();
}