use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::path::Path;
use std::{fs, io};
use std::io::{Write, Read, BufReader};
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::mem::size_of;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::types::Command;
use crate::types::Timer;
use crate::types::TimerType;
use crate::comm;

use chrono::Datelike;
use chrono::Timelike;

use crate::TIMERS_GLOB;

/// Execute command
/// 
/// This function executed the specified program on a different thread.
/// If command program is not specified, it returns with Err.
/// 
/// # Return
/// 
/// Return with `Result<(), String>`.
pub fn exec_command(command: Command, id: String) -> Result<(), String> {
    if command.bin.is_empty() {
        return Err(String::from("Command is not defined"));
    }

    std::thread::spawn(move || {
        let log_file = format!("logs/{}.log", id);
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

/// Process content of active timers
/// 
/// This function reads the specified time and try to parse it for a `Timer` struct. In case of any failure
/// function returns with `None`, else return with `Some(Timer)`.
pub fn process_timer_file(file_path: &String) -> Option<Timer> {
    // Get timer ID
    let file_name: &str = match file_path.split("/").collect::<Vec<&str>>().last() {
        Some(v) => v,
        None => return None,
    };
    
    let timer_id: &str = match file_name.split(".conf").collect::<Vec<&str>>().first() {
        Some(v) => v,
        None => return None,
    };
    
    // Read file as config
    let timer_info: HashMap<String, String> = match onlyati_config::read_config(file_path.as_str()) {
        Ok(r) => r,
        Err(e) => {
            println!("Error during config reading: {}", e);
            return None;
        }
    };

    // Get timer type
    let timer_type: TimerType = match timer_info.get("type") {
        Some(ref v) if v.as_str() == "every" => TimerType::Every,
        Some(ref v) if v.as_str() == "oneshot" => TimerType::OneShot,
        Some(_) => {
            println!("Invalid timer type for {}", file_path);
            return None;
        },
        None => {
            println!("Missing timer type for {}", file_path);
            return None;
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
                return None;
            }

            let multipliers: Vec<u64> = vec![60*60, 60, 1];
            for i in 0..times.len() {
                match times[i].parse::<u64>() {
                    Ok(r) => {
                        if r > 59 {
                            println!("HH:MM:SS values must be between 0 and 59: {}", file_path);
                            return None;
                        }
                        seconds += r * multipliers[i];
                    },
                    _ => {
                        println!("Invalid interval value for {}", file_path);
                        println!("Interval format must follow: HH:MM:SS format!");
                        return None;
                    }
                }
            }

            Duration::new(seconds, 0)
        },
        None => {
            println!("Missing interval for {}", file_path);
            return None;
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
            return None;
        },
    };

    // Get user
    let timer_user = match timer_info.get("user") {
        Some(v) => {
            v.to_string()
        },
        None => {
            println!("Missing user for {}", file_path);
            return None;
        }
    };

    let timer_command: Command = Command::new(timer_command, timer_user);

    let v = chrono::Local::now();
    let v = v.num_seconds_from_midnight();
    let v: u64 = v.into();

    let timer_next_hit = v + timer_interval.as_secs();

    return Some(Timer::new(String::from(timer_id), timer_type, timer_interval, timer_command, timer_next_hit));
}

/// Read active timers
/// 
/// This function read the active timers from the <root_dir>/startup_timers directory. Files are technically
/// links to the <root_dir>/all_timers directory.
fn read_startup_timer() -> Vec<Timer> {
    let timer_files = fs::read_dir("startup_timers").unwrap()
        .collect::<Result<Vec<_>, io::Error>>().unwrap();

    let mut timers: Vec<Timer> = Vec::with_capacity(timer_files.len() * size_of::<Timer>());

    for file in timer_files {
        let file_path = format!("{}", file.path().display());
        match process_timer_file(&file_path) {
            Some(t) => timers.push(t),
            None => println!("Error occured during processing of: {}", file_path),
        }
    }

    return timers;
}

/// Timer handler function
/// 
/// First, this function reads all available timers from startup_timers directory and upload them to a global list.
/// After, it starts a new thread, which will have one task: watch startup_timers directory and in case of CREATE or REMOVE
/// event, modify the global timer list and Hermes data
pub fn start_timer_refresh() -> Result<(), String> {
    // Make an initial list
    let timers = read_startup_timer();
    let timer_mut = TIMERS_GLOB.set(Mutex::new(timers));
    if let Err(_) = timer_mut {
        println!("Error during mutex data bind!");
        return Err(String::from("Error during mutex data bind"));
    }
    return Ok(());
}

/// Prepare and start UNIX socket
/// 
/// This method preapre UNIX socket (create it and set permission and owners), then start liseting on a thread.
pub fn start_unix_socket(socket: &Path) -> Result<(), String> {
    // Prepare UNIX socket
    if socket.exists() {
        if let Err(e) = fs::remove_file(socket) {
            return Err(format!("Error during socket remove: {:?}", e));
        }
    }

    let listener = match UnixListener::bind(socket) {
        Ok(l) => l,
        Err(e) => {
            return Err(format!("Error during socket preparation: {:?}", e));
        },
    };

    let mut permission = fs::metadata(socket).unwrap().permissions();
    permission.set_mode(0o775);
    if let Err(e) = fs::set_permissions(socket, permission) {
        return Err(format!("Error during permission change: {:?}", e));
    }

    let chown = std::process::Command::new("/usr/bin/chown")
        .arg("root:olympus")
        .arg(socket)
        .output()
        .expect("Ownership change of sockert has failed");

    if !chown.status.success() {
        std::io::stdout().write_all(&chown.stdout).unwrap();
        std::io::stderr().write_all(&chown.stderr).unwrap();
        return Err(String::from("Error during ownership change"));
    }

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        listen_socket(stream);
                    });
                },
                Err(e) => println!("Error occured during listening: {:?}", e),
            }
        }
    });

    return Ok(());
}

