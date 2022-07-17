use std::fs;
use std::mem::size_of;
use chrono::prelude::*;
use chrono::Duration;

use crate::types::Timer;
use crate::process;
use crate::TIMERS_GLOB;

/// Help response
/// 
/// This function is called if help command received via socket
pub fn help(_options: Vec<String>) -> Result<String, String> {
    let mut response = String::new();

    response += "Possible Chronos commands:\n";
    response += "List active timers:                  list active\n";
    response += "List started timers:                 list startup\n";
    response += "List details of started timers:      list startup expanded\n";
    response += "List all timer config:               list all\n";
    response += "List details of all timer config:    list all expanded\n";
    response += "Purge timer:                         purge <timer-id>\n";
    response += "Add timer:                           add <timer-id>\n";

    return Ok(response);
}

/// List response
/// 
/// This function is called if list command is receive via socket
pub fn list(options: Vec<String>) -> Result<String, String> {
    if options.len() == 0 {
        return Err(String::from("You must specifiy what you want list: active, startup or all. See help for more info"));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List all timers from global timer vector                                                  */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("active") {
        /*---------------------------------------------------------------------------------------*/
        /* Collect information from global shared list, and process later                        */
        /* So Mutex is kept until copy not until end of process                                  */
        /*---------------------------------------------------------------------------------------*/
        let mut timers: Vec<Timer> = Vec::new();

        {
            let timer_mut = TIMERS_GLOB.get();
            let timer_temp: Vec<Timer> = match timer_mut {
                Some(_) => {
                    let timers = timer_mut.unwrap().lock().unwrap();
                    let mut temp: Vec<Timer> = Vec::with_capacity(timers.len() * size_of::<Timer>());
                    for timer in timers.iter() {
                        temp.push(timer.clone());
                    }
                    temp
                },
                None => return Err(String::from("Internal error during clamiming global timer list")),
            };
            timers = timer_temp;
        }

        /*---------------------------------------------------------------------------------------*/
        /* Format the output                                                                     */
        /*---------------------------------------------------------------------------------------*/
        let response = print_timers(timers, true);

        return Ok(response);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List timers from file on exapnded way with more details                                   */
    /*-------------------------------------------------------------------------------------------*/
    if (options[0] == String::from("startup") || options[0] == String::from("all")) && options.len() >= 2 {
        if options[1] == String::from("expanded") {
            let paths = match fs::read_dir(format!("{}_timers", options[0])) {
                Ok(p) => p,
                Err(e) => return Err(format!("Internal error during listing files: {:?}\n", e)),
            };

            let mut timers: Vec<Timer> = Vec::new();
            for path in paths {
                let path = match path {
                    Ok(p) => p,
                    Err(e) => return Err(format!("Internal error occured: {:?}", e)),
                };

                let path = format!("{}", path.path().display());
                if let Some(timer) = process::process_timer_file(&path) {
                    timers.push(timer);
                }
            }

            let response = print_timers(timers, false);
            return Ok(response);
        }
        else {
            return Err(format!("Invalid syntax at '{}'\n", options[1]));
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List files from *_timers directory                                                        */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("startup") || options[0] == String::from("all") {
        let paths = match fs::read_dir(format!("{}_timers", options[0])) {
            Ok(p) => p,
            Err(e) => return Err(format!("Internal error during listing files: {:?}\n", e)),
        };

        let mut response = String::new();
        for path in paths {
            let path = match path {
                Ok(p) => p,
                Err(e) => return Err(format!("Internal error during directory scan: {:?}\n", e)),
            };

            let path = format!("{}", path.path().display());
            let quals: Vec<&str> = path.split("/").collect();
            let quals: Vec<&str> = quals[1].split(".").collect();

            let mut file_name = String::new();
            if quals[quals.len() - 1] == "conf" {
                for i in 0..quals.len() - 1 {
                    file_name += quals[i];

                    if i != quals.len() - 2 {
                        file_name += ".";
                    }
                }
            }

            response += format!("{}\n", file_name).as_str();
        }
        return Ok(response);
    }

    return Err(format!("Specified parameter is invalid: {}\n", options[0]));
}

/// Format timer vector then print it
fn print_timers(timers: Vec<Timer>, need_next_hit: bool) -> String {
    let mut max_len_name: usize = 0;
    let mut max_len_int: usize = 0;
    let mut max_len_user: usize = 0;

    // Calculate the max length of fields
    for timer in &timers {
        if timer.name.len() > max_len_name {
            max_len_name = timer.name.len();
        }

        let temp_dur = format!("{:?}", timer.interval);
        if temp_dur.len() > max_len_int {
            max_len_int = temp_dur.len();
        }

        if timer.command.user.len() > max_len_user {
            max_len_user = timer.command.user.len();
        }
    }

    if max_len_int < "Interval".len() {
        max_len_int = "Interval".len();
    }

    let mut response = String::new();
    if need_next_hit {
        response += format!("{:max_len_name$} | {:7} | {:max_len_int$} | {:20} | {:max_len_user$} | {}\n", "Name", "Kind", "Interval", "Next hit", "User", "Command").as_str();
    } else {
        response += format!("{:max_len_name$} | {:7} | {:max_len_int$} | {:max_len_user$} | {}\n", "Name", "Kind", "Interval", "User", "Command").as_str();
    }

    // Fill up the output into response
    for timer in timers {
        let mut cmd = String::new();
        for i in 2..timer.command.args.len() {
            cmd += &timer.command.args[i][..];
            cmd += " ";
        }

        let time_now = Local::now();
        let time_now = time_now.num_seconds_from_midnight();
        let time_now: u64 = time_now.into();
        
        if need_next_hit {
            let mut next_hit = String::new();
            if timer.next_hit < time_now {
                // It is on tomorrow
                let now = Local::now() + Duration::days(1);

                let hours = timer.next_hit / 60 / 60;
                let minutes = timer.next_hit - hours * 60 * 60;
                let seconds = timer.next_hit - hours * 60 * 60 - minutes * 60;                
                
                next_hit = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), hours, minutes, seconds);
            }
            else {
                // It is on today
                let now = Local::now();

                let hours = timer.next_hit / 60 / 60;
                let minutes = (timer.next_hit - hours * 60 * 60) / 60;
                let seconds = timer.next_hit - hours * 60 * 60 - minutes * 60;

                next_hit = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), hours, minutes, seconds);
            }

            response += format!("{:max_len_name$} | {:7} | {:max_len_int$?} | {:20} | {:max_len_user$} | {}\n", timer.name, format!("{}", timer.kind), timer.interval, next_hit, timer.command.user, cmd).as_str();
        } else {
            response += format!("{:max_len_name$} | {:7} | {:max_len_int$?} | {:max_len_user$} | {}\n", timer.name, format!("{}", timer.kind), timer.interval, timer.command.user, cmd).as_str();
        }
    }

    return response;
}