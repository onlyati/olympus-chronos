use std::thread;
use std::env;
use std::fmt;
use std::{fs, io};
use std::mem::size_of;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use chrono::Datelike;
use chrono::Timelike;

/// Enum for type of timer
#[derive(PartialEq)]
enum TimerType {
    Every,
}

impl fmt::Display for TimerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TimerType::Every => "every",
        };
        write!(f, "{}", printable)
    }
}

/// Struct for commands
#[derive(Clone)]
struct Command {
    bin: String,
    args: Vec<String>,
}

/// Timer struct
struct Timer {
    name: String,
    kind: TimerType,
    interval: Duration,
    command: Command,
}

impl Timer {
    fn new(name: String, kind: TimerType, interval: Duration, command: Command) -> Timer {
        return Timer {
            name: name,
            kind: kind,
            interval: interval,
            command : command,
        }
    }
}

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

    let config: HashMap<String, String>;
    match onlyati_config::read_config(args[1].as_str()) {
        Ok(r) => config = r,
        Err(e) => {
            println!("Error during config reading: {}", e);
            return;
        }
    }

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
    /* --> all_timers                                                                            */
    /* --> active_timers                                                                         */
    /* --> logs                                                                                  */
    /*                                                                                           */
    /* If any of them does not exist, program will try to create them. If creation is failed then*/
    /* program make an exit.                                                                     */
    /*-------------------------------------------------------------------------------------------*/
    match config.get("timer_location") {
        Some(v) => {
            if let Err(e) = check_and_build_dirs(v) {
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

        timers.push(Timer::new(String::from(timer_id), timer_type, timer_interval, timer_command));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Start timers                                                                              */
    /* ============                                                                              */
    /*                                                                                           */
    /* Create a Channel clone the transmitter to every single timer. Timers will run as threads  */
    /* which are sleeping until interval has expired. After interval has expired, it sends a sig-*/
    /* nal via Channel and main program starts the command belongs to timer on another thread.   */
    /*-------------------------------------------------------------------------------------------*/
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    for timer in &timers {
        let temp_tx = tx.clone();
        if timer.kind == TimerType::Every {
            match set_every_timer(timer.name.clone(), timer.interval.clone(), temp_tx) {
                Ok(s) => println!("{}", s),
                Err(s) => {
                    println!("{}", s);
                    return;
                },
            }
        }
    }

    loop {
        match rx.recv() {
            Ok(s) => {
                for timer in &timers {
                    if timer.name == s {
                        println!("Timer ({}) has expired, execute command: {}", s, timer.command.bin);
                        let _ = exec_command(timer.command.clone(), timer.name.clone(), config.get("timer_location").unwrap().to_string());
                    }
                }
            },
            Err(_) => println!("Error during receive"),
        }
    }
}

/// Verify that directory structure exists
/// 
/// This program check that file sturcture exist which is requires for the program.
/// If some directory does not exist, it will try to create it.
/// 
/// # Return values
/// 
/// Return with `Result<(), String>. In case of Err, the parameter is the name of directory which had problem.
fn check_and_build_dirs(root: &str) -> Result<(), String> {
    if let Err(_) = create_dir_if_not_exist(root) {
        return Err(String::from(root));
    }

    if let Err(_) = create_dir_if_not_exist(format!("{}/all_timers", root).as_str()) {
        return Err(format!("{}/all_timers", root));
    }

    if let Err(_) = create_dir_if_not_exist(format!("{}/active_timers", root).as_str()) {
        return Err(format!("{}/active_timers", root));
    }

    if let Err(_) = create_dir_if_not_exist(format!("{}/logs", root).as_str()) {
        return Err(format!("{}/log", root));
    }

    return Ok(());
}

/// If specified directory does not exist, then try to create it.
fn create_dir_if_not_exist(path: &str) -> Result<(), ()> {
    if !std::path::Path::new(path).is_dir() {
        if let Err(_) = std::fs::create_dir_all(path) {
            return Err(());
        }
    }
    return Ok(());
}

/// Execute command
/// 
/// This function executed the specified program on a different thread.
/// If command program is not specified, it returns with Err.
/// 
/// # Return
/// 
/// Return with `Result<(), String>`.
fn exec_command(command: Command, id: String, root_dir: String) -> Result<(), String> {
    if command.bin.is_empty() {
        return Err(String::from("Command is not defined"));
    }

    std::thread::spawn(move || {
        let cmd = std::process::Command::new(command.bin).args(command.args).output().expect("failed to execute process");

        let log_file = format!("{}/logs/{}.log", root_dir, id);

        let output: String = match String::from_utf8(cmd.stdout) {
            Ok(r) => r,
            Err(_) => String::from(""),
        };

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
fn set_every_timer(name: String, interval: Duration, sender: Sender<String>) -> Result<String, String> {
    let tname = name.clone();
    std::thread::spawn(move || {
        loop {
            thread::sleep(interval);
            let tname = name.clone();
            let _ = sender.send(tname);
        }
    });

    return Ok(format!("Timer ({}) is defined!", tname));
}