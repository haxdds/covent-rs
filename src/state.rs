
use crossterm::event::KeyEvent;
use std::time::SystemTime;

// ============================================================================
// STATE
// ============================================================================
pub struct AppState {
    pub messages: Vec<Message>,
    pub input_buffer: String,
    pub scroll_offset: usize,
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

// ============================================================================
// EVENTS
// ============================================================================

pub enum AppEvent {
    // User pressed a key
    KeyPress(KeyEvent),
    // Message from TCP
    TcpMessage(String),
    // Client connected
    TcpConnected(std::net::SocketAddr),
    // Timer tick for redrawing UI
    UiTick,
}