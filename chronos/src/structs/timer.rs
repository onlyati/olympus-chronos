use std::collections::HashMap; 
use std::process::{Command, Stdio};
use std::io::BufReader;

use chrono::{Datelike, NaiveTime, Timelike, Local};
use tokio::time::Duration;

use crate::enums::timer_types::TimerType;
use crate::enums::command_output_type::CommandOutputType;
use crate::structs::command_output::CommandOutput;

/// Timer struct that store data about timer:
/// - id: indentifier of timer, must be unique
/// - type: type of timer as `TimerType` enum
/// - interval: interval period for timer
/// - command: what command timer has to be executed
/// - next_hit: when timer can run next time, seconds since UNIX_EPOCH
/// - days: which day timer can run, 'X' mean run and '_' mean don't run
#[derive(Clone)]
pub struct Timer {
    pub id: String,
    pub r#type: TimerType,
    pub interval: Duration,
    pub command: Vec<String>,
    pub next_hit: u64,
    pub days: Vec<char>,
    pub dynamic: bool,
}

impl Timer {
    /// Create new timer from specified informations
    pub fn new(id: String, r#type: TimerType, interval: Duration, command: Vec<String>, days: Vec<char>, dynamic: bool) -> Self {
        let mut timer = Timer {
            id,
            r#type,
            interval,
            command,
            next_hit: 0,
            days,
            dynamic,
        };

        timer.calculate_next_hit();

        return timer;
    }

