
use crossterm::event::KeyEvent;
use std::time::SystemTime;

// ============================================================================
// STATE
// ============================================================================
pub struct AppState {
    pub messages: Vec<Message>,
    pub input_buffer: String,
    pub scroll_offset: usize,
    pub running: bool,
    pub cursor_visible: bool,
    pub cursor_tick_count: u32, // Track ticks for blinking
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
            cursor_tick_count: 0,
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

    pub fn tick_cursor(&mut self) {
        self.cursor_tick_count += 1;
        // Toggle cursor every ~500ms (assuming UiTick is ~16ms, that's ~31 ticks)
        if self.cursor_tick_count >= 31 {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_tick_count = 0;
        }
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