use std::fmt;

use std::time::Duration;

/// Enum for type of timer
#[derive(PartialEq)]
pub enum TimerType {
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
pub struct Command {
    pub bin: String,
    pub args: Vec<String>,
}

/// Timer struct
pub struct Timer {
    pub name: String,
    pub kind: TimerType,
    pub interval: Duration,
    pub command: Command,
}

impl Timer {
    pub fn new(name: String, kind: TimerType, interval: Duration, command: Command) -> Timer {
        return Timer {
            name: name,
            kind: kind,
            interval: interval,
            command : command,
        }
    }
}