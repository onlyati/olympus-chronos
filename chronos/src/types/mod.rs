use std::fmt;
use std::time::Duration;
use std::collections::HashMap;
use chrono::{Weekday, Local, Datelike, Timelike};

/// Enum for type of timer
#[derive(PartialEq, Clone)]
pub enum TimerType {
    Every,
    OneShot,
    At,
}

impl fmt::Display for TimerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TimerType::Every => "every",
            TimerType::OneShot => "oneshot",
            TimerType::At => "at",
        };
        write!(f, "{}", printable)
    }
}

/// Struct for commands
#[derive(Clone)]
pub struct Command {
    pub bin: String,
    pub args: Vec<String>,
    pub user: String,
}

impl Command {
    pub fn new(cmd: Vec<String>, user: String) -> Command {
        let mut cmd_vec: Vec<String> = Vec::with_capacity((cmd.len() + 2) * std::mem::size_of::<String>());
        cmd_vec.push(String::from("-u"));
        cmd_vec.push(user.clone());
        for parm in cmd {
            cmd_vec.push(parm);
        }

        Command {
            bin: String::from("/usr/bin/sudo"),
            user: user,
            args: cmd_vec,
        }
    }
}

/// Timer struct
#[derive(Clone)]
pub struct Timer {
    pub name: String,
    pub kind: TimerType,
    pub interval: Duration,
    pub command: Command,
    pub next_hit: u64,
    pub days: Vec<bool>,
}

impl Timer {
    pub fn new(name: String, kind: TimerType, interval: Duration, command: Command, days: Vec<bool>) -> Timer {
        let now_from_unix_epoch = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n,
            Err(e) => panic!("Could not calculate time from UNIX_EPOC: {:?}", e),
        };

        let mut timer = Timer {
            name: name,
            kind: kind,
            interval: interval,
            command: command,
            next_hit: now_from_unix_epoch.as_secs(),
            days: days,
        };

        timer.calculate_next_hit();
        return timer;
    }

    pub fn calculate_next_hit(&mut self) {
        let today_weekday = Self::num_of_today();
        let now = Local::now().num_seconds_from_midnight();
        let now: u64 = now.into();

        // We are on that day when the timer has to run
        if self.days[today_weekday] {
            if self.kind == TimerType::At {        
                if now < self.interval.as_secs() {
                    self.next_hit = self.next_hit - now + self.interval.as_secs();
                    return;
                }
            }
            else {
                // For Every and OneShot timers just increment the current value
                self.next_hit += self.interval.as_secs();
                return;
            }
        }

        // From here, timer will not run today but on another day
        // Set next hit for today 24:00
        self.next_hit = self.next_hit + (86400 - now);

        // Find out which day need to run and calculate the differenec from today 24:00
        let next_weekday = Self::find_next_true(&self.days, today_weekday).unwrap();
        
        let day_diff = if next_weekday > today_weekday {
            next_weekday - today_weekday - 1
        }
        else {
            7 + (next_weekday - today_weekday - 1)
        };
        
        let day_diff = day_diff as u64;

        self.next_hit = self.next_hit + day_diff * 86400 + self.interval.as_secs();

    }

    fn find_next_true(elements: &Vec<bool>, cursor: usize) -> Option<usize> {
        let next_pos = cursor + 1;
        for i in next_pos..elements.len() {
            if elements[i] {
                return Some(i);
            }
        }
    
        for i in 0..next_pos {
            if elements[i] {
                return Some(i);
            }
        }
    
        return None;
    }
    
    fn num_of_today() -> usize {
        let now = Local::now();
        let mut day_map: HashMap<Weekday, usize> = HashMap::new();
        day_map.insert(Weekday::Mon, 0);
        day_map.insert(Weekday::Tue, 1);
        day_map.insert(Weekday::Wed, 2);
        day_map.insert(Weekday::Thu, 3);
        day_map.insert(Weekday::Fri, 4);
        day_map.insert(Weekday::Sat, 5);
        day_map.insert(Weekday::Sun, 6);
    
        return *day_map.get(&now.weekday()).unwrap();
    }
}
