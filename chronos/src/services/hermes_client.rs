use std::collections::HashMap;

use tonic::transport::{Channel, Certificate, ClientTlsConfig};
use tonic::{Request, Response, Status};

use hermes::hermes_client::{HermesClient};
use hermes::{SetPair, Pair};

mod hermes {
    tonic::include_proto!("hermes");
}

pub async fn start_hermes_client(config: &HashMap<String, String>, receiver: &std::sync::mpsc::Receiver<(String, String)>) -> Result<(), Box<dyn std::error::Error>> {
    // Address and table are mandatory parameters, they are enough for a simple non-TLS connection
    let addr = config.get("hermes.grpc.address").unwrap();
    let table = config.get("hermes.table").unwrap();

    // Get TLS related settiongs from the config
    let tls = match config.get("hermes.grpc.tls") {
        Some(tls) => tls.clone(),
        None => String::from("no"),
    };
    let tls_cert = config.get("hermes.grpc.tls.ca_cert");
    let tls_domain = config.get("hermes.grpc.tls.domain");

    // Create a gRPC channel for Hermes, TLS version if specified, else non-TLS
    let channel = if tls == "yes" && tls_cert.is_some() && tls_domain.is_some() {
        let pem = tokio::fs::read(tls_cert.unwrap()).await.unwrap();
        let ca = Certificate::from_pem(pem);

        let tls = ClientTlsConfig::new()
            .ca_certificate(ca)
            .domain_name(tls_domain.unwrap());

        Channel::from_shared(addr.clone())
            .unwrap()
            .tls_config(tls)
            .unwrap()
            .connect()
            .await
            .unwrap()
    }
    else {
        Channel::from_shared(addr.clone())
            .unwrap()
            .connect()
            .await
            .unwrap()
    };

    // Get key prefix for hermes
    let prefix = match config.get("hermes.key.prefix") {
        Some(s) => s.clone(),
        None => String::from(""),
    };

    // Create new gRPC client
    let mut client = HermesClient::new(channel);
    println!("Hermes client is ready");

    // Waiting for message what has to be send over to Hermes
    while let Ok(message) = receiver.recv() {
        println!("Update Hermes with {:?}", message);
        let key = format!("{}{}", prefix, message.0);
        let pair = SetPair {
            key: key,
            table: table.clone(),
            value: message.1,
        };

        let request = Request::new(pair);
        let response: Result<Response<Pair>, Status> = client.set(request).await;

        if let Err(e) = response {
            eprintln!("Failed to update Hermes: {}", e.message());
        }
    }

    return Ok(());
}
