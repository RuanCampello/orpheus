use crate::tui::keyboard::Navigable;
use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::TableState;
use rspotify::model::album::SimplifiedAlbum;
use rspotify::model::artist::FullArtist;
use rspotify::model::page::Page;
use rspotify::model::track::FullTrack;

pub(in crate::tui) struct ResultItem<T> {
    pub data: T,
    pub table_state: TableStateExt,
}

pub(in crate::tui) struct SearchResult {
    pub artists: Option<ResultItem<Page<FullArtist>>>,
    pub songs: Option<ResultItem<Page<FullTrack>>>,
    pub albums: Option<ResultItem<Page<SimplifiedAlbum>>>,
}

pub(in crate::tui) struct SearchState {
    pub results: SearchResult,
    pub input: String,
    pub active: bool,
    pub cursor_position: usize,
}

pub(in crate::tui) struct TableStateExt {
    pub state: TableState,
    pub max_size: usize,
    pub active: bool,
}

impl TableStateExt {
    pub fn new(max_size: usize) -> Self {
        let state = TableState::default().with_selected(0);
        Self { state, max_size, active: false }
    }
}

impl Navigable for TableStateExt {
    fn next(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| (i.saturating_add(1)) % self.max_size);

        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = self
            .state
            .selected()
            .map_or(0, |i| (i.saturating_sub(1)) % self.max_size);
        self.state.select(Some(i));
    }

    fn update(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            _ => {}
        }
    }
}

impl SearchState {
    pub fn new() -> Self {
        let results = SearchResult {
            albums: None,
            songs: None,
            artists: None,
        };

        Self {
            active: false,
            input: String::new(),
            results,
            cursor_position: 0,
        }
    }

    pub fn update(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.active = false,
            KeyCode::Backspace => self.delete_char(),
            _ => {}
        }
    }

    pub fn handle_char(&mut self, new_char: char) {
        if !self.active {
            if new_char == 'e' {
                self.active = true;
            }
            return;
        }

        self.input.insert(self.byte_index(), new_char);
        self.move_cursor_right();
    }

    // returns the byte index based on the current cursor position.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(index, _)| index)
            .nth(self.cursor_position)
            .unwrap_or(self.input.len())
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn delete_char(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let from_left_to_cursor = self.cursor_position - 1;
        // takes the text before and after the selected character to be deleted.
        let before_selection = self.input.chars().take(from_left_to_cursor);
        let after_selection = self.input.chars().skip(self.cursor_position);

        self.input = before_selection.chain(after_selection).collect();
        self.move_cursor_left()
    }

    fn clamp_cursor(&self, new_position: usize) -> usize {
        new_position.clamp(0, self.input.chars().count())
    }
}
