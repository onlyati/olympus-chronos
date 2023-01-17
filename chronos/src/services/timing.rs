use std::sync::mpsc::Sender;

pub async fn start_trigger_timer(sender: Sender<u64>) {
    let interval = tokio::time::Duration::from_secs(1);

    loop {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(v) => sender.send(v.as_secs()).unwrap(),
            Err(e) => panic!("Could not get the second since UNIX_EPOCH: {}", e),
        }

        tokio::time::sleep(interval).await;
    }
}