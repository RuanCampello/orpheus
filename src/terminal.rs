//! Terminal related code.

use crate::{
  config::Config,
  io::{
    Event,
    key::{self, EventHandler, Key},
  },
  state::{State, handler},
  ui::draw,
};
use ratatui::{
  Terminal,
  crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
      EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
    },
  },
  prelude::CrosstermBackend,
};
use std::{io::stdout, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum TerminalError {
  #[error("Io error from terminal execution: {0}")]
  Io(#[from] std::io::Error),
  #[error("Receiver channel error from key handler: {0}")]
  Recv(#[from] std::sync::mpsc::RecvError),
}

pub(crate) async fn start(config: &Config, state: &Arc<Mutex<State>>) -> Result<(), TerminalError> {
  let mut out = stdout();
  execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
  enable_raw_mode()?;

  let mut backend = CrosstermBackend::new(out);
  backend.execute(SetTitle("orpheus"))?;

  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;

  let event_handler = EventHandler::new(config.tick_rate.as_millis() as _);
  println!("{duration}", duration = config.tick_rate.as_millis() as u64);

  let mut is_first_render = true;

  loop {
    let mut state = state.lock().await;

    terminal.draw(|mut f| draw(&mut f, &state))?;

    match event_handler.next()? {
      // TODO: only quit if not in the search input
      key::Event::Input(key) => {
        if key == Key::Char('q') {
          break;
        };

        handler::handle(key, &mut state);
      }

      _ => {}
    }

    if is_first_render {
      state.dispatch(Event::UserPlaylists);

      is_first_render = false;
    }
  }

  disable_raw_mode()?;
  let mut out = stdout();
  execute!(out, LeaveAlternateScreen, DisableMouseCapture)?;
  Ok(())
}
