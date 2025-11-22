
use ratatui::crossterm::event::KeyEvent;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Borders, Block};
use tui_textarea::TextArea;
use std::time::SystemTime;

pub struct AppState {
    pub messages: Vec<Message>,
    pub scroll_offset: usize,
    pub running: bool,
    pub textarea: TextArea<'static>
}

pub struct Message {
    pub content: String,
    pub timestamp: SystemTime, 
}

pub fn get_default_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::from(vec![String::new()]);
    // Apply styling
    textarea.set_style(Style::default().fg(Color::White));
    // Create block with yellow styling applied to it
    textarea.set_cursor_style(Style::default().fg(Color::Red).add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::RAPID_BLINK));
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::LightRed)
            .title(" input (!quit to exit) ")
    );
    
    textarea
}

impl AppState {
    pub fn new() -> Self {

        let textarea = get_default_textarea();
      
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            running: true,
            textarea
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
    BashCmd(String),

}