use crate::tui::state::PlaylistState;
use crate::tui::State;
use ratatui::crossterm::event::KeyCode;

trait Navigable {
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
    pub(super) fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => self.handle_character(c),
            _ => self.handle_navigation(key),
        }
    }

    fn handle_navigation(&mut self, key: KeyCode) {
        if self.playlist_state.active {
            self.playlist_state.update(key);
        } else if self.search_state.active {
            self.search_state.update(key);
        };
    }

    fn handle_character(&mut self, c: char) {
        match c {
            '1' => self.playlist_state.update(KeyCode::Char(c)),
            'q' => match self.search_state.active {
                true => self.search_state.insert_char(c),
                false => self.should_quit = true,
            },
            'e' => self.search_state.active = true,
            _ => {
                if self.search_state.active {
                    self.search_state.insert_char(c)
                }
            }
        }
    }
}
