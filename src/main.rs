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

    let (sender, receiver) = channel::<Event>();

    let config = Config::default();
    let state = Arc::new(Mutex::new(State::new(config, sender)));
    let outer_state = state.clone();

    std::thread::spawn(move || {
        let mut io = Io::new(spotify, &state);
        start(receiver, &mut io);
    });

    terminal::start(&outer_state).await.unwrap();
}

#[tokio::main]
async fn start<'io>(receiver: Receiver<Event>, io: &mut Io) {
    while let Ok(event) = receiver.recv() {
        io.handle_event(event).await
    }
}
