use crate::state::AppEvent;
use ratatui::crossterm::event::{self, Event};
use tokio::sync::mpsc;

pub fn spawn_input_handler(event_tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        loop {
            match event::poll(std::time::Duration::from_millis(100)) {
                Ok(true) => {
                    if let Ok(Event::Key(key)) = event::read()
                        && event_tx.send(AppEvent::KeyPress(key)).await.is_err()
                    {
                        break;
                    }
                }
                Ok(false) => continue,
                Err(_) => break,
            }
        }
    });
}
