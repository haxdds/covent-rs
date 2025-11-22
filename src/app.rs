use tokio::sync::mpsc;
use std::io;
use crossterm::event::KeyCode;

use crate::state::{AppState, AppEvent};
use crate::network::NetworkHandle;
use crate::tui::Tui;

pub struct App {
    state: AppState,
    tui: Tui,
    event_rx: mpsc::Receiver<AppEvent>,
    network_handle_rx: mpsc::Receiver<NetworkHandle>,
    network_handle: Option<NetworkHandle>,
}

impl App {
    pub fn new(
        event_rx: mpsc::Receiver<AppEvent>,
        network_handle_rx: mpsc::Receiver<NetworkHandle>,
    ) -> io::Result<Self> {
        Ok(Self {
            state: AppState::new(),
            tui: Tui::new()?,
            event_rx,
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
                    self.state.add_message(format!("You: {}", msg));
                    if let Some(ref handle) = self.network_handle {
                        handle.send(msg).await;
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
                    }
                }
            }
        }
        
        Ok(())
    }
}
