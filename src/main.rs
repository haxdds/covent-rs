use std::io;
use tokio::sync::mpsc;
use clap::{Parser, Subcommand};

mod state;
mod network;
mod tui;
mod input;
mod app;

use state::AppEvent;
use network::{connect_to_remote};
use tui::UiTicker;
use app::App;

#[derive(Parser)]
#[command(name = "covent")]
#[command(about = "A TUI chat application")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to another client
    Connect {
        /// IP address to connect to
        #[arg(short, long)]
        ip: String,
        /// Port to connect to (default: 6969)
        #[arg(short, long, default_value_t = 6969)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    
    // Create event channel
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>(100);
    
    // Spawn background tasks
    input::InputHandler::spawn(event_tx.clone());
    UiTicker::spawn(event_tx.clone());
    
    let network_handle_rx = match cli.command {
        Commands::Connect { ip, port } => {
            // Parse IP address
            let ip_addr: std::net::IpAddr = ip.parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid IP address: {}", e)))?;
            
            // Connect to remote
            let handle = connect_to_remote(ip_addr, port, event_tx.clone())
                .await
                .map_err(|e| io::Error::new(io::ErrorKind::ConnectionRefused, format!("Failed to connect: {}", e)))?;
            
            // Create a channel to send the handle
            let (handle_tx, handle_rx) = mpsc::channel(1);
            handle_tx.send(handle).await.ok();
            handle_rx
        }
    };
    
    // Create and run the application
    let mut app = App::new(event_rx, network_handle_rx)?;
    app.run().await?;
    
    println!("Exiting.");
    Ok(())
}