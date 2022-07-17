use std::fmt;

use std::time::Duration;

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
}

impl Timer {
    pub fn new(name: String, kind: TimerType, interval: Duration, command: Command, next_hit: u64) -> Timer {
        return Timer {
            name: name,
            kind: kind,
            interval: interval,
            command: command,
            next_hit: next_hit,
        }
    }
}