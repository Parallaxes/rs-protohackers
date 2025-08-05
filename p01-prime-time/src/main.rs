mod prime;
mod protocol;

use server::{Metrics, run_tcp};
use std::{error::Error, net::SocketAddr};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::prime::is_prime;
use crate::protocol::*;

async fn prime_handler(
    stream: TcpStream,
    addr: SocketAddr,
    metrics: Metrics,
) -> Result<(), Box<dyn Error>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                server::log_info!(addr, "Connection closed by client");
                break;
            }
            Ok(n) => {
                metrics.bytes_received(n as u64);
                server::log_msg_in!(addr, line.trim());

                let response = match parse_request(&line) {
                    Ok(request) if request.is_valid() => {
                        if let Some(number) = request.get_number() {
                            let prime_result = is_prime(number);
                            server::log_info!(
                                addr,
                                format!("isPrime({}) = {}", number, prime_result)
                            );
                            serialize_response(&Response::new(prime_result))?
                        } else {
                            server::log_warning!(addr, "Invalid number in request");
                            let response = serialize_response(&MalformedResponse::new())?;
                            writer.write_all(response.as_bytes()).await?;
                            break;
                        }
                    }
                    _ => {
                        server::log_warning!(addr, "Malformed request");
                        let response = serialize_response(&MalformedResponse::new())?;
                        writer.write_all(response.as_bytes()).await?;
                        break;
                    }
                };

                writer.write_all(response.as_bytes()).await?;
                metrics.bytes_sent(response.len() as u64);
                server::log_msg_out!(addr, response.trim());
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
    run_tcp("0.0.0.0:8000", prime_handler).await
}
