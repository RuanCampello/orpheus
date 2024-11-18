use crate::tui::state::search::ResultItem;
use crate::tui::state::PlaylistState;
use crate::tui::State;
use ratatui::crossterm::event::KeyCode;
use rspotify::model::page::Page;

pub(super) trait Navigable {
    fn next(&mut self);
    fn previous(&mut self);
    fn toggle_active(&mut self);
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

impl State {
    pub(super) async fn handle_key(&mut self, key: KeyCode) {
        let on_playlist_sidebar = self.playlist_state.active;

        match key {
            // navigation keys
            KeyCode::Up | KeyCode::Down | KeyCode::Enter if on_playlist_sidebar => {
                self.navigate(key).await;
            }
            KeyCode::Left | KeyCode::Right if !on_playlist_sidebar => {
                self.control_playlist(key).await;
            }

            // character-specific actions
            KeyCode::Char(c) => self.handle_character(c),

            // search-specific actions
            KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace => {
                self.handle_search_control(key).await;
            }

            _ => {}
        }
    }

    fn handle_character(&mut self, c: char) {
        match self.search_state.active {
            false => match c {
                '1' => self.playlist_state.toggle_active(),
                's' => Self::toggle_active_state(&mut self.search_state.results.songs),
                'a' => Self::toggle_active_state(&mut self.search_state.results.albums),
                'd' => Self::toggle_active_state(&mut self.search_state.results.artists),
                'q' => self.should_quit = true,
                'e' => self.search_state.active = !self.search_state.active,
                _ => {}
            },
            true => self.search_state.handle_char(c),
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
            }
            KeyCode::Enter => self.play_selected_track().await,
            _ => {}
        }
    }

    /// Handles navigation on playlist page.
    async fn control_playlist(&mut self, key: KeyCode) {
        if let Some(selected_playlist) = &mut self.playlist_state.selected_playlist {
            let uri = &selected_playlist.uri;
            let offset_step = self.window.height.saturating_sub(8) as u32;
            // println!("{offset_step}");

            match key {
                KeyCode::Right => {
                    self.playlist_state.offset += offset_step;

                    if let Ok(playlist) = self
                        .client
                        .spotify
                        .user_playlist_tracks(
                            "spotify",
                            uri,
                            None,
                            None,
                            Some(self.playlist_state.offset),
                            None,
                        )
                        .await
                    {
                        selected_playlist.tracks = playlist;
                    }
                }
                KeyCode::Left => {
                    self.playlist_state.offset =
                        self.playlist_state.offset.saturating_sub(offset_step);

                    if let Ok(playlist) = self
                        .client
                        .spotify
                        .user_playlist_tracks(
                            "spotify",
                            uri,
                            None,
                            None,
                            Some(self.playlist_state.offset),
                            None,
                        )
                        .await
                    {
                        selected_playlist.tracks = playlist;
                    }
                }
                _ => {}
            }
        }
    }

    async fn navigate(&mut self, key: KeyCode) {
        if self.playlist_state.active {
            match key {
                KeyCode::Enter => {
                    if let Some(id) = self.playlist_state.state.selected() {
                        let uri = self.playlist_state.playlists[id].uri.as_ref();

                        let selected_playlist =
                            match self.client.spotify.playlist(uri, None, None).await {
                                Ok(playlist) => Some(playlist),
                                Err(_) => None,
                            };

                        self.playlist_state.selected_playlist = selected_playlist;
                        self.playlist_state.active = false;
                    }
                }
                _ => Self::update_navigation(&mut self.playlist_state, key),
            }
        }
        if let Some(songs) = &mut self.search_state.results.songs {
            if songs.table_state.active {
                Self::update_navigation(&mut songs.table_state, key);
            }
        }
        if let Some(albums) = &mut self.search_state.results.albums {
            if albums.table_state.active {
                Self::update_navigation(&mut albums.table_state, key);
            }
        }
        if let Some(artists) = &mut self.search_state.results.artists {
            if artists.table_state.active {
                Self::update_navigation(&mut artists.table_state, key);
            }
        }
    }

    fn update_navigation<T: Navigable>(navigable: &mut T, key: KeyCode) {
        match key {
            KeyCode::Up => navigable.previous(),
            KeyCode::Down => navigable.next(),
            _ => {}
        }
    }

    fn toggle_active_state<T>(state: &mut Option<ResultItem<Page<T>>>) {
        if let Some(result_item) = state {
            result_item.table_state.toggle_active();
        }
    }
}
