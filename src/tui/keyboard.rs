use crate::tui::state::PlaylistState;
use crate::tui::State;
use ratatui::crossterm::event::KeyCode;

enum KeyAction {
    SearchControl,
    Navigation,
    Character(char),
}

pub(super) trait Navigable {
    fn next(&mut self);
    fn previous(&mut self);
    fn update(&mut self, key: KeyCode);
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

    fn update(&mut self, key: KeyCode) {
        if self.active {
            match key {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Esc | KeyCode::Char('1') => self.active = false,
                _ => {}
            }
        } else if let KeyCode::Char('1') = key {
            self.active = true
        }
    }
}

impl State {
    pub(super) async fn on_key(&mut self, k: KeyCode) {
        if let Some(action) = self.determine_key_action(k) {
            match action {
                KeyAction::SearchControl => self.handle_search_control(k).await,
                KeyAction::Navigation => self.handle_navigation(k),
                KeyAction::Character(c) => self.on_char(c),
            }
        }
    }

    fn determine_key_action(&self, k: KeyCode) -> Option<KeyAction> {
        match k {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace if self.search_state.active => {
                Some(KeyAction::SearchControl)
            }
            KeyCode::Down | KeyCode::Up | KeyCode::Right | KeyCode::Left => {
                Some(KeyAction::Navigation)
            }
            KeyCode::Char(c) => Some(KeyAction::Character(c)),
            _ => None,
        }
    }

    async fn handle_search_control(&mut self, k: KeyCode) {
        match k {
            KeyCode::Esc | KeyCode::Backspace => self.search_state.update(k),
            KeyCode::Enter => self.search().await,
            _ => {}
        }
    }

    fn handle_navigation(&mut self, k: KeyCode) {
        if !self.is_navigation_key(k) {
            return;
        }

        if self.playlist_state.active {
            self.playlist_state.update(k);
        }
        if let Some(songs) = &mut self.search_state.results.songs {
            if songs.table_state.active {
                songs.table_state.update(k)
            }
        }
        if let Some(albums) = &mut self.search_state.results.albums {
            if albums.table_state.active {
                albums.table_state.update(k)
            }
        }
        if let Some(artists) = &mut self.search_state.results.artists {
            if artists.table_state.active {
                artists.table_state.update(k)
            }
        }
    }

    fn on_char(&mut self, c: char) {
        match c {
            '1' => self.playlist_state.update(KeyCode::Char(c)),
            'q' => match self.search_state.active {
                true => self.search_state.handle_char(c),
                _ => self.should_quit = true,
            },
            's' => match &mut self.search_state.results.songs {
                Some(songs) => {
                    songs.table_state.active = !songs.table_state.active;
                }
                None => self.search_state.handle_char(c),
            },
            'a' => match &mut self.search_state.results.albums {
                Some(albums) => albums.table_state.active = !albums.table_state.active,
                None => self.search_state.handle_char(c),
            },
            'd' => match &mut self.search_state.results.artists {
                Some(artists) => artists.table_state.active = !artists.table_state.active,
                None => self.search_state.handle_char(c),
            },
            _ => self.search_state.handle_char(c),
        }
    }

    fn is_navigation_key(&self, k: KeyCode) -> bool {
        matches!(k, KeyCode::Down | KeyCode::Up)
    }
}
