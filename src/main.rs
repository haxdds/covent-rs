use std::io;
use tokio::sync::mpsc;

mod state;
mod network;
mod tui;
mod input;
mod app;

use state::AppEvent;
use network::spawn_tcp_listener;
use tui::UiTicker;
use app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Create event channel
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>(100);
    
    // Spawn background tasks
    input::InputHandler::spawn(event_tx.clone());
    UiTicker::spawn(event_tx.clone());
    let network_handle_rx = spawn_tcp_listener(event_tx.clone()).await;
    
    // Create and run the application
    let mut app = App::new(event_rx, event_tx, network_handle_rx)?; // Pass event_tx here
    app.run().await?;
    
    println!("Exiting.");
    Ok(())
}