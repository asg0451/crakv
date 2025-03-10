use std::sync::Arc;

use async_maelstrom::{Status, runtime::Runtime};
use tracing::info;

mod echo;
mod simple_kv;

#[tokio::main]
async fn main() -> Status {
    // Log to stderr where Maelstrom will capture it
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    info!("starting");

    // Create an echo process and a runtime to execute it
    let process: echo::EchoServer = Default::default();
    let r = Arc::new(Runtime::new(std::env::args().collect(), process).await?);

    // Drive the runtime, and ...
    let (r1, r2, r3) = (r.clone(), r.clone(), r.clone());
    let t1 = tokio::spawn(async move { r1.run_io_egress().await });
    let t2 = tokio::spawn(async move { r2.run_io_ingress().await });
    let t3 = tokio::spawn(async move { r3.run_process().await });

    // ... wait until the Maelstrom system closes stdin and stdout
    info!("running");
    let _ignored = tokio::join!(t1, t2, t3);

    info!("stopped");

    Ok(())
}
