use tokio::{
    net::TcpListener,
    io::{AsyncWriteExt, AsyncBufReadExt},
    sync::mpsc,
};
use crate::AppEvent;


// ============================================================================
// NETWORK HANDLE
// ============================================================================

pub struct NetworkHandle {
    pub tx: mpsc::Sender<String>,
}

impl NetworkHandle {
    pub async fn send(&self, msg: String) {
        self.tx.send(msg).await.ok();
    }
}

// ============================================================================
// TCP LISTENER (same as before)
// ============================================================================

pub async fn spawn_tcp_listener(
    event_tx: mpsc::Sender<AppEvent>,
) -> mpsc::Receiver<NetworkHandle> {
    let (handle_tx, handle_rx) = mpsc::channel(1);
    
    tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:6969").await.unwrap();
        
        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(x) => x,
                Err(_) => break,
            };
            
            event_tx.send(AppEvent::TcpConnected(addr)).await.ok();
            
            let (read_half, write_half) = stream.into_split();
            let (client_tx, mut client_rx) = mpsc::channel::<String>(100);
            
            handle_tx.send(NetworkHandle { tx: client_tx }).await.ok();
            
            // Writer task
            let mut write_half = write_half;
            tokio::spawn(async move {
                while let Some(msg) = client_rx.recv().await {
                    if write_half.write_all(msg.as_bytes()).await.is_err() {
                        break;
                    }
                    write_half.write_all(b"\n").await.ok();
                }
            });
            
            // Reader task
            let event_tx_clone = event_tx.clone();
            tokio::spawn(async move {
                let mut reader = tokio::io::BufReader::new(read_half);
                let mut line = String::new();
                
                while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    event_tx_clone
                        .send(AppEvent::TcpMessage(line.trim().to_string()))
                        .await
                        .ok();
                    line.clear();
                }
            });
        }
    });
    
    handle_rx
}