mod protocol;
mod session;

use server::{Metrics, run_tcp};
use std::{error::Error, net::SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use protocol::{Message, serialize_mean};

use crate::session::Session;

async fn query_handler(
    stream: TcpStream,
    addr: SocketAddr,
    metrics: Metrics,
) -> Result<(), Box<dyn Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut buf = [0u8; 9];

    let mut session = Session::new();

    loop {
        match reader.read_exact(&mut buf).await {
            Ok(0) => {
                server::log_info!(addr, "Connection closed by client");
                break;
            }
            Ok(n) => {
                metrics.bytes_received(n as u64);
                server::log_msg_in!(addr, format!("Received {} bytes", n));

                match Message::parse(&buf) {
                    Some(Message::Query { mintime, maxtime }) => {
                        let mean = session.query(mintime, maxtime);
                        let response = serialize_mean(mean);
                        writer.write_all(&response).await?;
                    }
                    Some(Message::Insert { timestamp, price }) => {
                        session.insert(timestamp, price);
                    }
                    _ => {
                        server::log_warning!(addr, "Malformed request");
                        let response = "unrecognized request, disconnecting";
                        writer.write_all(response.as_bytes()).await?;
                        break;
                    }
                }
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
    run_tcp("0.0.0.0:8000", query_handler).await
}
