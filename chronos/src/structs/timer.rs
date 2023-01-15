use std::collections::HashMap;

use chrono::{Datelike, NaiveTime, Timelike};
use tokio::time::Duration;

use crate::enums::timer_types::TimerType;

/// Timer struct that store data about timer:
/// - id: indentifier of timer, must be unique
/// - type: type of timer as `TimerType` enum
/// - interval: interval period for timer
/// - command: what command timer has to be executed
/// - next_hit: when timer can run next time, seconds since UNIX_EPOCH
/// - days: which day timer can run, 'X' mean run and '_' mean don't run
pub struct Timer {
    pub id: String,
    pub r#type: TimerType,
    pub interval: Duration,
    pub command: Vec<String>,
    pub next_hit: u64,
    pub days: Vec<char>,
}

impl Timer {
    pub fn new(id: String, r#type: TimerType, interval: Duration, command: Vec<String>, days: Vec<char>) -> Self {
        let mut timer = Timer {
            id,
            r#type,
            interval,
            command,
            next_hit: 0,
            days
        };

        timer.calculate_next_hit();

        return timer;
    }

    pub fn from_config(config: HashMap<String, String>) -> Result<Timer, String> {
        let id = match config.get("id") {
            Some(id) => id.clone(),
            None => return Err(String::from("Failed to fetch id")),
        };

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

        let command = match config.get("command") {
            Some(command) => command.split_whitespace().map(|x| String::from(x)).collect::<Vec<String>>(),
            None => return Err(String::from("Property 'command' is not specified")),
        };

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

        let timer = Timer::new(id, r#type, interval, command, days);

        return Ok(timer);
    }

    pub fn calculate_next_hit(&mut self) {
        verbose_println!("{}: Calculate next hit", self.id);
        verbose_println!("{}: Timer type: {}", self.id, self.r#type);

        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(e) => panic!("Failed for calculate time since UNIX_EPICH: {}", e),
        };

        let last_midnight = now / 86400;
        let interval = self.interval.as_secs();
        verbose_println!("{}: Last midnight: {}", self.id, last_midnight);
        verbose_println!("{}: Interval ins seconds: {}", self.id, interval);

        if self.r#type == TimerType::At {
            let days_until_next = self.calculate_next_day_diff_index();
            verbose_println!("{}: Days until next run: {}", self.id, days_until_next);

            self.next_hit = interval + last_midnight + days_until_next * 86400;
            verbose_println!("{}: Next hit: {}", self.id, self.next_hit);
        }
        else {
            let days_until_next = self.calculate_next_day_diff_index();
            verbose_println!("{}: Days until next run: {}", self.id, days_until_next);

            if days_until_next == 0 {
                self.next_hit = interval + now;
            }
            else {
                self.next_hit = interval + last_midnight + days_until_next * 86400;
            }
            verbose_println!("{}: Next hit: {}", self.id, self.next_hit);
        }
    }

    fn calculate_next_day_diff_index(&self) -> u64 {
        let mut difference: u64 = 0;

        let today_index = chrono::Local::now().weekday().num_days_from_monday() as usize;
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

        if today_next_hit_theory <= 86400 && today_next_hit_theory > secs_since_midnight {
            return difference;
        }

        // Timer cannot be schedule today, so find next good day
        let next_index = today_index + 1 % self.days.len();
        

        for i in next_index..self.days.len() {
            if self.days[i] == 'X' {
                return difference;
            }
            difference += 1;
        }

        for i in 0..next_index {
            if self.days[i] == 'X' {
                return difference;
            }
            difference += 1;
        }

        return difference;
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