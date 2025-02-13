use crate::tui::keyboard::Navigable;
use crate::tui::state::search::TableStateExt;
use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use rspotify::model::page::Page;
use rspotify::model::playlist::{FullPlaylist, PlaylistTrack, SimplifiedPlaylist};

pub(in crate::tui) struct PlaylistState {
    pub active: bool,
    pub offset: u32,
    pub offset_step: u32,
    pub state: ListState,

    pub selected_playlist: SelectedPlaylist,
    pub playlists: Vec<SimplifiedPlaylist>,
}

pub(in crate::tui) struct SelectedPlaylist {
    pub playlist: Option<FullPlaylist>,
    pub state: TableStateExt,
}

impl AsRef<PlaylistState> for PlaylistState {
    fn as_ref(&self) -> &PlaylistState {
        self
    }
}

pub trait Playable {
    /// Returns the selected song, its index where was selected and its possible playlist identifier.
    fn get_selected_track_uri(&self) -> (Option<String>, Option<usize>, Option<String>);
}

impl Playable for &PlaylistState {
    fn get_selected_track_uri(&self) -> (Option<String>, Option<usize>, Option<String>) {
        let selected_playlist = self.selected_playlist.playlist.as_ref();
        let idx = self.selected_playlist.state.state.selected().unwrap_or(0);
        let identifier = selected_playlist.map(|playlist| playlist.uri.to_string());

        let uri = selected_playlist
            .and_then(|playlist| playlist.tracks.items.get(idx))
            .and_then(|playlist_track| playlist_track.track.as_ref())
            .map(|track| track.uri.to_string());

        (uri, Some(idx), identifier)
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

    /// Returns the selected playlist uri on the sidebar if some.
    pub fn selected_playlist_uri(&self) -> Option<String> {
        let idx = self.state.selected().unwrap_or(0);
        let playlist = self.playlists.get(idx)?.uri.to_string();

        Some(playlist)
    }

    /// Handles the playlist page horizontal navigation.
    pub fn handle_navigation(&mut self, key: KeyCode) {
        let offset_step = self.offset_step;
        #[rustfmt::skip]
        let length = self.selected_playlist.playlist.as_ref().unwrap().tracks.items.len() as u32;

        match key {
            #[rustfmt::skip]
            KeyCode::Right if offset_step <= length => self.offset += offset_step,
            KeyCode::Left => self.offset = self.offset.saturating_sub(offset_step),
            _ => {}
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

    fn set_active(&mut self, active: bool) {
        self.active = active;
        self.selected_playlist.state.active = !active;
    }
}
