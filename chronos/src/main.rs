use std::env;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;

use once_cell::sync::OnceCell;

mod types;
use crate::types::Timer;

static TIMERS_GLOB: OnceCell<Mutex<Vec<Timer>>> = OnceCell::new();

mod files;
mod process;
mod hermes;

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
        println!("Config path must be specified as parameter!");
        return;
    }

    let mut config: HashMap<String, String> = match onlyati_config::read_config(args[1].as_str()) {
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
    /* |-> active_timers                                                                         */
    /* '-> logs                                                                                  */
    /*                                                                                           */
    /* If any of them does not exist, program will try to create them. If creation is failed then*/
    /* program make an exit.                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    match config.get("timer_location") {
        Some(v) => {
            if let Err(e) = files::check_and_build_dirs(v) {
                println!("Error occured during '{}' directory creation!", e);
                return;
            }
        }
        None => {
            println!("Option 'timer_location' is not defined in config file");
            return;
        }
    }

    if let None = config.get("timer_check_interval") {
        println!("Item (timer_check_interval) cannot be found in config, default will be used: 00:00:30");
        config.insert(String::from("timer_check_interval"), String::from("00:00:30"));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Read active timers                                                                        */
    /* ==================                                                                        */
    /*                                                                                           */
    /* Read active timers from active_timers directory. This directory contains links which are  */
    /* point to the file in all_timers directory.                                                */
    /*                                                                                           */
    /* If any timer file parse has failed, then program makes a warning, but does not exit.      */
    /*-------------------------------------------------------------------------------------------*/
    match process::start_timer_refresh(config.get("timer_location").unwrap(), config.get("timer_check_interval").unwrap()) {
        Ok(_) => println!("Timers are read"),
        Err(e) => {
            println!("{}", e);
            return;
        },
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Upload timers onto Hermes                                                                 */
    /* =========================                                                                 */
    /*                                                                                           */
    /* If hermes is available upload the timers onto that on the specified port and address at   */
    /* 'hermes_address' property.                                                                */
    /*-------------------------------------------------------------------------------------------*/
    match config.get("hermes_address") {
        Some(v) => {
            println!("Update Hermes with timer data");

            std::env::set_var("CHRONOS_HERMES_ADDR", v);

            let status = hermes::hermes_del_group("timer");
            println!("{:?}", status);

            let status = hermes::hermes_add_group("timer");
            println!("{:?}", status);

            let timer_mut = TIMERS_GLOB.get();
            match timer_mut {
                Some(_) => {
                    let timers = timer_mut.unwrap().lock().unwrap();
                    for timer in timers.iter() {
                        let info = format!("{}s {} {:?}", timer.interval.as_secs(), timer.command.bin, timer.command.args);
                        let status = hermes::hermes_add_timer(timer.name.as_str(), info.as_str());
                        println!("{:?}", status);
                    }
                },
                None => {
                    println!("Failed toget timer list, cannot upload to Hermes!");
                    return;
                }
            }
        },
        None => println!("Hermes location is not specified. Updates will not be send there!"),
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
                        for timer in timers.iter_mut() {
                            if timer.next_hit == s {
                                println!("{} has expired", timer.name);
                                let _ = process::exec_command(timer.command.clone(), timer.name.clone(), config.get("timer_location").unwrap().to_string());
                                timer.next_hit = s + timer.interval.as_secs();
                                if timer.next_hit >= 86400 {
                                    timer.next_hit = timer.next_hit - 86400;
                                }
                            }
                        }
                    },
                    None => println!("Failed to retreive timers list"),
                }
            },
            Err(_) => println!("Error during receive"),
        }
    }
}

