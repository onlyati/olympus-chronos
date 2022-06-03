use std::env;
use std::{fs, io};
use std::mem::size_of;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use chrono::Timelike;

mod types;
use crate::types::Command;
use crate::types::Timer;
use crate::types::TimerType;

mod files;
mod process;

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

    let config: HashMap<String, String> = match onlyati_config::read_config(args[1].as_str()) {
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

    /*-------------------------------------------------------------------------------------------*/
    /* Read active timers                                                                        */
    /* ==================                                                                        */
    /*                                                                                           */
    /* Read active timers from active_timers directory. This directory contains links which are  */
    /* point to the file in all_timers directory.                                                */
    /*                                                                                           */
    /* If any timer file parse has failed, then program makes a warning, but does not exit.      */
    /*-------------------------------------------------------------------------------------------*/
    let timer_path = format!("{}/active_timers", config.get("timer_location").unwrap());
    let timer_files = fs::read_dir(timer_path.as_str()).unwrap()
        .collect::<Result<Vec<_>, io::Error>>().unwrap();

    let mut timers: Vec<Timer> = Vec::with_capacity(timer_files.len() * size_of::<Timer>());

    for file in timer_files {
        let file_path = format!("{}", file.path().display());
        
        // Get timer ID
        let file_name: &str = match file_path.split("/").collect::<Vec<&str>>().last() {
            Some(v) => v,
            None => continue,
        };
        
        let timer_id: &str = match file_name.split(".conf").collect::<Vec<&str>>().first() {
            Some(v) => v,
            None => continue,
        };
        
        // Read file as config
        let timer_info: HashMap<String, String> = match onlyati_config::read_config(file_path.as_str()) {
            Ok(r) => r,
            Err(e) => {
                println!("Error during config reading: {}", e);
                continue;
            }
        };

        // Get timer type
        let timer_type: TimerType = match timer_info.get("type") {
            Some(ref v) if v.as_str() == "every" => TimerType::Every,
            Some(_) => {
                println!("Invalid timer type for {}", file_path);
                continue;
            },
            None => {
                println!("Missing timer type for {}", file_path);
                continue;
            },
        };

        // Get timer interval
        let timer_interval: Duration = match timer_info.get("interval") {
            Some(v) => {
                let times = v.split(":").collect::<Vec<&str>>();
                let mut seconds = 0;

                if times.len() != 3 {
                    println!("Invalid interval value for {}", file_path);
                    println!("Interval format must follow: HH:MM:SS format!");
                    continue;
                }

                let multipliers: Vec<u64> = vec![60*60, 60, 1];
                for i in 0..times.len() {
                    match times[i].parse::<u64>() {
                        Ok(r) => seconds += r * multipliers[i],
                        _ => {
                            println!("Invalid interval value for {}", file_path);
                            println!("Interval format must follow: HH:MM:SS format!");
                            continue;
                        }
                    }
                }

                Duration::new(seconds, 0)
            },
            None => {
                println!("Missing interval for {}", file_path);
                continue;
            },
        };

        // Get command
        let timer_command: Command = match timer_info.get("command") {
            Some(v) => {
                let args = v.split_whitespace().collect::<Vec<&str>>();
                if args.len() > 1 {
                    let mut real_args: Vec<String> = Vec::with_capacity(args.len() * size_of::<String>());

                    for i in 1..args.len()  {
                        real_args.push(String::from(args[i]));
                    }

                    Command {
                        bin: String::from(args[0]),
                        args: real_args,
                    }
                }
                else {
                    Command {
                        bin: String::from(args[0]),
                        args: Vec::new(),
                    }
                }
            },
            None => {
                println!("Missing command for {}", file_path);
                continue;
            },
        };

        let v = chrono::Local::now();
        let v = v.num_seconds_from_midnight();
        let v: u64 = v.into();

        let timer_next_hit = v + timer_interval.as_secs();

        timers.push(Timer::new(String::from(timer_id), timer_type, timer_interval, timer_command, timer_next_hit));
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

            let status = hermes_del_group(v, "timer");
            println!("{:?}", status);

            let status = hermes_add_group(v, "timer");
            println!("{:?}", status);

            for timer in &timers {
                let info = format!("{}s {} {:?}", timer.interval.as_secs(), timer.command.bin, timer.command.args);
                let status = hermes_add_timer(v, timer.name.as_str(), info.as_str());
                println!("{:?}", status);
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
                for timer in &mut timers {
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
            Err(_) => println!("Error during receive"),
        }
    }
}

fn hermes_del_group(address: &str, name: &str) -> Result<String, String> {
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("DELETE /group?name={} HTTP/1.1\r\nAccept: */*\r\nContent-Length: 0\r\n", name);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}

fn hermes_add_group(address: &str, name: &str) -> Result<String, String> {
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("POST /group?name={} HTTP/1.1\r\nAccept: */*\r\nContent-Length: 0\r\n", name);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}

fn hermes_add_timer(address: &str, name: &str, content: &str) -> Result<String, String> {
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("POST /item?name={}&group=timer HTTP/1.1\r\nAccept: */*\r\nContent-Length: {}\r\n\r\n{}\r\n", name, content.len(), content);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}