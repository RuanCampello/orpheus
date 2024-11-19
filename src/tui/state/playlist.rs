use crate::tui::keyboard::Navigable;
use crate::tui::state::search::TableStateExt;
use crate::tui::state::PlaylistState;
use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use rspotify::model::page::Page;
use rspotify::model::playlist::{FullPlaylist, PlaylistTrack, SimplifiedPlaylist};

pub(in crate::tui::state) struct SelectedPlaylist {
    pub playlist: Option<FullPlaylist>,
    pub state: TableStateExt,
}

pub trait Playable {
    fn get_selected_track_uri(&self) -> Option<String>;
}

impl Playable for PlaylistState {
    fn get_selected_track_uri(&self) -> Option<String> {
        let selected_playlist = self.selected_playlist.playlist.as_ref()?;
        let idx = self.selected_playlist.state.state.selected().unwrap_or(0);
        let uri = selected_playlist
            .tracks
            .items
            .get(idx)?
            .track
            .as_ref()
            .map(|track| track.uri.to_string());

        uri
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
            offset_step: 0,
        }
    }

    pub fn update_tracks(&mut self, new_tracks: Page<PlaylistTrack>) {
        let Some(playlist) = &mut self.selected_playlist.playlist else {
            return;
        };

        playlist.tracks = new_tracks;
    }

    pub fn update(&mut self, new_playlist: Option<FullPlaylist>) {
        let size = new_playlist
            .as_ref()
            .map(|playlist| playlist.tracks.items.len())
            .unwrap_or_default();

        self.selected_playlist.playlist = new_playlist;
        self.active = false;
        self.selected_playlist.state.max_size = size;
    }

    /// Returns the selected playlist on the sidebar if some.
    pub fn selected_playlist_uri(&self) -> Option<&str> {
        let idx = self.state.selected().unwrap_or(0);
        let playlist = &self.playlists.get(idx)?.uri;

        Some(playlist)
    }

    /// Handles the playlist page horizontal navigation.
    pub fn handle_navigation(&mut self, key: KeyCode) {
        let offset_step = self.offset_step;

        match key {
            KeyCode::Right => {
                self.offset += offset_step;
            }
            KeyCode::Left => {
                self.offset = self.offset.saturating_sub(offset_step);
            }
            _ => unreachable!(),
        }
    }
}

impl Navigable for PlaylistState {
    fn next(&mut self) {
        let i = self
            .state
            .selected()
            .unwrap_or(usize::MAX)
            .saturating_add(1)
            % self.playlists.len();
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = self.state.selected().unwrap_or(0).saturating_sub(1) % self.playlists.len();
        self.state.select(Some(i));
    }

    fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}
