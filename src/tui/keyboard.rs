use crate::tui::state::playlist::Playable;
use crate::tui::state::search::ActiveResult;
use crate::tui::state::{Tab, VolumeAction};
use crate::tui::State;
use ratatui::crossterm::event::KeyCode;

pub(super) trait Navigable {
    fn next(&mut self);
    fn previous(&mut self);
    fn set_active(&mut self, active: bool);
}

impl State {
    pub(super) async fn handle_key(&mut self, key: KeyCode) {
        let on_playlist_page =
            self.tab.eq(&Tab::PlaylistPage) && self.playlist_state.selected_playlist.state.active;
        let on_search_page = self.tab.eq(&Tab::SearchResults) || self.playlist_state.active;

        match key {
            // search/playlist navigation
            KeyCode::Up | KeyCode::Down | KeyCode::Enter if on_search_page => {
                self.navigate(key).await;
            }

            // playlist page navigation
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Enter
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Esc
                if on_playlist_page =>
            {
                match key {
                    KeyCode::Enter => {
                        let uri = self.playlist_state.get_selected_track_uri();
                        self.play_selected_track(uri).await;
                    }
                    KeyCode::Left | KeyCode::Right => {
                        self.playlist_state.handle_navigation(key);
                        self.new_playlist_page().await;
                    }
                    KeyCode::Up | KeyCode::Down => {
                        Self::update_navigation(
                            &mut self.playlist_state.selected_playlist.state,
                            key,
                        );
                    }
                    KeyCode::Esc => self.playlist_state.selected_playlist.state.active = false,
                    _ => unreachable!(),
                }
            }

            // character-specific actions
            KeyCode::Char(c) => self.handle_character(c).await,

            // search-specific actions
            KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace => {
                self.handle_search_control(key).await;
            }

            _ => {}
        }
    }

    async fn handle_character(&mut self, c: char) {
        let search_state = &mut self.search_state;

        match search_state.active {
            false => match c {
                '1' => {
                    search_state.set_active(ActiveResult::None);
                    self.playlist_state.set_active(!self.playlist_state.active)
                }
                's' => search_state.set_active(ActiveResult::Songs),
                // 'a' => search_state.set_active(ActiveResult::Albums),
                'd' => search_state.set_active(ActiveResult::Artists),
                'e' => {
                    search_state.set_active(ActiveResult::None);
                    search_state.active = !search_state.active
                }
                '+' => self.update_volume(VolumeAction::Increase).await,
                '-' => self.update_volume(VolumeAction::Decrease).await,
                'q' => self.should_quit = true,
                _ => {}
            },
            true => search_state.handle_char(c),
        }
    }

    async fn handle_search_control(&mut self, key: KeyCode) {
        let active = self.search_state.active;

        match key {
            KeyCode::Esc | KeyCode::Backspace if active => self.search_state.update(key),
            KeyCode::Enter if active => {
                self.search().await;

                self.search_state.active = false;
                if let Some(songs) = &mut self.search_state.results.songs {
                    songs.table_state.active = true
                }

                self.tab = Tab::SearchResults;
            }
            _ => {}
        }
    }

    /// Updates the playlist page based on offset and offset step.
    async fn new_playlist_page(&mut self) {
        let Some(uri) = self.playlist_state.selected_playlist_uri() else {
            return;
        };

        if let Ok(playlist) = self
            .client
            .spotify
            .user_playlist_tracks(
                "spotify",
                uri,
                None,
                Some(self.playlist_state.offset_step),
                Some(self.playlist_state.offset),
                None,
            )
            .await
        {
            self.playlist_state.update_tracks(playlist);
        }
    }

    /// Handles the playlist sidebar and the search results navigation.
    async fn navigate(&mut self, key: KeyCode) {
        if self.playlist_state.active {
            match key {
                KeyCode::Enter => {
                    let Some(uri) = self.playlist_state.selected_playlist_uri() else {
                        return;
                    };

                    let selected_playlist = self
                        .client
                        .spotify
                        .playlist(uri, None, None)
                        .await
                        .map(Some)
                        .unwrap_or(None);

                    self.playlist_state.update(selected_playlist);
                    self.tab = Tab::PlaylistPage
                }
                _ => Self::update_navigation(&mut self.playlist_state, key),
            }
        }

        if let Some(songs) = &mut self.search_state.results.songs {
            if !songs.table_state.active {
                return;
            }

            match key {
                KeyCode::Enter => {
                    let uri = (&*songs).get_selected_track_uri();
                    self.play_selected_track(uri).await;
                }
                _ => Self::update_navigation(&mut songs.table_state, key),
            }
        } 
        // else if let Some(albums) = &mut self.search_state.results.albums {
        //     if albums.table_state.active {
        //         Self::update_navigation(&mut albums.table_state, key);
        //     }
        // }
        else if let Some(artists) = &mut self.search_state.results.artists {
            if artists.table_state.active {
                Self::update_navigation(&mut artists.table_state, key);
            }
        }
    }

    /// Handles `Navigable` vertical navigation.
    fn update_navigation<T: Navigable>(navigable: &mut T, key: KeyCode) {
        match key {
            KeyCode::Up => navigable.previous(),
            KeyCode::Down => navigable.next(),
            _ => {}
        }
    }
}
