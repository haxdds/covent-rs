use std::io;
use tokio::sync::mpsc;

mod app;
mod input;
mod network;
mod state;
mod tui;

use app::App;
use input::spawn_input_handler;
use network::spawn_tcp_listener;
use state::AppEvent;
use tui::spawn_ui_ticker;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>(100);

    spawn_input_handler(event_tx.clone());
    spawn_ui_ticker(event_tx.clone());
    let network_handle_rx = spawn_tcp_listener(event_tx.clone()).await;

    let mut app = App::new(event_rx, event_tx, network_handle_rx)?;
    app.run().await?;

    println!("Exiting.");
    Ok(())
}
