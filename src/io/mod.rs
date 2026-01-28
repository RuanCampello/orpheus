//! Async IO operations.

use crate::state::State;
use rspotify::AuthCodePkceSpotify as Spotify;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct Io<'io> {
    spotify: Spotify,
    state: &'io Arc<Mutex<State>>,
}

/// IO events that are layzily sent to the queue
/// of the [IO manager](self::Io) to be asynchronously executed.
pub(crate) enum Event {}

impl<'io> Io<'io> {
    pub fn new(spotify: Spotify, state: &'io Arc<Mutex<State>>) -> Self {
        Self { spotify, state }
    }

    pub async fn handle_event(&mut self, event: Event) {
        match event {}
    }
}
