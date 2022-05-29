use std::thread;
use std::io::Write;
use std::time::Duration;
use std::sync::mpsc::Sender;

use crate::types::Command;

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
pub fn set_every_timer(name: String, interval: Duration, sender: Sender<String>) -> Result<String, String> {
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