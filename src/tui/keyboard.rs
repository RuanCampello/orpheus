use crate::tui::{PlaylistState, State};
use ratatui::crossterm::event::KeyCode;

enum KeyAction {
    Navigation,
    Character(char),
}

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
        if let Some(action) = self.determine_key_action(key) {
            match action {
                KeyAction::Navigation => self.handle_navigation(key),
                KeyAction::Character(c) => self.handle_character(c),
            }
        }
    }

    fn determine_key_action(&self, key: KeyCode) -> Option<KeyAction> {
        match key {
            KeyCode::Up | KeyCode::Down | KeyCode::Esc => Some(KeyAction::Navigation),
            KeyCode::Char(c) => Some(KeyAction::Character(c)),
            _ => None,
        }
    }

    fn handle_navigation(&mut self, key: KeyCode) {
        if self.playlist_state.active {
            self.playlist_state.update(key);
        }
    }

    fn handle_character(&mut self, c: char) {
        match c {
            '1' => self.playlist_state.update(KeyCode::Char(c)),
            _ => {}
        }
    }
}
