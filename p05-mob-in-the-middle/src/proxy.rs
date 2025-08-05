use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, AsyncBufReadExt};
use std::error::Error;
use std::net::SocketAddr;

use crate::rewrite::rewrite_boguscoin;

const UPSTREAM_ADDR: &str = "chat.protohackers.com:16963";

pub async fn handle_client(client: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let upstream =  TcpStream::connect(UPSTREAM_ADDR).await?;

    let (client_reader, mut client_writer) = client.into_split();
    let mut client_reader = BufReader::new(client_reader);
    let (upstream_reader, mut upstream_writer) = upstream.into_split();
    let mut upstream_reader = BufReader::new(upstream_reader);

    // Client -> Upstream
    let c2u = tokio::spawn(async move {
        let mut line = Vec::new();
        loop {
            line.clear();
            let n = client_reader.read_until(b'\n', &mut line).await?;
            if n == 0 { break; }
            let rewritten = rewrite_boguscoin(&line);
            upstream_writer.write_all(&rewritten).await?;
            server::log_msg_in!(addr, format!("{}", String::from_utf8_lossy(&line)));
            server::log_msg_in!(addr, format!("(REWRITTEN) {} ", String::from_utf8_lossy(&rewritten)));
        }
        Ok::<_, std::io::Error>(())
    });

    // Upstream -> Client
    let u2c = tokio::spawn(async move {
        let mut line = Vec::new();
        loop {
            line.clear();
            let n = upstream_reader.read_until(b'\n', &mut line).await?;
            if n == 0 { break; }
            let rewritten = rewrite_boguscoin(&line);
            client_writer.write_all(&rewritten).await?;
            server::log_msg_out!(addr, format!("{}", String::from_utf8_lossy(&line)));
            server::log_msg_out!(addr, format!("(REWRITTEN) {} ", String::from_utf8_lossy(&rewritten)));
        }
        Ok::<_, std::io::Error>(())
    });

    let _ = tokio::try_join!(c2u, u2c);

    Ok(())
}