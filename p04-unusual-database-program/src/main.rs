mod db;
mod protocol;

use server::{Metrics, run_udp};
use std::sync::Arc;
use std::{error::Error, net::SocketAddr};
use tokio::net::UdpSocket;

use crate::db::KVStore;
use crate::protocol::{Request, format_response, parse_request};

static DB: tokio::sync::OnceCell<Arc<KVStore>> = tokio::sync::OnceCell::const_new();

async fn kv_handler(
    packet: Vec<u8>,
    client_addr: SocketAddr,
    socket: Arc<UdpSocket>,
    metrics: Metrics,
) -> Result<(), Box<dyn Error>> {
    let db = DB
        .get_or_init(|| async { Arc::new(KVStore::new().await) })
        .await;

    match parse_request(&packet) {
        Ok(Request::Insert { key, value }) => {
            server::log_msg_in!(
                client_addr,
                format!(
                    "INSERT: {} = {:?}",
                    String::from_utf8_lossy(&key),
                    String::from_utf8_lossy(&value)
                )
            );

            if key == b"version" {
                server::log_warning!(client_addr, "Attempt to modify version key ignored");
            } else {
                let _ = db.insert(&key, &value).await;
                server::log_info!(client_addr, "Insert completed");
            }
        }

        Ok(Request::Retrieve { key }) => {
            server::log_msg_in!(
                client_addr,
                format!("RETRIEVE: {}", String::from_utf8_lossy(&key))
            );

            if let Some(value) = db.get(&key).await {
                let response = format_response(&key, &value);

                socket.send_to(&response, client_addr).await?;
                metrics.bytes_sent(response.len() as u64);

                server::log_msg_in!(client_addr, format!("Response: {} bytes", response.len()));
            } else {
                let response = format_response(&key, &[]);
                socket.send_to(&response, client_addr).await?;
                metrics.bytes_sent(response.len() as u64);

                server::log_msg_out!(client_addr, "Key not found, sent empty response");
            }
        }

        Err(e) => {
            server::log_warning!(client_addr, format!("Protocol error: {:?}", e));
            metrics.error_occurred();
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run_udp("0.0.0.0:8000", kv_handler).await
}
