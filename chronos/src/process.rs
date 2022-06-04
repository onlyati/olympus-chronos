use std::thread;
use std::{fs, io};
use std::io::Write;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::mem::size_of;
use std::collections::HashMap;

use crate::types::Command;
use crate::types::Timer;
use crate::types::TimerType;

use chrono::Datelike;
use chrono::Timelike;

/// Execute command
/// 
/// This function executed the specified program on a different thread.
/// If command program is not specified, it returns with Err.
/// 
/// # Return
/// 
/// Return with `Result<(), String>`.
pub fn exec_command(command: Command, id: String, root_dir: String) -> Result<(), String> {
    if command.bin.is_empty() {
        return Err(String::from("Command is not defined"));
    }

    std::thread::spawn(move || {
        let log_file = format!("{}/logs/{}.log", root_dir, id);
        let mut file =  match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(log_file) {
            Ok(r) => r,
            Err(e) => {
                println!("Error during try write timer log file: {:?}", e);
                return;
            }
        };

        let now = chrono::Local::now();
        let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
        writeln!(&mut file, "{} {} Timer has expired", now, id).unwrap();

        let cmd = std::process::Command::new(command.bin).args(command.args).output().expect("failed to execute process");
        let output: String = match String::from_utf8(cmd.stdout) {
            Ok(r) => r,
            Err(_) => String::from(""),
        };

        let now = chrono::Local::now();
        let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());        
        writeln!(&mut file, "{} {} Command has run, {}", now, id, cmd.status).unwrap();

        if !output.is_empty() {
            let lines = output.lines();
            for line in lines {
                let now = chrono::Local::now();
                let now = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
                writeln!(&mut file, "{} {} Command output -> {}", now, id, line).unwrap();
            }
        }
    });

    return Ok(());
}

/// Create timer with every type
/// 
/// This function creates a new thread. This thread will sleep for the specified interval and after it,
/// it sends back a signal to main program that timer has expired.
pub fn set_every_timer(sender: Sender<u64>) -> Result<(), String> {
    let interval = Duration::from_secs(1);
    std::thread::spawn(move || {
        loop {
            thread::sleep(interval);
            let v = chrono::Local::now();
            let v = v.num_seconds_from_midnight();
            let v: u64 = v.into();
            let _ = sender.send(v);
        }
    });

    return Ok(());
}

/// Read active timers
/// 
/// This function read the active timers from the ${chronos_dir}/active_timers directory. Files are technically
/// links to the ${chronos_dir}/all_timers directory.
pub fn read_active_timer(root_dir: &String) -> Vec<Timer> {
    let timer_path = format!("{}/active_timers", root_dir);
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
        let timer_command: Vec<String> = match timer_info.get("command") {
            Some(v) => {
                let temp = v.split_whitespace().collect::<Vec<&str>>();
                let mut args: Vec<String> = Vec::new();
                for cmd in temp {
                    args.push(String::from(cmd));
                }

                args
            },
            None => {
                println!("Missing command for {}", file_path);
                continue;
            },
        };

        // Get user
        let timer_user = match timer_info.get("user") {
            Some(v) => {
                v.to_string()
            },
            None => {
                println!("Missing user for {}", file_path);
                continue;
            }
        };

        let timer_command: Command = Command::new(timer_command, timer_user);

        let v = chrono::Local::now();
        let v = v.num_seconds_from_midnight();
        let v: u64 = v.into();

        let timer_next_hit = v + timer_interval.as_secs();

        timers.push(Timer::new(String::from(timer_id), timer_type, timer_interval, timer_command, timer_next_hit));
    }

    return timers;
}