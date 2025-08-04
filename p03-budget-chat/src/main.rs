mod chat;
mod client;
mod protocol;

use std::{error::Error, net::SocketAddr, sync::Arc};
use server::{run_tcp, Metrics};
use tokio::net::TcpStream;
use tokio::sync::OnceCell;

use crate::chat::ChatRoom;

static CHAT_ROOM: OnceCell<Arc<ChatRoom>> = OnceCell::const_new();

async fn chatroom_handler(
    stream: TcpStream,
    addr: SocketAddr,
    metrics: Metrics,
) -> Result<(), Box<dyn Error>> {
    server::log_info!(addr, "Chat client connected");

    let chat_room = CHAT_ROOM.get_or_init(|| async {
        ChatRoom::new()
    }).await;

    // Delegate to client handdler
    client::handle_client(stream, chat_room.clone(), addr, metrics).await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run_tcp("0.0.0.0:8000", chatroom_handler).await
}