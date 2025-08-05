use crate::chat::ChatRoom;
use crate::protocol::{format_join_message, format_leave_message, format_user_list, is_valid_name};
use server::Metrics;
use std::{net::SocketAddr, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc::unbounded_channel;

pub async fn handle_client(
    stream: TcpStream,
    chat_room: Arc<ChatRoom>,
    addr: SocketAddr,
    metrics: Metrics,
) {
    let (tx, mut rx) = unbounded_channel();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut message_buffer = Vec::<u8>::new();

    let welcome_msg = b"Welcome to Nyx 3.0! What shall I call you?\n";
    if writer.write_all(welcome_msg).await.is_err() {
        server::log_error!(addr, "Failed to send welcome message");
        return;
    }
    metrics.bytes_sent(welcome_msg.len() as u64);
    server::log_msg_out!(addr, "Welcome message sent");

    let mut name = String::new();
    match reader.read_line(&mut name).await {
        Ok(0) => {
            server::log_info!(addr, "Client disconnected during handshake");
            return;
        }
        Ok(n) => {
            metrics.bytes_received(n as u64);
            server::log_msg_in!(addr, format!("Name: {}", name.trim()));
        }
        Err(e) => {
            server::log_error!(addr, format!("Failed to read name: {}", e));
            metrics.error_occurred();
            return;
        }
    }

    let name = name.trim().to_string();

    if !is_valid_name(&name) {
        server::log_warning!(addr, format!("Invalid name rejected: {}", name));
        let _ = writer.write_all(b"Invalid name. Disconnecting\n").await;
        return;
    }

    // Send presence notification and user list
    server::log_info!(addr, format!("User '{}' joined", name));
    chat_room.join(name.clone(), tx).await;
    let join_notif = format_join_message(&name);
    chat_room.broadcast(&join_notif, Some(&name)).await;
    let list_notif = format_user_list(&chat_room.user_list(Some(&name)).await);
    let _ = writer.write_all(list_notif.as_bytes()).await;

    // Main message loop
    loop {
        let mut buffer = [0u8; 1024];
        tokio::select! {
            result = reader.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        server::log_info!(addr, format!("User '{}' disconnected", name));
                        break;
                    }
                    Ok(n) => {
                        metrics.bytes_received(n as u64);
                        message_buffer.extend_from_slice(&buffer[..n]);

                        while let Some(newline_pos) = message_buffer.iter().position(|&b| b == b'\n') {
                            let line_bytes = message_buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                            let line = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 1]);
                            let msg = line.trim();

                            server::log_msg_in!(addr, format!("[{}]: {}", name, msg));
                            let formatted_msg = format!("[{}] {}\n", name, msg);
                            chat_room.broadcast(&formatted_msg, Some(&name)).await;
                        }
                    }
                    Err(e) => {
                        server::log_error!(addr, format!("Read error for '{}': {}", name, e));
                        metrics.error_occurred();
                        break;
                    }
                }
            }
            // Receive from chat room
            Some(msg) = rx.recv() => {
                if let Err(e) = writer.write_all(msg.as_bytes()).await {
                    server::log_error!(addr, format!("Write error for '{}': {}", name, e));
                    metrics.error_occurred();
                    break;
                }
                metrics.bytes_sent(msg.len() as u64);
                server::log_msg_out!(addr, msg.trim());
            }
        }
    }

    // Remove user from chat room
    chat_room.leave(&name).await;
    let leave_notif = format_leave_message(&name);
    chat_room.broadcast(&leave_notif, Some(&name)).await;
    server::log_info!(addr, format!("User '{}' has left the chat", name));
}
