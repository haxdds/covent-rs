use ratatui::crossterm::event::KeyEvent;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use std::collections::VecDeque;
use std::time::SystemTime;
use tui_textarea::TextArea;

pub struct AppState {
    pub messages: VecDeque<Message>,
    pub scroll_offset: usize,
    pub running: bool,
    pub textarea: TextArea<'static>,
}

pub struct Message {
    pub content: String,
    pub timestamp: SystemTime,
}

pub fn get_default_textarea() -> TextArea<'static> {
    let mut textarea = TextArea::from(vec![String::new()]);
    textarea.set_style(Style::default().fg(Color::White));
    textarea.set_cursor_style(
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::UNDERLINED)
            .add_modifier(Modifier::RAPID_BLINK),
    );
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::LightRed)
            .title(" input (!quit to exit) "),
    );
    textarea
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            scroll_offset: 0,
            running: true,
            textarea: get_default_textarea(),
        }
    }

    pub fn add_message(&mut self, content: String) {
        self.messages.push_back(Message {
            content,
            timestamp: SystemTime::now(),
        });

        if self.messages.len() > 100 {
            self.messages.pop_front();
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
