mod proxy;
mod rewrite;

use server::{Metrics, run_tcp};
use std::{error::Error, net::SocketAddr};
use tokio::net::TcpStream;

use crate::proxy::handle_client;

async fn middle_handler(
    stream: TcpStream,
    addr: SocketAddr,
    metrics: Metrics,
) -> Result<(), Box<dyn Error>> {
    server::log_info!(addr, "Proxy connection opened");
    let result = handle_client(stream, addr).await;

    if let Err(ref e) = result {
        metrics.error_occurred();
        server::log_error!(addr, "Proxy handler error: {}", e);
    } else {
        server::log_info!(addr, "Proxy connection closed");
    }

    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run_tcp("0.0.0.0:8000", middle_handler).await
}