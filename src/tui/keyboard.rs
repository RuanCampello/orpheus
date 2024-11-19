use crate::tui::state::playlist::Playable;
use crate::tui::state::search::ResultItem;
use crate::tui::State;
use ratatui::crossterm::event::KeyCode;
use rspotify::model::page::Page;

pub(super) trait Navigable {
    fn next(&mut self);
    fn previous(&mut self);
    fn toggle_active(&mut self);
}

impl State {
    pub(super) async fn handle_key(&mut self, key: KeyCode) {
        let on_playlist_sidebar = self.playlist_state.active;

        match key {
            // search/playlist navigation
            KeyCode::Up | KeyCode::Down | KeyCode::Enter if on_playlist_sidebar => {
                self.navigate(key).await;
            }

            // playlist page navigation
            KeyCode::Up | KeyCode::Down | KeyCode::Enter | KeyCode::Left | KeyCode::Right
                if self.playlist_state.selected_playlist.playlist.is_some() =>
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
                    _ => unreachable!(),
                }
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

                    self.playlist_state.update(selected_playlist)
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

    /// Handles `Navigable` vertical navigation.
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