/// This function is called by UNIX socket listener thread to handle a connection
fn listen_socket(mut stream: UnixStream) {
    let buffer = BufReader::new(&stream);

    let mut length_u8: Vec<u8> = Vec::with_capacity(5 * size_of::<usize>());   // Store bytes while readin, itis the message length
    let mut length: usize = 0;                                                 // This will be the parsed lenght from length_u8

    let mut msg_u8: Vec<u8> = Vec::new();                                      // Store message bytes

    let mut index = 0;                                                  // Index and read_msg are some variable for parsing incoming message
    let mut read_msg: bool = false;

    /*-------------------------------------------------------------------------------------------*/
    /* Read message from the buffer and parse it accordingly                                     */
    /*-------------------------------------------------------------------------------------------*/
    for byte in buffer.bytes() {
        match byte {
            Ok(b) => {
                /* It was the first space, first word must be a number which is the length of the subsequent message */
                if b == b' ' && !read_msg {
                    let msg_len_t = String::from_utf8(length_u8.clone()).unwrap();
                    length = match msg_len_t.parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            let _ = stream.write_all(b"First word must be a number which is the lenght of message\n");
                            return;
                        }
                    };
                    msg_u8 = Vec::with_capacity(length);
                    read_msg = true;
                    continue;
                }

                // Set timeout to avoid infinite waiting on the stream
                stream.set_read_timeout(Some(Duration::new(0, 250))).unwrap();

                /* Read from buffer */
                if read_msg {
                    msg_u8.push(b);
                    index += 1;
                    if index == length {
                        break;
                    }
                    continue;
                }
                else {
                    length_u8.push(b);
                    continue;
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let _ = stream.write_all(b"ERROR: Request is not complete within time\n");
                return;
            },
            Err(e) => {
                println!("Unexpected error: {:?}", e);
                let _ = stream.write_all(b"ERROR: Internal server error during stream reading\n");
                return;
            },
        }
    }

    if !read_msg {
        /* This happen when the first world was not a number and new line was incoming */
        let _ = stream.write_all(b"First word must be a number which is the lenght of message\n");
        return;
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Readin from buffer was okay, now parse it then call the command coordinator and return    */
    /* with the answer of the command                                                            */
    /*-------------------------------------------------------------------------------------------*/
    let command = String::from_utf8(msg_u8).unwrap();

    let mut verb: String = String::from("");
    let mut options: Vec<String> = Vec::with_capacity(5 * size_of::<String>());

    let mut index = 0;
    for word in command.split_whitespace() {
        if index == 0 {
            verb = String::from(word);
        }
        else {
            options.push(String::from(word));
        }
        index += 1;
    }

    match command_coordinator(verb, options) {
        Ok(s) => {
            let _ = stream.write_all(s.as_bytes());
        },
        Err(e) => {
            let error_msg = format!("ERROR: {}", e);
            let _ = stream.write_all(error_msg.as_bytes());
        }
    }
}

fn command_coordinator(verb: String, options: Vec<String>) -> Result<String, String> {
    let help_verb = String::from("help");
    let list_verb = String::from("list");
    let purge_veb = String::from("purge");

    if verb == purge_veb {
        return comm::purge(options);
    }

    if verb == help_verb {
        return comm::help(options);
    }

    if verb == list_verb {
        return comm::list(options);
    }

    return Err(String::from("Invalid command verb\n"));
}