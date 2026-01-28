//! Async IO operations.

use crate::state::State;
use rspotify::{AuthCodePkceSpotify as Spotify, prelude::OAuthClient};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct Io<'io> {
    spotify: Spotify,
    state: &'io Arc<Mutex<State>>,
}

/// IO events that are layzily sent to the queue
/// of the [IO manager](self::Io) to be asynchronously executed.
pub(crate) enum Event {
    /// Get the current logged user's playlists.
    UserPlaylists,
}

impl<'io> Io<'io> {
    pub fn new(spotify: Spotify, state: &'io Arc<Mutex<State>>) -> Self {
        Self { spotify, state }
    }

    pub async fn handle_event(&mut self, event: Event) {
        match event {
            Event::UserPlaylists => self.current_user_playlists().await,
        }
    }

    async fn current_user_playlists(&mut self) {
        let playlists = self
            .spotify
            .current_user_playlists_manual(Some(25), None)
            .await;

        // TODO: handle those errors
        match playlists {
            Ok(playlists) => {
                let mut state = self.state.lock().await;
                state.playlists = Some(playlists);
            }
            _ => unreachable!(),
        }
    }
}
