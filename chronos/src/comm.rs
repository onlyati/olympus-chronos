use std::fs;

use crate::TIMERS_GLOB;

pub fn help(_options: Vec<String>) -> Result<String, String> {
    let mut response = String::new();

    response += "Possible Chronos commands:\n";
    response += "List active timers:       list active\n";
    response += "List started timers:      list startup\n";
    response += "List all timer config:    list all\n";
    response += "Purge timer:              purge <timer-id>\n";
    response += "Add timer:                add <timer-id>\n";

    return Ok(response);
}

pub fn list(options: Vec<String>) -> Result<String, String> {
    if options.len() == 0 {
        return Err(String::from("You must specifiy what you want list: active, startup or all. See help for more info"));
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List all timers from global timer vector                                                  */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("active") {
        let mut response = String::new();

        let timer_mut = TIMERS_GLOB.get();
        match timer_mut {
            Some(_) => {
                let timers = timer_mut.unwrap().lock().unwrap();
                for timer in timers.iter() {
                    let cmd = format!("{} {} {:?}", timer.command.user, timer.command.bin, timer.command.args);
                    response += format!("{}, {}, {:?}, {}, {}\n", timer.name, timer.kind, timer.interval, timer.next_hit, cmd).as_str();
                }
            },
            None => return Err(String::from("Internal error during clamiming global timer list")),
        }

        return Ok(response);
    }

    /*-------------------------------------------------------------------------------------------*/
    /* List files from startup_timers direcotry                                                  */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("startup") {
        let paths = match fs::read_dir("startup_timers") {
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

    /*-------------------------------------------------------------------------------------------*/
    /* List files from all_timers directory                                                      */
    /*-------------------------------------------------------------------------------------------*/
    if options[0] == String::from("all") {
        let paths = match fs::read_dir("all_timers") {
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