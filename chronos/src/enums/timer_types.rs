use std::fmt;

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