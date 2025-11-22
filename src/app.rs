use tokio::sync::mpsc;
use std::io;
use crossterm::event::KeyCode;

use crate::state::{AppState, AppEvent};
use crate::network::{NetworkHandle, connect_to_server};
use crate::tui::Tui;

pub struct App {
    state: AppState,
    tui: Tui,
    event_rx: mpsc::Receiver<AppEvent>,
    event_tx: mpsc::Sender<AppEvent>, // Add this
    network_handle_rx: mpsc::Receiver<NetworkHandle>,
    network_handle: Option<NetworkHandle>,
}

impl App {
    pub fn new(
        event_rx: mpsc::Receiver<AppEvent>,
        event_tx: mpsc::Sender<AppEvent>, // Add this parameter
        network_handle_rx: mpsc::Receiver<NetworkHandle>,
    ) -> io::Result<Self> {
        Ok(Self {
            state: AppState::new(),
            tui: Tui::new()?,
            event_rx,
            event_tx, // Add this
            network_handle_rx,
            network_handle: None,
        })
    }
    
    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> bool {
        // Returns should_quit
        match key.code {
            KeyCode::Esc => true,
            KeyCode::Enter => {
                let msg = self.state.input_buffer.clone();
                if !msg.is_empty() {
                    // Check if it's a command (starts with "!")
                    if msg.starts_with("!") {
                        self.handle_command(msg).await;
                    } else {
                        // Regular message
                        self.state.add_message(format!("You: {}", msg));
                        if let Some(ref handle) = self.network_handle {
                            handle.send(msg).await;
                        }
                    }
                    self.state.input_buffer.clear();
                }
                false
            }
            KeyCode::Char(c) => {
                self.state.input_buffer.push(c);
                false
            }
            KeyCode::Backspace => {
                self.state.input_buffer.pop();
                false
            }
            KeyCode::Up => {
                if self.state.scroll_offset < self.state.messages.len().saturating_sub(1) {
                    self.state.scroll_offset += 1;
                }
                false
            }
            KeyCode::Down => {
                self.state.scroll_offset = self.state.scroll_offset.saturating_sub(1);
                false
            }
            _ => false,
        }
    }
    
    async fn handle_command(&mut self, cmd: String) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        
        match parts[0] {
            "!connect" => {
                // Parse --ip flag
                let mut ip = None;
                let mut i = 1;
                while i < parts.len() {
                    if parts[i] == "--ip" && i + 1 < parts.len() {
                        ip = Some(parts[i + 1].to_string());
                        break;
                    }
                    i += 1;
                }
                
                if let Some(ip_addr) = ip {
                    self.state.add_message(format!("Connecting to {}...", ip_addr));
                    self.tui.draw(&self.state).ok();
                    
                    // Get event_tx from somewhere - we'll need to store it
                    // For now, let's add it to App struct
                    if let Some(handle) = connect_to_server(ip_addr, self.event_tx.clone()).await {
                        self.network_handle = Some(handle);
                        self.state.add_message("Connected successfully!".to_string());
                    } else {
                        self.state.add_message("Failed to connect.".to_string());
                    }
                } else {
                    self.state.add_message("Usage: !connect --ip <ip_address>".to_string());
                }
            }
            _ => {
                self.state.add_message(format!("Unknown command: {}", parts[0]));
            }
        }
    }
    
    fn handle_tcp_message(&mut self, msg: String) -> io::Result<()> {
        self.state.add_message(format!("Remote: {}", msg));
        self.state.scroll_offset = 0;
        self.tui.draw(&self.state)
    }
    
    fn handle_tcp_connected(&mut self, addr: std::net::SocketAddr) -> io::Result<()> {
        self.state.add_message(format!("Client connected: {}", addr));
        self.state.scroll_offset = 0;
        self.tui.draw(&self.state)
    }
    
    pub async fn run(&mut self) -> io::Result<()> {
        let mut should_quit = false;
        
        while !should_quit {
            tokio::select! {
                Some(handle) = self.network_handle_rx.recv() => {
                    self.network_handle = Some(handle);
                    self.state.add_message("Client connected!".to_string());
                    self.tui.draw(&self.state)?;
                }
                
                Some(event) = self.event_rx.recv() => {
                    match event {
                        AppEvent::KeyPress(key) => {
                            should_quit = self.handle_key_event(key).await;
                            self.tui.draw(&self.state)?;
                        }
                        AppEvent::TcpMessage(msg) => {
                            self.handle_tcp_message(msg)?;
                        }
                        AppEvent::TcpConnected(addr) => {
                            self.handle_tcp_connected(addr)?;
                        }
                        AppEvent::UiTick => {
                            self.tui.draw(&self.state)?;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }
}
