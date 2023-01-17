use clap::Parser;
use tonic::transport::{Channel, Certificate, ClientTlsConfig};
use tonic::{Request, Response, Status};
use std::process::exit;

use chronos::chronos_client::{ChronosClient};
use chronos::{Empty, TimerList, TimerIdArg, TimerArg};

mod chronos {
    tonic::include_proto!("chronos");
}

mod arg;
use arg::{Args, Action};

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        match main_asnyc().await {
            Ok(rc) => exit(rc),
            Err(_) => exit(-999),
        }
    });
}

async fn main_asnyc() -> Result<i32, Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Measure runtime of script
    let start = std::time::Instant::now();

    // Try to create and connect to gRPC server
    let grpc_channel = create_grpc_channel(args.clone()).await;

    let mut grpc_client = ChronosClient::new(grpc_channel);

    let mut final_rc = 0;

    match args.action {
        Action::Create { ref id, ref r#type, ref interval, ref command, ref days } => {
            let parms = TimerArg {
                id: id.clone(),
                r#type: r#type.clone(),
                interval: interval.clone(),
                command: command.clone(),
                days: days.clone(),
            };
            let response: Result<Response<Empty>, Status> = grpc_client.create_timer(Request::new(parms)).await;
            match response {
                Ok(_) => {
                    println!("Timer is created!");
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        },
        Action::ListActive => {
            let response: Result<Response<TimerList>, Status> = grpc_client.list_active_timers(Request::new(Empty { })).await;
            match response {
                Ok(resp) => {
                    let timers = resp.into_inner();
                    let mut timers = timers.timers;
                    timers.sort_by(|a, b| a.next_hit.cmp(&b.next_hit));

                    let mut width_id = 2;
                    let mut width_command = 7;

                    for timer in &timers {
                        if timer.id.len() > width_id {
                            width_id = timer.id.len();
                        }
                        if timer.command.len() > width_command {
                            width_command = timer.command.len();
                        }
                    }

                    println!("{:^w_id$} | {:^7} | {:^8} | {:^19} | {:^7} | {:^1} | {:<w_cmd$}", "ID", "Type", "Period", "Next run", "Days", "D", "Command", w_id = width_id, w_cmd = width_command);
                    println!("{:-<w_id$} + {:-<7} + {:-<8} + {:-<19} + {:-<7} + {:-<1} + {:-<w_cmd$}", "", "", "", "", "", "", "", w_id = width_id, w_cmd = width_command);

                    for timer in timers {
                        let r#dyn = if timer.dynamic { "Y" } else { "N" };
                        println!("{:w_id$} | {:7} | {:8} | {:19} | {:7} | {:1} | {:w_cmd$}", timer.id, timer.r#type, timer.interval, timer.next_hit, timer.days, r#dyn, timer.command, w_id = width_id, w_cmd = width_command);
                    }
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        Action::ListStatic => {
            let response: Result<Response<TimerList>, Status> = grpc_client.list_timer_configs(Request::new(Empty { })).await;
            match response {
                Ok(resp) => {
                    let timers = resp.into_inner();
                    let timers = timers.timers;

                    let mut width_id = 2;
                    let mut width_command = 7;

                    for timer in &timers {
                        if timer.id.len() > width_id {
                            width_id = timer.id.len();
                        }
                        if timer.command.len() > width_command {
                            width_command = timer.command.len();
                        }
                    }

                    println!("{:^w_id$} | {:^7} | {:^8} | {:^7} | {:^1} | {:<w_cmd$}", "ID", "Type", "Period", "Days", "D", "Command", w_id = width_id, w_cmd = width_command);
                    println!("{:-<w_id$} + {:-<7} + {:-<8} + {:-<7} + {:-<1} + {:-<w_cmd$}", "", "", "", "", "", "", w_id = width_id, w_cmd = width_command);

                    for timer in timers {
                        let r#dyn = if timer.dynamic { "Y" } else { "N" };
                        println!("{:w_id$} | {:7} | {:8} | {:7} | {:1} | {:w_cmd$}", timer.id, timer.r#type, timer.interval, timer.days, r#dyn, timer.command, w_id = width_id, w_cmd = width_command);
                    }
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        Action::Purge { ref id } => {
            let response: Result<Response<Empty>, Status> = grpc_client.purge_timer(Request::new(TimerIdArg { id: id.clone() })).await;
            match response {
                Ok(_) => {
                    println!("Timer is purged");
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        Action::Refresh { ref id } => {
            let response: Result<Response<Empty>, Status> = grpc_client.refresh_timer(Request::new(TimerIdArg { id: id.clone() })).await;
            match response {
                Ok(_) => {
                    println!("Timer refreshed");
                }
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        Action::VerboseLogOff => {
            let response: Result<Response<Empty>, Status> = grpc_client.verbose_log_off(Request::new(Empty {})).await;
            match response {
                Ok(_) => println!("Log verbose is off"),
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
        Action::VerboseLogOn => {
            let response: Result<Response<Empty>, Status> = grpc_client.verbose_log_on(Request::new(Empty {})).await;
            match response {
                Ok(_) => println!("Log verbose is on"),
                Err(e) => {
                    eprintln!("Failed request: {}", e.message());
                    final_rc = 4;
                }
            }
        }
    }

    let elapsed = start.elapsed();
    print_verbose(&args, format!("Measured runtime: {:?}", elapsed));

    return Ok(final_rc);
}

/// Print text only, when verbose flag is set
fn print_verbose<T: std::fmt::Display>(args: &Args, text: T) {
    if args.verbose {
        println!("> {}", text);
    }
}

/// Create a new gRPC channel which connection to Hephaestus
async fn create_grpc_channel(args: Args) -> Channel {
    if !args.hostname.starts_with("cfg://") {
        print_verbose(&args, "Not cfg:// protocol is given");
        return Channel::from_shared(args.hostname.clone())
            .unwrap()
            .connect()
            .await
            .unwrap();
    }

    let host = args.hostname[6..].to_string();

    print_verbose(&args, format!("cfg:// is specified, will be looking for in {} for {} settings", host, args.config));

    let config = match onlyati_config::read_config(&args.config[..]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read config: {}", e);
            std::process::exit(2);
        }
    };

    let addr = match config.get(&format!("node.{}.address", host)) {
        Some(a) => a.clone(),
        None => {
            eprintln!("No address is found for '{}' in config", host);
            std::process::exit(2);
        }
    };

    let ca = config.get(&format!("node.{}.ca_cert", host));
    let domain = config.get(&format!("node.{}.domain", host));

    print_verbose(&args, format!("{:?}, {:?}", ca, domain));

    if ca.is_some() && domain.is_some() {
        let pem = match tokio::fs::read(ca.unwrap()).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to read {}: {}", ca.unwrap(), e);
                std::process::exit(2);
            }
        };
        let ca = Certificate::from_pem(pem);

        let tls = ClientTlsConfig::new()
            .ca_certificate(ca)
            .domain_name(domain.unwrap());
        
        return Channel::from_shared(addr)
            .unwrap()
            .tls_config(tls)
            .unwrap()
            .connect()
            .await
            .unwrap();
    }
    else {
        return Channel::from_shared(addr)
            .unwrap()
            .connect()
            .await
            .unwrap();
    }
}
