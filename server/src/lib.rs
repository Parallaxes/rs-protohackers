use std::{
    error::Error,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub connections_total: Arc<AtomicU64>,
    pub connections_active: Arc<AtomicU64>,
    pub bytes_received: Arc<AtomicU64>,
    pub bytes_sent: Arc<AtomicU64>,
    pub errors_total: Arc<AtomicU64>,
    pub start_time: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            connections_total: Arc::new(AtomicU64::new(0)),
            connections_active: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            errors_total: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn connection_opened(&self) {
        self.connections_total.fetch_add(1, Ordering::Relaxed);
        self.connections_active.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn bytes_received(&self, count: u64) {
        self.bytes_received.fetch_add(count, Ordering::Relaxed);
    }

    pub fn bytes_sent(&self, count: u64) {
        self.bytes_sent.fetch_add(count, Ordering::Relaxed);
    }

    pub fn error_occurred(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn print_stats(&self) {
        println!("=== SERVER METRICS ===");
        println!("Uptime: {:?}", self.uptime());
        println!(
            "Total connections: {}",
            self.connections_total.load(Ordering::Relaxed)
        );
        println!(
            "Active connections: {}",
            self.connections_active.load(Ordering::Relaxed)
        );
        println!(
            "Bytes received: {}",
            self.bytes_received.load(Ordering::Relaxed)
        );
        println!("Bytes sent: {}", self.bytes_sent.load(Ordering::Relaxed));
        println!(
            "Total errors: {}",
            self.errors_total.load(Ordering::Relaxed)
        );
        println!("======================");
    }
}

pub async fn run_tcp<F, Fut>(addr: &str, handler: F) -> Result<(), Box<dyn Error>>
where
    F: Fn(TcpStream, SocketAddr, Metrics) -> Fut + Send + Sync + 'static + Copy,
    Fut: std::future::Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
{
    let listener = TcpListener::bind(addr).await?;
    let metrics = Metrics::new();

    log_info!(addr, "Server started");

    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            metrics_clone.print_stats();
        }
    });

    loop {
        let (stream, client_addr) = listener.accept().await?;
        log_info!(client_addr, "New connection");

        metrics.connection_opened();
        let metrics_clone = metrics.clone();

        tokio::spawn(async move {
            let result = handler(stream, client_addr, metrics_clone.clone()).await;
            metrics_clone.connection_closed();

            if let Err(e) = result {
                metrics_clone.error_occurred();
                log_error!(client_addr, format!("Connection error: {}", e));
            }
        });
    }
}

pub async fn run_udp<F, Fut>(addr: &str, handler: F) -> Result<(), Box<dyn Error>>
where
    F: Fn(Vec<u8>, SocketAddr, Arc<UdpSocket>, Metrics) -> Fut + Send + Sync + 'static + Copy,
    Fut: std::future::Future<Output = Result<(), Box<dyn Error>>> + Send + 'static,
{
    let socket = UdpSocket::bind(addr).await?;
    let socket = Arc::new(socket);
    let metrics = Metrics::new();

    log_info!(addr, "Server started");

    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            metrics_clone.print_stats();
        }
    });

    let mut buf = [0; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, client_addr)) => {
                metrics.bytes_received(len as u64);
                log_msg_in!(client_addr, format!("UDP packet ({} bytes)", len));

                let socket_clone = socket.clone();
                let metrics_clone = metrics.clone();
                let packet_data = buf[..len].to_vec();

                tokio::spawn(async move {
                    if let Err(e) = handler(
                        packet_data,
                        client_addr,
                        socket_clone,
                        metrics_clone.clone(),
                    )
                    .await
                    {
                        metrics_clone.error_occurred();
                        log_error!(client_addr, format!("Handler error: {}", e));
                    }
                });
            }
            Err(e) => {
                metrics.error_occurred();
                log_error!(addr, format!("UDP recv error: {}", e));
            }
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($addr:expr, $msg:expr) => {
        println!("[INFO] [{}] {}", $addr, $msg);
    };
    ($msg:expr, $addr:expr) => {
        println!("[INFO] [SERVER] {}", format!($msg, $addr));
    };
}

#[macro_export]
macro_rules! log_error {
    ($addr:expr, $msg:expr, $err:expr) => {
        eprintln!("[ERROR] [{}] {}: {}", $addr, $msg, $err);
    };
    ($addr:expr, $msg:expr) => {
        eprintln!("[ERROR] [{}] {}", $addr, $msg);
    };
}

#[macro_export]
macro_rules! log_warning {
    ($addr:expr, $msg:expr) => {
        println!("[WARNING] [{}] {}", $addr, $msg);
    };
}

#[macro_export]
macro_rules! log_msg_out {
    ($addr:expr, $msg:expr) => {
        println!("[--->] [{}] {}", $addr, $msg);
    };
}

#[macro_export]
macro_rules! log_msg_in {
    ($addr:expr, $msg:expr) => {
        println!("[<---] [{}] {}", $addr, $msg);
    };
}
