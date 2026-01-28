//! Async IO operations.

pub(crate) mod key;

use crate::state::{State, handler::ViewId};
use rspotify::{
  AuthCodePkceSpotify as Spotify,
  model::{AdditionalType, PlayableItem, PlaylistId},
  prelude::{BaseClient, OAuthClient},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) struct Io<'io> {
  spotify: Spotify,
  state: &'io Arc<Mutex<State>>,
}

/// IO events that are layzily sent to the queue
/// of the [IO manager](self::Io) to be asynchronously executed.
#[allow(unused)]
pub(crate) enum Event {
  /// Get the current logged user's playlists.
  UserPlaylists,
  /// Get the current playback state.
  GetCurrentPlayback,
  /// Get the given playlist tracks.
  PlaylistTracks(PlaylistId<'static>, u32),

  Seek(u32),
  NextTrack,
}

#[allow(unused)]
impl<'io> Io<'io> {
  pub fn new(spotify: Spotify, state: &'io Arc<Mutex<State>>) -> Self {
    Self { spotify, state }
  }

  pub async fn handle_event(&mut self, event: Event) {
    match event {
      Event::UserPlaylists => self.current_user_playlists().await,
      Event::GetCurrentPlayback => self.current_playback().await,
      Event::PlaylistTracks(id, offset) => self.playlist_tracks(id, offset).await,

      Event::Seek(_) => todo!(),
      Event::NextTrack => todo!(),
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

  async fn current_playback(&mut self) {
    let context = self
      .spotify
      .current_playback(
        None,
        Some(vec![&AdditionalType::Episode, &AdditionalType::Track]),
      )
      .await;

    match context {
      Ok(Some(context)) => {
        let mut state = self.state.lock().await;
        state.current_playback_context = Some(context.clone());

        match context.item {
          Some(PlayableItem::Track(_)) => {}
          Some(PlayableItem::Episode(_)) => {}
          _ => {}
        }
      }

      Ok(None) => {}

      Err(_) => unreachable!(),
    }
  }

  async fn playlist_tracks(&mut self, id: PlaylistId<'_>, offset: u32) {
    if let Ok(tracks) = self
      .spotify
      .playlist_items_manual(id, None, None, None, Some(offset))
      .await
    {
      let mut state = self.state.lock().await;
      state.playlist_tracks = Some(tracks);
      // TODO: push the view to stack
    };
  }

  async fn seek(&mut self, ms: u32) {
    todo!()
  }
}
