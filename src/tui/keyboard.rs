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
        match k {
            KeyCode::Down | KeyCode::Up if self.playlist_state.active => {
                self.playlist_state.update(k)
            }
            KeyCode::Down | KeyCode::Up if self.search_state.results.songs.is_some() => {
                if let Some(songs) = &mut self.search_state.results.songs {
                    songs.state.update(k);
                }
            }
            _ => {}
        }
    }

    fn on_char(&mut self, c: char) {
        match c {
            '1' => self.playlist_state.update(KeyCode::Char(c)),
            'q' => match self.search_state.active {
                true => self.search_state.handle_char(c),
                _ => self.should_quit = true,
            },
            _ => self.search_state.handle_char(c),
        }
    }
}
