use clap::Parser;

mod arg;
use arg::{Args, Action};

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        main_asnyc().await.unwrap();
    });
}

async fn main_asnyc() -> Result<i32, Box<dyn std::error::Error>> {
    let args = Args::parse();

    

    return Ok(0);
}
