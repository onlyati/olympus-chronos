use std::env;
use std::fs;
use std::mem::size_of;
use std::path::Path;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;
use std::process::exit;

use once_cell::sync::OnceCell;

mod types;
use crate::types::Timer;
use crate::types::TimerType;

static TIMERS_GLOB: OnceCell<Mutex<Vec<Timer>>> = OnceCell::new();

mod files;
mod process;
mod hermes;
mod comm;

fn main() {
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
        println!("Chronos directory must be defined!");
        return;
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Set working directory                                                                     */
    /* =====================                                                                     */
    /*                                                                                           */
    /* To make file functions more transparent, work directory is changed to there where every   */
    /* files can be located.                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    let mut dev_mode: bool = false;
    if args[1] == "--dev" {
        dev_mode = true;
    }

    let work_dir = Path::new(&args[args.len() - 1]);
    if !work_dir.exists() {
        println!("Working directory does not exist: {}", work_dir.display());
        exit(1);
    }

    if let Err(e) = env::set_current_dir(work_dir) {
        println!("Work directory change to {} has failed: {:?}", work_dir.display(), e);
        exit(1);
    }


    /*-------------------------------------------------------------------------------------------*/
    /* Read the configuration from main.conf member                                              */
    /*-------------------------------------------------------------------------------------------*/
    let config: HashMap<String, String> = match onlyati_config::read_config("main.conf") {
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
    let socket = if dev_mode {
        Path::new("/tmp/chronos-dev.sock")
    } else {
        Path::new("/tmp/chronos.sock")
    };


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
                let timer_mut = TIMERS_GLOB.get();
                match timer_mut {
                    Some(_) => {
                        let mut timers = timer_mut.unwrap().lock().unwrap();
                        let mut purged_timers: Vec<usize> = Vec::with_capacity(10 * size_of::<usize>());
                        let mut index: usize = 0;

                        for timer in timers.iter_mut() {
                            if timer.next_hit == s {
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
                                    let file_path = format!("{}/startup_timers/{}.conf", config.get("timer_location").unwrap(), timer.name);
                                    match fs::remove_file(file_path) {
                                        Ok(_) => println!("OneShot timer ({}) is fired, so it is disabled", timer.name),
                                        Err(e) => println!("OneShot timer ({}) is fired, but link remove failed: {:?}", timer.name, e),
                                    }
                                }
                            }
                            index += 1;
                        }

                        for i in purged_timers {
                            timers.remove(i);
                        }
                    },
                    None => println!("Failed to retreive timers list"),
                }
            },
            Err(_) => println!("Error during receive"),
        }
    }
}

