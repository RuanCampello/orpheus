mod auth;
mod config;
mod io;
mod state;
mod terminal;
mod ui;

use crate::{
  config::Config,
  io::{Event, Io},
  state::State,
};
use std::sync::{
  Arc,
  mpsc::{Receiver, channel},
};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  Terminal(#[from] terminal::TerminalError),
}

#[tokio::main]
async fn main() {
  let spotify = match auth::authenticate().await {
    Ok(client) => client,
    Err(e) => {
      eprintln!("Authentication failed: {}", e);
      std::process::exit(1);
    }
  };

  let file = tracing_appender::rolling::hourly("./logs", "tui.log");
  let (non_blocking, _guard) = tracing_appender::non_blocking(file);
  let subscriber = tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .with_ansi(false)
    .finish();

  tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set default subscriber for tracing");

  let (sender, receiver) = channel::<Event>();

  let config = Config::default();
  let state = Arc::new(Mutex::new(State::new(config, sender)));
  let outer_state = state.clone();

  std::thread::spawn(move || {
    let mut io = Io::new(spotify, &state);
    start(receiver, &mut io);
  });

  terminal::start(&config, &outer_state).await.unwrap();
}

#[tokio::main]
async fn start<'io>(receiver: Receiver<Event>, io: &mut Io) {
  while let Ok(event) = receiver.recv() {
    io.handle_event(event).await
  }
}
