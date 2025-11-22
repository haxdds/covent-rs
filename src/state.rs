
use ratatui::crossterm::event::KeyEvent;
use std::time::SystemTime;

pub struct AppState {
    pub messages: Vec<Message>,
    pub input_buffer: String,
    pub scroll_offset: usize,
    pub running: bool,
    pub cursor_visible: bool,
}

pub struct Message {
    pub content: String,
    pub timestamp: SystemTime, 
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            scroll_offset: 0,
            running: true,
            cursor_visible: true,
        }
    }
    
    pub fn add_message(&mut self, content: String) {
        self.messages.push(Message {
            content,
            timestamp: SystemTime::now(),
        });
        
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }

        self.scroll_offset = 0;
    }
}

pub enum AppEvent {

    KeyPress(KeyEvent),
    TcpMessage(String),
    TcpConnected(std::net::SocketAddr),
    UiTick,

}