    /// Parse time from timer config file
    /// 
    /// First validate content of config, then when it is fine create a Timer struct
    pub fn from_config(config: HashMap<String, String>) -> Result<Timer, String> {
        // Parse for id
        let id = match config.get("id") {
            Some(id) => id.clone(),
            None => return Err(String::from("Failed to fetch id")),
        };

        // Parse for type
        let r#type = match config.get("type") {
            Some(r#type) => {
                if r#type == "at" {
                    TimerType::At
                }
                else if r#type == "oneshot" {
                    TimerType::OneShot
                }
                else if r#type == "every" {
                    TimerType::Every
                }
                else {
                    return Err(String::from("Acceptable values for 'type' property: at, oneshot or every"));
                }
            }
            None => return Err(String::from("Property 'type' is not specified")),
        };

        // Parse for interval
        let interval = match config.get("interval") {
            Some(interval) => {
                let time = match NaiveTime::parse_from_str(interval, "%H:%M:%S") {
                    Ok(i) => i,
                    Err(e) => return Err(format!("Failed to parse interval: {}", e)),
                };
                tokio::time::Duration::from_secs(time.num_seconds_from_midnight().into())
            }
            None => return Err(String::from("Property 'interval' is not specified")),
        };

        // Parse for command
        let command = match config.get("command") {
            Some(command) => command.split_whitespace().map(|x| String::from(x)).collect::<Vec<String>>(),
            None => return Err(String::from("Property 'command' is not specified")),
        };

        // Parse for dayy when timer can run
        let days = match config.get("days") {
            Some(days) => {
                if days.len() != 7 {
                    return Err(String::from("Property 'days' value is wrong, it must be 7 character and can contain only 'X' or '_'"));
                }

                for c in days.chars() {
                    if c != 'X' && c != '_' {
                        return Err(String::from("Property 'days' value is wrong, it must be 7 character and can contain only 'X' or '_'"));
                    }
                }

                days.chars().collect::<Vec<char>>()
            }
            None => vec!['X', 'X', 'X', 'X', 'X', 'X', 'X']
        };

        let timer = Timer::new(id, r#type, interval, command, days, false);

        return Ok(timer);
    }

    /// Calculate when the timer should run next time
    pub fn calculate_next_hit(&mut self) {
        verbose_println!("calculate_next_hit: {}: Calculate next hit", self.id);
        verbose_println!("calculate_next_hit: {}: Timer type: {}", self.id, self.r#type);
        verbose_println!("calculate_next_hit: {}: Days: {:?}", self.id, self.days);

        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(e) => panic!("Failed for calculate time since UNIX_EPICH: {}", e),
        };
        verbose_println!("calculate_next_hit: {}: Time now: {}", self.id, now);

        let last_midnight = now - now % 86400;
        let interval = self.interval.as_secs();
        verbose_println!("calculate_next_hit: {}: Last midnight: {}", self.id, last_midnight);
        verbose_println!("calculate_next_hit: {}: Interval ins seconds: {}", self.id, interval);

        let tz_diff = Local::now().offset().utc_minus_local();


        if self.r#type == TimerType::At {
            let days_until_next = self.calculate_next_day_diff_index();
            verbose_println!("calculate_next_hit: {}: Days until next run: {}", self.id, days_until_next);

            if tz_diff < 0 {
                self.next_hit = interval + last_midnight + days_until_next * 86400 - tz_diff.abs() as u64;
            }
            else {
                self.next_hit = interval + last_midnight + days_until_next * 86400 + tz_diff.abs() as u64;
            }
            verbose_println!("calculate_next_hit: {}: Next hit: {}", self.id, self.next_hit);
        }
        else {
            let days_until_next = self.calculate_next_day_diff_index();
            verbose_println!("calculate_next_hit: {}: Days until next run: {}", self.id, days_until_next);

            if days_until_next == 0 {
                self.next_hit = interval + now;
            }
            else {
                if tz_diff < 0 {
                    self.next_hit = interval + last_midnight + days_until_next * 86400 - tz_diff.abs() as u64;
                }
                else {
                    self.next_hit = interval + last_midnight + days_until_next * 86400 + tz_diff.abs() as u64;
                }
            }
            verbose_println!("calculate_next_hit: {}: Next hit: {}", self.id, self.next_hit);
        }
    }

    /// Internally used by `calculate_next_hit`. This function checks which is that day when timer can run again
    fn calculate_next_day_diff_index(&self) -> u64 {
        let mut difference: u64 = 0;

        let today_index = chrono::Local::now().weekday().num_days_from_monday() as usize;
        verbose_println!("calculate_next_day_diff_index: {}: Today index is {}", self.id, today_index);

        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(e) => panic!("Failed for calculate time since UNIX_EPICH: {}", e),
        };

        let secs_since_midnight = now % 86400;

        // Check that timer can be scheduled today
        let today_next_hit_theory = if self.r#type == TimerType::At {
            self.interval.as_secs()
        }
        else {
            self.interval.as_secs() + secs_since_midnight
        };

        verbose_println!("calculcate_next_day_diff_index: {}: Today next theoretically hit: {}", self.id, today_next_hit_theory);
        verbose_println!("calculcate_next_day_diff_index: {}: Seconds since midnight: {}", self.id, secs_since_midnight);

        if self.r#type != TimerType::At && today_next_hit_theory <= 86400 && today_next_hit_theory > secs_since_midnight && self.days[today_index] == 'X' {
            return difference;
        }

        if self.r#type == TimerType::At && today_next_hit_theory > secs_since_midnight && self.days[today_index] == 'X' {
            return difference;
        }

        // Timer cannot be schedule today, so find next good day
        let next_index = today_index + 1 % (self.days.len() - 1);
        difference += 1;
        verbose_println!("calculcate_next_day_diff_index: {}: Check available day from: {}", self.id, next_index);
        

        for i in next_index..self.days.len() {
            verbose_println!("Checking [{}] -> {}; Current difference: {}", i, self.days[i], difference);
            if self.days[i] == 'X' {
                return difference;
            }
            difference += 1;
        }

        for i in 0..next_index {
            verbose_println!("Checking [{}] -> {}; Current difference: {}", i, self.days[i], difference);
            if self.days[i] == 'X' {
                return difference;
            }
            difference += 1;
        }

        verbose_println!("calculcate_next_day_diff_index: {}: difference until next day: {}", self.id, difference);
        return difference;
    }

    /// Check that timer should run, depend that what time is it now
    pub fn should_run(&self, now: u64) -> bool {
        if self.next_hit <= now {
            return true;
        }
        return false;
    }

    /// Execute command which belong to timer
    pub async fn execute(&self) -> Option<(Vec<CommandOutput>, i32)> {
        if self.command.len() == 0 {
            verbose_println!("execute: {}: Command vector is empty", self.id);
            return None;
        }

        let mut cmd = Command::new("/usr/bin/bash");
        cmd.arg("-c");

        let arg = self.command[..].join(" ");
        verbose_println!("execute: {}: Command argument: /usr/bin/bash -c \"{}\"", self.id, arg);
        cmd.arg(arg);

        let mut child = cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdout: Vec<CommandOutput> = Vec::new();
        let mut stderr: Vec<CommandOutput> = Vec::new();

        std::thread::scope(|spawner| {
            spawner.spawn(|| {
                let pipe = child.stdout.as_mut().unwrap();
                stdout = crate::services::file::read_buffer(&mut BufReader::new(pipe), CommandOutputType::Info);
            });
            spawner.spawn(|| {
                let pipe = child.stderr.as_mut().unwrap();
                stderr = crate::services::file::read_buffer(&mut BufReader::new(pipe), CommandOutputType::Error);
            });

        });

        stdout.append(&mut stderr);
        stdout.sort_by(|a, b| a.time.cmp(&b.time));

        let status = child.wait();
        verbose_println!("execute: {}: Command end status: {:?}", self.id, status);
        let status = match status {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to wait for child: {}", e);
                return Some((stdout, -999));
            }
        };
        
        return Some((stdout, status.code().unwrap()));
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id {
            return true;
        }
        return false;
    }
}

impl Eq for Timer {}