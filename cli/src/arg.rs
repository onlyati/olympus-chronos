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
    #[arg(value_parser = check_hostname)]
    pub hostname: String,

    /// If cfg:// specified at hostname, then this is where the config is read.
    #[arg(short, long, default_value_t = String::from("/etc/olympus/chronos/client.conf"))]
    pub config: String,

    /// Show more detail about connection
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Turn on verbose log
    VerboseLogOn,

    /// Turn off verbose log
    VerboseLogOff,

    /// List currently active timers
    ListActive,

    /// List static timers
    ListStatic,

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
        #[arg(value_parser = validate_type)]
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

fn validate_type(s: &str) -> Result<String, String> {
    if s != "at" && s != "every" && s != "oneshot" {
        return Err(String::from("Type can be only: at, every or oneshot"));
    }

    return Ok(String::from(s));
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

fn check_hostname(s: &str) -> Result<String, String> {
    if !s.starts_with("http://") && !s.starts_with("https://") && !s.starts_with("cfg://") {
        return Err(String::from("Protocol for hostname can be http:// or https:// or cfg://. "));
    }

    if s.starts_with("http://") || s.starts_with("https://") {
        if !s.contains(':') {
            return Err(String::from("Port number is not specified after the hostname. "));
        }
        else {
            let port = s.split(':').nth(2);
            match port {
                Some(p) => {
                    match p.parse::<u32>() {
                        Ok(num) => {
                            if num > 65535 {
                                return Err(String::from("Port number can be between 0..65535"));
                            }
                        },
                        Err(_) => {
                            return Err(String::from("Failed to convert port number to numbers"));
                        }
                    }
                },
                None => return Err(String::from("Port number is not specified after the hostname. ")),
            }
        }
    }

    return Ok(String::from(s));
}