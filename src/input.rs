use tokio::sync::mpsc;
use ratatui::crossterm::event::{self, Event};
use crate::state::AppEvent;

pub struct InputHandler;

impl InputHandler {
    pub fn spawn(event_tx: mpsc::Sender<AppEvent>) {
        tokio::spawn(async move {
            loop {
                if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                    if let Ok(Event::Key(key)) = event::read() {
                        if event_tx.send(AppEvent::KeyPress(key)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }
}