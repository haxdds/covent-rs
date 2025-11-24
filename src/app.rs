use ratatui::crossterm::event::KeyCode;
use std::io;
use tokio::sync::mpsc;

use crate::network::{NetworkHandle, connect_to_server};
use crate::state::{self, AppEvent, AppState};
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

    async fn handle_key_event(&mut self, key: ratatui::crossterm::event::KeyEvent) {
        if key.code == KeyCode::Enter && key.modifiers.is_empty() {
            let msg = self.state.textarea.lines()[0].clone();

            if !msg.is_empty() {
                if msg.starts_with("!") {
                    self.handle_command(msg).await;
                } else {
                    self.state.add_message(format!("You: {}", msg));
                    if let Some(ref handle) = self.network_handle {
                        handle.send(msg).await.ok();
                    }
                }

                self.state.textarea = state::get_default_textarea();
            }
        } else {
            self.state.textarea.input(key);
        }
    }

    async fn execute_bash_command(&mut self, cmd: &str, send_to_remote: bool) {
        match tokio::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .await
        {
            Ok(output) => {
                let has_stdout = !output.stdout.is_empty();
                let has_stderr = !output.stderr.is_empty();

                if has_stdout {
                    let stdout_str = String::from_utf8_lossy(&output.stdout);
                    self.state
                        .add_message(format!("bash output: {}", stdout_str.trim()));

                    if send_to_remote {
                        self.event_tx
                            .send(AppEvent::BashCmd(stdout_str.trim().to_string()))
                            .await
                            .ok();
                    }
                }

                if has_stderr {
                    let stderr_str = String::from_utf8_lossy(&output.stderr);
                    self.state
                        .add_message(format!("bash error: {}", stderr_str.trim()));

                    if send_to_remote {
                        self.event_tx
                            .send(AppEvent::BashCmd(stderr_str.trim().to_string()))
                            .await
                            .ok();
                    }
                }

                if !has_stdout && !has_stderr {
                    self.state
                        .add_message("bash command produced no output.".to_string());
                }
            }
            Err(e) => {
                self.state
                    .add_message(format!("Failed to run bash command: {}", e));
            }
        }
    }

    fn parse_flag<'a>(&self, parts: &'a [&str], flag: &str) -> Option<&'a str> {
        parts.windows(2).find(|w| w[0] == flag).map(|w| w[1])
    }

    async fn handle_connect_command(&mut self, parts: &[&str]) {
        if let Some(ip_addr) = self.parse_flag(parts, "-ip") {
            self.state
                .add_message(format!("Connecting to {}...", ip_addr));
            self.tui.draw(&self.state).ok();

            if let Some(handle) =
                connect_to_server(ip_addr.to_string(), self.event_tx.clone()).await
            {
                self.network_handle = Some(handle);
                self.state
                    .add_message("Connected successfully!".to_string());
            } else {
                self.state.add_message("Failed to connect.".to_string());
            }
        } else {
            self.state
                .add_message("Usage: !connect -ip <ip_address>".to_string());
        }
    }

    async fn handle_bash_command(&mut self, parts: &[&str]) {
        if parts.len() > 1 {
            let bash_cmd = parts[1..].join(" ");
            self.execute_bash_command(&bash_cmd, false).await;
        } else {
            self.state.add_message("Usage: !bash <command>".to_string());
        }
    }

    async fn handle_command(&mut self, cmd: String) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "!connect" => self.handle_connect_command(&parts).await,
            "!quit" => self.state.running = false,
            "!bash" => self.handle_bash_command(&parts).await,
            _ => self
                .state
                .add_message(format!("Unknown command: {}", parts[0])),
        }
    }

    async fn handle_tcp_message(&mut self, msg: String) -> io::Result<()> {
        if msg.starts_with(">bash") {
            self.state.add_message(format!("Remote Command: {}", msg));
            let parts: Vec<&str> = msg.split_whitespace().collect();
            if parts.len() > 1 {
                let bash_cmd = parts[1..].join(" ");
                self.execute_bash_command(&bash_cmd, true).await;
            }
        } else {
            self.state.add_message(format!("Remote: {}", msg));
        }
        self.state.scroll_offset = 0;
        self.tui.draw(&self.state)
    }

    fn handle_tcp_connected(&mut self, addr: std::net::SocketAddr) -> io::Result<()> {
        self.state
            .add_message(format!("Client connected: {}", addr));
        self.state.scroll_offset = 0;
        self.tui.draw(&self.state)
    }

    pub async fn run(&mut self) -> io::Result<()> {
        while self.state.running {
            tokio::select! {
                Some(handle) = self.network_handle_rx.recv() => {
                    self.network_handle = Some(handle);
                    self.state.add_message("Client connected!".to_string());
                    self.tui.draw(&self.state)?;
                }

                Some(event) = self.event_rx.recv() => {
                    match event {
                        AppEvent::KeyPress(key) => {
                            self.handle_key_event(key).await;
                            self.tui.draw(&self.state)?;
                        }
                        AppEvent::TcpMessage(msg) => {
                            self.handle_tcp_message(msg).await?;
                        }
                        AppEvent::TcpConnected(addr) => {
                            self.handle_tcp_connected(addr)?;
                        }
                        AppEvent::UiTick => {
                            self.tui.draw(&self.state)?;
                        }
                        AppEvent::BashCmd(msg) => {
                            self.state.add_message(format!("bash output: {}", msg.trim()));
                            if let Some(ref handle) = self.network_handle {
                                handle.send(msg).await.ok();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
