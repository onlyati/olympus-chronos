use std::fmt;

#[derive(Clone, Copy)]
pub enum CommandOutputType {
    Info,
    Error,
}

impl fmt::Display for CommandOutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            CommandOutputType::Info => "I",
            CommandOutputType::Error => "E",
        };
        write!(f, "{}", display)
    }
}