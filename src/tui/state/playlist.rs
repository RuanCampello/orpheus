use crate::tui::state::search::TableStateExt;
use crate::tui::state::PlaylistState;
use ratatui::widgets::ListState;
use rspotify::model::playlist::{FullPlaylist, SimplifiedPlaylist};

pub struct SelectedPlaylist {
    pub playlist: Option<FullPlaylist>,
    pub state: TableStateExt,
}

pub trait Playable {
    fn get_selected_track_uri<'a>(self) -> Option<String>;
}

impl Playable for &SelectedPlaylist {
    fn get_selected_track_uri<'a>(self) -> Option<String> {
        if let Some(selected_playlist) = &self.playlist {
            let idx = self.state.state.selected().unwrap_or(0);
            if let Some(track) = selected_playlist.tracks.items.get(idx) {
                return track.track.as_ref().map(|uri| uri.uri.to_string());
            }
        }
        None
    }
}

impl PlaylistState {
    pub fn new(playlists: Vec<SimplifiedPlaylist>) -> Self {
        let state = ListState::default().with_selected(Some(0));
        let selected_playlist_state = TableStateExt::new(0);

        Self {
            playlists,
            selected_playlist: SelectedPlaylist {
                playlist: None,
                state: selected_playlist_state,
            },
            state,
            active: false,
            offset: 0,
        }
    }
}
