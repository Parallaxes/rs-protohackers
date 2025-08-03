use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("Server started on port 8000");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connection from {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_stream(stream).await {
                eprintln!("Connection error: {:?}", e);
            }
        });
    }
}

async fn handle_stream(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                // EOF reached
                println!("Connection closed by client");
                break;
            }
            Ok(n) => {
                println!("Received {} bytes: {:?}", n, &buf[..n]);
                stream.write_all(&buf[..n]).await?;
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}