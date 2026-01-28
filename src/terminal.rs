//! Terminal related code.

use crate::{io::Event, state::State};
use ratatui::{
    Terminal,
    crossterm::{
        ExecutableCommand,
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
        execute,
        terminal::{
            EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
        },
    },
    prelude::CrosstermBackend,
};
use std::{
    io::stdout,
    sync::{Arc, mpsc::Receiver},
};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub(crate) enum TerminalError {
    #[error("Io error from terminal execution: {0}")]
    Io(#[from] std::io::Error),
}

pub(crate) async fn start(state: &Arc<Mutex<State>>) -> Result<(), TerminalError> {
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut backend = CrosstermBackend::new(out);
    backend.execute(SetTitle("orpheus"))?;

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    loop {
        let state = state.lock().await;

        terminal.draw(|mut f| todo!("draw callbacks"))?;

        // TODO: add event handler channel
        if event::poll(state.config.tick_rate)? {
            match event::read()? {
                event::Event::Key(key) if key.code == KeyCode::Char('q') => {
                    break;
                }
                _ => continue,
            }
        }
    }

    disable_raw_mode()?;
    let mut out = stdout();
    execute!(out, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
