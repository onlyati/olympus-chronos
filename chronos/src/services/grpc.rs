use std::collections::HashMap;
use std::str::FromStr;

use tonic::transport::{Identity, ServerTlsConfig};
use tonic::{transport::Server, Request, Response, Status};

use chronos::chronos_server::{Chronos, ChronosServer};
use chronos::{Empty, Timer, TimerList, TimerIdArg, TimerArg};

mod chronos {
    tonic::include_proto!("chronos");
}

use crate::VERBOSE;
use crate::TIMERS;

#[derive(Debug, Default)]
struct ChronosGrpc {
    timer_dir: String,
}

#[tonic::async_trait]
impl Chronos for ChronosGrpc {
    async fn verbose_log_on(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let mut v = VERBOSE.write().unwrap();
        *v = true;
        return Ok(Response::new(Empty {}));
    }

    async fn verbose_log_off(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let mut v = VERBOSE.write().unwrap();
        *v = false;
        return Ok(Response::new(Empty {}));
    }

    async fn list_active_timers(&self, _request: Request<Empty>) -> Result<Response<TimerList>, Status> {
        let timers = TIMERS.lock().unwrap();

        let mut ret_timers: Vec<Timer> = Vec::new();

        for timer in timers.iter() {
            let timer_item = Timer {
                id: timer.id.clone(),
                r#type: format!("{}", timer.r#type),
                interval: format!("{:?}", timer.interval),
                command: timer.command.join(" "),
                next_hit: format!("{:?}", tokio::time::Duration::from_secs(timer.next_hit)),
                days: timer.days.iter().collect(),
                dynamic: timer.dynamic
            };
            ret_timers.push(timer_item);
        }

        let ret_timers = TimerList {
            timers: ret_timers,
        };

        return Ok(Response::new(ret_timers));
    }

    async fn list_timer_configs(&self, _request: Request<Empty>) -> Result<Response<TimerList>, Status> {
        let timer_configs = crate::services::file::read_conf_files(&self.timer_dir);
        let mut timers: Vec<crate::structs::timer::Timer> = Vec::new();
        for config in timer_configs {
            match crate::structs::timer::Timer::from_config(config) {
                Ok(timer) => timers.push(timer),
                Err(e) => eprintln!("Failed to parse timer: {}", e),
            };            
        }

        let mut ret_timers: Vec<Timer> = Vec::new();

        for timer in timers.iter() {
            let timer_item = Timer {
                id: timer.id.clone(),
                r#type: format!("{}", timer.r#type),
                interval: format!("{:?}", timer.interval),
                command: timer.command.join(" "),
                next_hit: String::from("None"),
                days: timer.days.iter().collect(),
                dynamic: false,
            };
            ret_timers.push(timer_item);
        }

        let ret_timers = TimerList {
            timers: ret_timers,
        };

        return Ok(Response::new(ret_timers));
    }

    async fn purge_timer(&self, request: Request<TimerIdArg>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;

        let mut timers = TIMERS.lock().unwrap();
        let mut remove_index: Option<usize> = None;
        for i in 0..timers.len() {
            if timers[i].id == id {
                remove_index = Some(i);
                break;
            }
        }

        match remove_index {
            Some(index) => {
                timers.remove(index);
                return Ok(Response::new(Empty {}));
            }
            None => {
                return Err(Status::not_found(format!("No active timer was found with {} id", id)));
            }
        }
    }

    async fn refresh_timer(&self, request: Request<TimerIdArg>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;

        let path = format!("{}/{}.conf", self.timer_dir, id);
        let result = match crate::services::file::read_conf_file(path.as_str()) {
            Ok(conf) => conf,
            Err(e) => return Err(Status::cancelled(format!("Failed to read timer config '{}': {}", path, e))),
        };

        let timer = match crate::structs::timer::Timer::from_config(result) {
            Ok(timer) => timer,
            Err(e) => return Err(Status::cancelled(format!("Failed to parse timer: {}", e))),
        };

        let mut timers = TIMERS.lock().unwrap();
        
        for active_timer in timers.iter_mut() {
            if active_timer.id == id {
                *active_timer = timer;
                return Ok(Response::new(Empty {}));
            }
        }

        timers.push(timer);

        return Ok(Response::new(Empty {}));
    }

    async fn create_timer(&self, request: Request<TimerArg>) -> Result<Response<Empty>, Status> {
        let args = request.into_inner();
        let mut timer_config: HashMap<String, String> = HashMap::new();
        timer_config.insert(String::from("id"), args.id);
        timer_config.insert(String::from("type"), args.r#type);
        timer_config.insert(String::from("interval"), args.interval);
        timer_config.insert(String::from("command"), args.command);
        timer_config.insert(String::from("days"), args.days);

        let timer = match crate::structs::timer::Timer::from_config(timer_config) {
            Ok(timer) => timer,
            Err(e) => return Err(Status::cancelled(e)),
        };

        let mut timers = TIMERS.lock().unwrap();
        if !timers.contains(&timer) {
            timers.push(timer);
            return Ok(Response::new(Empty {}));
        }

        return Err(Status::already_exists("Timer id already active"));
    }
}


/// Start gRPC server, this must be run from a tokio runtime environment
pub async fn start_server(config: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    match config.get("host.grpc.address") {
        Some(addr) => {
            // Create structs
            let mut hepha_grpc = ChronosGrpc::default();
            hepha_grpc.timer_dir = config.get("timer.all_dir").unwrap().clone();

            let hepha_service = ChronosServer::new(hepha_grpc);

            let addr_list = tokio::net::lookup_host(addr).await?;

            let mut addr: Option<String> = None;
            for a in addr_list {
                addr = Some(format!("{}", a));
            }
            let addr = addr.unwrap();
            let addr = std::net::SocketAddr::from_str(&addr[..])?;

            // Read that TLS is required
            let tls = {
                match config.get("host.grpc.tls") {
                    Some(v) => v,
                    None => "no"
                }
            };

            if tls == "yes" {
                // If TLS required, we need to read certifications and keys and setup TLS for server
                let server_cert = match config.get("host.grpc.tls.pem") {
                    Some(v) => tokio::fs::read(v).await?,
                    None => {
                        eprintln!("Property 'host.grpc.tls.pem' is not specified");
                        return Ok(());
                    }
                };
                let server_key = match config.get("host.grpc.tls.key") {
                    Some(v) => tokio::fs::read(v).await?,
                    None => {
                        eprintln!("Property 'host.grpc.tls.key' is not specified");
                        return Ok(());
                    }
                };
                let server_identity = Identity::from_pem(server_cert, server_key);

                let tls = ServerTlsConfig::new()
                    .identity(server_identity);

                println!("Start gRPC endpoint in on {} with TLS", addr);
                Server::builder()
                    .tls_config(tls)?
                    .add_service(hepha_service)
                    .serve(addr)
                    .await?;
            }
            else {
                // If TLS is not reoquired, just start the server
                println!("Start gRPC endpoint on {}", addr);
                Server::builder()
                    .add_service(hepha_service)
                    .serve(addr)
                    .await?;    
            }
            
        }
        None => eprintln!("Hostname and port is not found in config with 'host.grpc.address' property"),
    }

    return Ok(());
}
