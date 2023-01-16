use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
pub struct Args {
    /// Specifiy the action what to do
    #[command(subcommand)]
    pub action: Action,

    /// Specifiy the host name or config pointer, for example: http://example.com or cfg://example
    #[arg(short = 'H', long)]
    pub hostname: String,

    /// If cfg:// specified at hostname, then this is where the config is read.
    #[arg(short, long, default_value_t = String::from("/etc/olympus/chronos/client.conf"))]
    pub config: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Turn on verbose log
    VerboseLogOn {},

    /// Turn off verbose log
    VerboseLogOff {},

    /// List currently active timers
    ListActive {},

    /// List static timers
    ListStatic {},

    /// Purge active timer
    Purge {
        /// Timer identifier, must be unique. Mandatory for purge, refresh and create actions.
        #[arg(short, long)]
        id: String,
    },

    /// Create dynamic timer
    Create {
        /// Timer identifier, must be unique. Mandatory for purge, refresh and create actions.
        #[arg(short, long)]
        id: String,

        /// Timer type: every, at or oneshot. Mandatory for create action.
        #[arg(short, long)]
        r#type: String,

        /// Timer interval in HH:MM:SS format. Mandatory for create action.
        #[arg(short = 'I', long)]
        interval: String,

        /// Command that timer execute. Mandatory for create action.
        #[arg(short, long)]
        command: String,

        /// Which days should it run, default is every day. Mandatory for create action.
        #[arg(short, long, default_value_t = String::from("XXXXXXX"))]
        #[arg(value_parser = validate_days)]
        days: String,
    },

    /// Refresh static timer
    Refresh {
        /// Timer identifier, must be unique. Mandatory for purge, refresh and create actions.
        #[arg(short, long)]
        id: String,
    },
}

fn validate_days(s: &str) -> Result<String, String> {
    if s.len() != 7 {
        return Err(String::from("Parameter must be 7 character"));
    }

    for c in s.chars() {
        if c != '_' && c != 'X' {
            return Err(String::from("Parameter can only contain 'X' and '_' characters"));
        }
    }

    return Ok(String::from(s));
}