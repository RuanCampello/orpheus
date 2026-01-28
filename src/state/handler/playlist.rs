use crate::{
  io::{Event, key::Key},
  state::{State, handler::select_handler},
};

pub fn handler(key: Key, state: &mut State) {
  match key {
    Key::Down => {
      if let Some(page) = &state.playlists {
        if let Some(playlist_index) = state.selected_playlist_index {
          let next = select_handler(&page.items, Some(playlist_index));
          state.selected_playlist_index = Some(playlist_index);
        }
      }
    }

    Key::Enter => {
      if let (Some(playlists), Some(playlist_index)) =
        (&state.playlists, &state.selected_playlist_index)
      {
        if let Some(playlist) = playlists.items.get(*playlist_index) {
          let id = playlist.id.to_owned();
          // TODO: add playlist offset to state
          state.dispatch(Event::PlaylistTracks(id, 0));
        }
      }
    }

    _ => {}
  }
}
