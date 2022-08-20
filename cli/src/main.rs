use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::process::exit;
use std::env;
use std::path::Path;
use std::collections::HashMap;

fn main() {
    /*-------------------------------------------------------------------------------------------*/
    /* Read all parameter then parse them to words                                               */
    /*-------------------------------------------------------------------------------------------*/
    let args: Vec<String> = env::args().collect();
    let args = args.join(" ");
    let mut args: Vec<&str> = args.split_whitespace().collect();

    args.remove(0);

    let defaults = get_defaults();

    /*-------------------------------------------------------------------------------------------*/
    /* Parse the input and upload the Argument struct with those values                          */
    /*-------------------------------------------------------------------------------------------*/
    let mut input: Argument = Argument { address: None, command: None, verbose: false };

    for i in 0..args.len() {
        if args[i] == "-h" || args[i] == "--help" {
            // It is the help of CLI, show it then exit
            display_help();
            exit(0);
        }

        if args[i] == "--version" {
            println!("v0.1.2");
            exit(0);
        }

        if i > 0 {
            if args[i - 1] == "-a" {
                input.address = Some(String::from(args[i]));
                continue;
            }
        }

        if args[i] == "-a" {
            // Address must be followed after "-a", no extra to do just check the next word for address
            continue;
        }

        if args[i] == "-v" {
            // We want to display more things
            input.verbose = true;
            continue;
        }

        // At this point, it is the COMMAND part of argument
        if let None = &mut input.command {
            input.command = Some(String::from(args[i]));
            continue;
        }
        if let Some(cmd) = &mut input.command {
            cmd.push(' ');
            cmd.push_str(args[i]);
        }
    }

    if let None = input.address {
        input.address = match defaults.get("address") {
            Some(addr) => Some(addr.clone()),
            None => None,
        };
    }

    if input.verbose {
        println!("#Address: >{:?}<", input.address);
        println!("#Command: >{:?}<", input.command);
    }

    let message = match input.command {
        Some(cmd) => {
            format!("{} {}", cmd.len(), cmd)
        },
        None => {
            println!(">Error\nCommand must be specified");
            exit(1);
        }
    };

    if input.verbose {
        println!("#Message to Chronos: >{}<", message);
    }

    let mut stream = match input.address {
        Some(addr) => {
            match UnixStream::connect(Path::new(&addr)) {
                Ok(v) => v,
                Err(e) => {
                    println!(">Error\nError during connect to socket: {e:?}");
                    exit(1);        
                }
            }
        },
        None => {
            println!(">Error\nAddress field is not specified");
            exit(1);
        }
    };

    let now = std::time::Instant::now();
    stream.write(message.as_bytes()).unwrap();
    
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let elapsed = now.elapsed();
    println!("{response}");

    if input.verbose{
        println!("#Elapsed time: {:?}", elapsed);
    }

    if response.lines().next().unwrap() != ">Done" {
        exit(10);
    }
}

fn get_defaults() -> HashMap<String, String> {
    let config = match onlyati_config::read_config("/etc/olympus/chronos/defaults") {
        Ok(conf) => conf,
        Err(_) => HashMap::new(),
    };

    return config;
}

struct Argument {
    address: Option<String>,
    command: Option<String>,
    verbose: bool,
}

fn display_help() {
    println!("Syntax of command:");
    println!("");
    println!("   chronos-cli [-v] -a <address> COMMAND");
    println!("");
    println!("   -v");
    println!("      Verbose switch. If this is put in the command then more details is written.");
    println!("");
    println!("   -a <address>");
    println!("      UNIX domain socker for Chronos server.");
    println!("");
    println!("   COMMAND");
    println!("      Hermes command what you want to execute. Execute 'help' Hermes command to display them.");
}
