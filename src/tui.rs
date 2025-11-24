use crate::state::{AppEvent, AppState};
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::io;
use tokio::sync::mpsc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        Ok(Self { terminal })
    }

    pub fn draw(&mut self, state: &AppState) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(frame.area());

            let messages: Vec<ListItem> = state
                .messages
                .iter()
                .map(|m| {
                    let color = if m.content.starts_with("You:") || m.content.starts_with("Remote:")
                    {
                        Color::White
                    } else {
                        Color::LightRed
                    };
                    ListItem::new(format!(
                        "[{}] {}",
                        chrono::DateTime::<chrono::Local>::from(m.timestamp).format("%H:%M:%S"),
                        m.content
                    ))
                    .style(Style::default().fg(color))
                })
                .collect();

            let messages_widget = List::new(messages).block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Color::LightGreen)
                    .title(format!(
                        " covent v{} [{}] ",
                        VERSION,
                        chrono::Local::now().format("%H:%M:%S")
                    )),
            );

            let available_height = chunks[0].height.saturating_sub(2) as usize;
            let total_messages = state.messages.len();

            let actual_offset = if total_messages > available_height {
                total_messages.saturating_sub(available_height + state.scroll_offset)
            } else {
                0
            };

            let mut list_state = ratatui::widgets::ListState::default();
            list_state = list_state.with_offset(actual_offset);
            frame.render_stateful_widget(messages_widget, chunks[0], &mut list_state);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .style(Style::default().fg(Color::Cyan));

            let mut scrollbar_state = ScrollbarState::new(total_messages).position(
                total_messages
                    .saturating_sub(state.scroll_offset)
                    .saturating_sub(1),
            );

            frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);

            frame.render_widget(&state.textarea, chunks[1]);
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
    }
}

pub fn spawn_ui_ticker(event_tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
            if event_tx.send(AppEvent::UiTick).await.is_err() {
                break;
            }
        }
    });
}
