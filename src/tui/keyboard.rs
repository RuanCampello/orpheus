use crate::tui::state::search::ResultItem;
use crate::tui::state::PlaylistState;
use crate::tui::State;
use ratatui::crossterm::event::KeyCode;
use rspotify::model::page::Page;

enum KeyAction {
    SearchControl,
    Navigation,
    Character(char),
}

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
        match key {
            // navigation keys
            KeyCode::Up | KeyCode::Down => self.navigate(key),

            // character-specific actions
            KeyCode::Char(c) => self.handle_character(c),

            // search-specific actions
            KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace => {
                if self.search_state.active {
                    self.handle_search_control(key).await;
                }
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
        match key {
            KeyCode::Esc | KeyCode::Backspace => self.search_state.update(key),
            KeyCode::Enter => self.search().await,
            _ => {}
        }
    }

    fn navigate(&mut self, key: KeyCode) {
        if self.playlist_state.active {
            Self::update_navigation(&mut self.playlist_state, key);
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
