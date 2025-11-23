use crate::AppEvent;
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{
        TcpListener, TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::mpsc,
    time::timeout,
};

pub struct NetworkHandle {
    pub tx: mpsc::Sender<String>,
}

impl NetworkHandle {
    pub async fn send(&self, msg: String) -> Result<(), mpsc::error::SendError<String>> {
        self.tx.send(msg).await
    }
}

// Helper function to spawn a writer task for a TCP connection
fn spawn_writer_task(mut write_half: OwnedWriteHalf, mut client_rx: mpsc::Receiver<String>) {
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if write_half.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
            write_half.write_all(b"\n").await.ok();
        }
    });
}

// Helper function to spawn a reader task for a TCP connection
fn spawn_reader_task(read_half: OwnedReadHalf, event_tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            event_tx
                .send(AppEvent::TcpMessage(line.trim().to_string()))
                .await
                .ok();
            line.clear();
        }
    });
}

// Helper function to set up reader and writer tasks for a TCP stream
fn setup_tcp_connection(stream: TcpStream, event_tx: mpsc::Sender<AppEvent>) -> NetworkHandle {
    let (read_half, write_half) = stream.into_split();
    let (client_tx, client_rx) = mpsc::channel::<String>(100);

    spawn_writer_task(write_half, client_rx);
    spawn_reader_task(read_half, event_tx);

    NetworkHandle { tx: client_tx }
}

pub async fn spawn_tcp_listener(event_tx: mpsc::Sender<AppEvent>) -> mpsc::Receiver<NetworkHandle> {
    let (handle_tx, handle_rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let listener = match TcpListener::bind("0.0.0.0:6969").await {
            Ok(l) => l,
            Err(e) => {
                event_tx
                    .send(AppEvent::TcpMessage(format!(
                        "Failed to bind listener: {}",
                        e
                    )))
                    .await
                    .ok();
                return;
            }
        };

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(x) => x,
                Err(_) => break,
            };

            event_tx.send(AppEvent::TcpConnected(addr)).await.ok();

            let handle = setup_tcp_connection(stream, event_tx.clone());
            handle_tx.send(handle).await.ok();
        }
    });

    handle_rx
}

pub async fn connect_to_server(
    ip: String,
    event_tx: mpsc::Sender<AppEvent>,
) -> Option<NetworkHandle> {
    let addr = format!("{}:6969", ip);
    let connect_future = TcpStream::connect(addr);

    match timeout(Duration::from_secs(5), connect_future).await {
        Ok(Ok(stream)) => {
            let peer_addr = stream.peer_addr().ok()?;
            event_tx.send(AppEvent::TcpConnected(peer_addr)).await.ok();

            let handle = setup_tcp_connection(stream, event_tx.clone());
            Some(handle)
        }
        Err(e) => {
            event_tx
                .send(AppEvent::TcpMessage(format!("Connection timed out: {}", e)))
                .await
                .ok();
            None
        }
        Ok(Err(e)) => {
            event_tx
                .send(AppEvent::TcpMessage(format!("Connection failed: {}", e)))
                .await
                .ok();
            None
        }
    }
}
