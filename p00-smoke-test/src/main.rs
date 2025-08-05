use std::{error::Error, net::SocketAddr};

use server::{Metrics, run_tcp};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

async fn echo_handler(
    mut stream: TcpStream,
    addr: SocketAddr,
    metrics: Metrics,
) -> Result<(), Box<dyn Error>> {
    server::log_info!(addr, "Echo handler started");
    let mut buf = [0u8; 1024];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                server::log_info!(addr, "Connection closed by client");
                break;
            }
            Ok(n) => {
                metrics.bytes_received(n as u64);
                server::log_msg_in!(addr, format!("{} bytes", n));

                stream.write_all(&buf[..n]).await?;
                metrics.bytes_sent(n as u64);
                server::log_msg_out!(addr, format!("{} byte echoed", n));
            }
            Err(e) => {
                metrics.error_occurred();
                server::log_error!(addr, format!("Read error: {}", e));
                break;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run_tcp("0.0.0.0:8000", echo_handler).await
}
