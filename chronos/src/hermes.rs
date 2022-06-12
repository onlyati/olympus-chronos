use std::net::TcpStream;
use std::io::{Read, Write};

/// Delete group from Hermes
pub fn hermes_del_group(name: &str) -> Result<String, String> {
    let address = match std::env::var("CHRONOS_HERMES_ADDR") {
        Ok(addr) => addr,
        Err(_) => return Err(String::from("Hermes is not enabled")),
    };

    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("DELETE /group?name={} HTTP/1.1\r\nAccept: */*\r\nContent-Length: 0\r\n", name);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}

/// Add group to Hermes
pub fn hermes_add_group(name: &str) -> Result<String, String> {
    let address = match std::env::var("CHRONOS_HERMES_ADDR") {
        Ok(addr) => addr,
        Err(_) => return Err(String::from("Hermes is not enabled")),
    };
    
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("POST /group?name={} HTTP/1.1\r\nAccept: */*\r\nContent-Length: 0\r\n", name);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}

/// Add timer onto timer group in Hermes
pub fn hermes_add_timer(name: &str, content: &str) -> Result<String, String> {
    let address = match std::env::var("CHRONOS_HERMES_ADDR") {
        Ok(addr) => addr,
        Err(_) => return Err(String::from("Hermes is not enabled")),
    };
    
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("POST /item?name={}&group=timer HTTP/1.1\r\nAccept: */*\r\nContent-Length: {}\r\n\r\n{}\r\n", name, content.len(), content);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}

/// Add timer onto timer group in Hermes
pub fn hermes_del_timer(name: &str) -> Result<String, String> {
    let address = match std::env::var("CHRONOS_HERMES_ADDR") {
        Ok(addr) => addr,
        Err(_) => return Err(String::from("Hermes is not enabled")),
    };
    
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            let msg = format!("DELETE /item?name={}&group=timer HTTP/1.1\r\nAccept: */*\r\nContent-Length: 0\r\n\r\n", name);
            stream.write(msg.as_bytes()).unwrap();
            let mut buffer = [0; 1024];

            match stream.read(&mut buffer) {
                Ok(r) => return Ok(String::from_utf8_lossy(&buffer[0..r]).trim().to_string()),
                Err(e) => return Err(format!("Error: {:?}", e)),
            }
        },
        Err(e) => return Err(format!("Failed to connect to Hermes: {}", e)),
    }
}