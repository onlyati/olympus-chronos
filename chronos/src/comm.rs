use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::mem::size_of;
use chrono::prelude::*;
use chrono::Duration;

use crate::types::Timer;
use crate::process;
use crate::types::TimerType;

use crate::TIMERS;

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
    response += "Enable startup timer:                startup enable <timer-id>\n";
    response += "Disable startup timer:               startup disable <timer-id>\n";

    return Ok(response);
}

/// Function to manipulate default timers
/// 
/// Via this function, users can define which timers must be start aftert Chronos startup
pub fn startup(options: Vec<String>) -> Result<String, String> {
    if options.len() < 2 {
        return Err(String::from("Invalid syntax"));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Enable startup timer (create symlink)                                                     */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("enable") {
        let path = format!("all_timers/{}.conf", options[1]);
        let path = Path::new(&path);

        if !path.exists() {
            return Err(format!("Timer ({}) does not exist in all_timers\n", options[1]));
        }

        let path = format!("../all_timers/{}.conf", options[1]);
        let path = Path::new(&path);

        let symlink = format!("startup_timers/{}.conf", options[1]);
        let symlink = Path::new(&symlink);

        if symlink.exists() {
            return Err(format!("Timer ({}) is already enabled\n", options[1]));
        }

        match std::os::unix::fs::symlink(path, symlink) {
            Ok(_) => return Ok(format!("Timer ({}) will be added automatically after startup\n", options[1])),
            Err(e) => return Err(format!("Failed to enable timer startup for {}: {:?}\n", options[1], e)),
        }
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Disable startup timer (remove symlink)                                                    */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("disable") {
        let path = format!("startup_timers/{}.conf", options[1]);
        let path = Path::new(&path);

        if !path.exists() {
            return Err(format!("Timer ({}) cannot be found in startup timers\n", options[1]));
        }

        match fs::remove_file(path) {
            Ok(_) => return Ok(format!("Timer ({}) will not be added automatically after startup\n", options[1])),
            Err(e) => return Err(format!("Failed to disable timer startup for {}: {:?}\n", options[1], e)),
        }
    }

    return Err(String::from("Invalid enable request\n"));
}

/// Add new timer
/// 
/// This function add new timer from all_timers directory
pub fn add(options: Vec<String>) -> Result<String, String> {
    if options.len() == 0 {
        return Err(String::from("Timer ID is missing\n"));
    }

    let path = format!("all_timers/{}.conf", options[0]);

    let timer = match process::process_timer_file(&path) {
        Some(t) => t,
        None =>  return Err(format!("Error in timer config, see Chronos log for details")),
    };

    let mut timers = TIMERS.lock().unwrap();

    let mut found: bool = false;
    for timer in timers.iter() {
        if timer.name == options[0] {
            found = true;
            break;
        }
    }

    if !found {
        timers.push(timer);
        return Ok(format!("Timer ({}) has been added\n", options[0]));
    } else {
        return Err(format!("Timer ({}) is already exist\n", options[0]));
    }
}

/// Purge timer
/// 
/// This function is called if purge command is receive via socket
pub fn purge(options: Vec<String>) -> Result<String, String> {
    if options.len() == 0 {
        return Err(String::from("Timer ID is missing"));
    }

    let mut timers = TIMERS.lock().unwrap();
    let mut rem_index: Option<usize> = None;
    let mut index = 0;
    for timer in timers.iter() {
        if timer.name == options[0] {
            rem_index = Some(index);
            break;
        }
        index += 1;
    }
    
    if let Some(i) = rem_index {
        if i < timers.len() {
            timers.remove(i);
            return Ok(format!("Timer ({}) has been purged\n", options[0]));
        } else {
            return Err(format!("Internal error occured: length of timers: {}, purge index: {}\n", timers.len(), i));
        }
    }

    return Err(String::from("Invalid purge request"));
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
        let timers: Vec<Timer> = {
            let timers = TIMERS.lock().unwrap();
            let mut temp: Vec<Timer> = Vec::with_capacity(timers.len() * size_of::<Timer>());
            for timer in timers.iter() {
                temp.push(timer.clone());
            }
            temp
        };

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
fn print_timers(mut timers: Vec<Timer>, need_next_hit: bool) -> String {
    let mut max_len_name: usize = 0;
    let mut max_len_int: usize = 0;
    let mut max_len_user: usize = 0;

    let sort = |k: &Timer| -> u64 {
        let diff = day_difference(&k.days);
        if diff != 0 {
            return k.next_hit + 86400 * diff as u64;
        }

        if k.kind == TimerType::At && k.next_hit < Local::now().num_seconds_from_midnight() as u64 {
            return k.next_hit + 86400;

        }
        return k.next_hit;
    };

    timers.sort_unstable_by_key(sort);

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

    if max_len_int < "Not applicable".len() {
        max_len_int = "Not applicable".len();
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

        let interval = if timer.kind == TimerType::At {
            String::from("Not applicable")
        } else {
            format!("{:?}", timer.interval)
        };
        
        if need_next_hit {
            let day_diff = day_difference(&timer.days);

            let next_hit: String;
            if timer.next_hit < time_now {
                // It is on tomorrow
                let now = {
                    if day_diff == 0 {
                        Local::now() + Duration::days(1)
                    }
                    else {
                        Local::now() + Duration::days(day_diff)
                    }
                };

                let hours = timer.next_hit / 60 / 60;
                let minutes = (timer.next_hit - hours * 60 * 60) / 60;
                let seconds = timer.next_hit - hours * 60 * 60 - minutes * 60;
                
                if day_diff == 0 || timer.kind == TimerType::At {
                    next_hit = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), hours, minutes, seconds);
                }
                else {
                    next_hit = format!("{}-{:02}-{:02} 00:00:00", now.year(), now.month(), now.day());
                }                
            }
            else {
                // It is on today
                let now = {
                    if day_diff == 0 {
                        Local::now()
                    }
                    else {
                        Local::now() + Duration::days(day_diff)
                    }
                };

                let hours = timer.next_hit / 60 / 60;
                let minutes = (timer.next_hit - hours * 60 * 60) / 60;
                let seconds = timer.next_hit - hours * 60 * 60 - minutes * 60;

                if day_diff == 0 || timer.kind == TimerType::At {
                    next_hit = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", now.year(), now.month(), now.day(), hours, minutes, seconds);
                }
                else {
                    next_hit = format!("{}-{:02}-{:02} 00:00:00", now.year(), now.month(), now.day());
                }                
            }

            response += format!("{:max_len_name$} | {:7} | {:max_len_int$} | {:20} | {:max_len_user$} | {}\n", timer.name, format!("{}", timer.kind), interval, next_hit, timer.command.user, cmd).as_str();
        } else {
            response += format!("{:max_len_name$} | {:7} | {:max_len_int$} | {:max_len_user$} | {}\n", timer.name, format!("{}", timer.kind), interval, timer.command.user, cmd).as_str();
        }
    }

    return response;
}

fn day_difference(days: &Vec<bool>) -> i64 {
    let mut day_map: HashMap<Weekday, usize> = HashMap::new();
    day_map.insert(Weekday::Mon, 0);
    day_map.insert(Weekday::Tue, 1);
    day_map.insert(Weekday::Wed, 2);
    day_map.insert(Weekday::Thu, 3);
    day_map.insert(Weekday::Fri, 4);
    day_map.insert(Weekday::Sat, 5);
    day_map.insert(Weekday::Sun, 6);

    let now = Local::now();
    let now = now.weekday();

    let today = day_map.get(&now).unwrap();

    if days[*today] {
        return 0;
    }

    let mut next = {
        let mut next_day: usize = 99;
        for day in *today..7 {
            if days[day] == true {
                next_day = day;
                break;
            }
        }
        next_day
    };

    if next == 99 {
        next = {
            let mut next_day: usize = 99;
            for day in 0..*today {
                if days[day] == true {
                    next_day = day;
                    break;
                }
            }
            next_day
        };
    }

    let mut day_diff: i64 = 0;
     
    if next > *today {
        let temp = next - *today;
        day_diff = temp as i64;
    }

    if next < *today {
        let temp = 7 - *today + next;
        day_diff = temp as i64;
    }

    return day_diff;
}