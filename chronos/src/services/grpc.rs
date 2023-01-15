use tonic::transport::{Identity, ServerTlsConfig};
use tonic::{transport::Server, Request, Response, Status};

use chronos::chronos_server::{Chronos, ChronosServer};
use chronos::{Empty, Timer, TimerList, TimerIdArg, TimerArg};

mod chronos {
    tonic::include_proto!("chronos");
}

use crate::VERBOSE;

#[derive(Debug, Default)]
struct ChronosGrpc { }

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

    async fn list_active_timers(&self, request: Request<Empty>) -> Result<Response<TimerList>, Status> {
        unimplemented!()
    }

    async fn list_timer_configs(&self, request: Request<Empty>) -> Result<Response<TimerList>, Status> {
        unimplemented!()
    }

    async fn purge_timer(&self, request: Request<TimerIdArg>) -> Result<Response<Empty>, Status> {
        unimplemented!()
    }

    async fn refresh_timer(&self, request: Request<TimerIdArg>) -> Result<Response<Empty>, Status> {
        unimplemented!()
    }

    async fn create_timer(&self, request: Request<TimerArg>) -> Result<Response<Empty>, Status> {
        unimplemented!()
    }
}
