use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::process::exit;
use std::env;
use std::path::Path;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing input parameters!");
        exit(1);
    }

    args.remove(0);

    if args[0] == "--version" {
        println!("v.0.1.1");
        exit(0);
    }

    let socket = if args[0] == "--dev" {
        args.remove(0);
        Path::new("/tmp/chronos-dev.sock")
    } else {
        Path::new("/tmp/chronos.sock")
    };

    let mut message = String::new();
    for arg in args {
        message += &arg[..];
        message += " ";
    }

    let count = message.len();

    let message = format!("{} {}", count, message);

    let mut stream = match UnixStream::connect(socket) {
        Ok(v) => v,
        Err(e) => {
            println!("Error during connect to socket: {e:?}");
            exit(1);
        }
    };

    if let Err(e) = stream.write_all(&message.as_bytes()) {
        println!("Error during sending data: {e:?}");
    }

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("{response}");
}
