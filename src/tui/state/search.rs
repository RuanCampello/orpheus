use crate::tui::keyboard::Navigable;
use crate::tui::state::playlist::Playable;
use ratatui::crossterm::event::KeyCode;
use ratatui::widgets::TableState;
use rspotify::model::album::SimplifiedAlbum;
use rspotify::model::artist::FullArtist;
use rspotify::model::page::Page;
use rspotify::model::track::FullTrack;

#[derive(Debug, PartialEq, Default)]
pub(in crate::tui) enum ActiveResult {
    Songs,
    Artists,
    Albums,
    #[default]
    None,
}

pub(in crate::tui) struct ResultItem<T> {
    pub data: Option<T>,
    pub table_state: TableStateExt,
}

pub(in crate::tui) struct SearchResult {
    pub artists: ResultItem<Page<FullArtist>>,
    pub songs: ResultItem<Page<FullTrack>>,
    pub albums: ResultItem<Page<SimplifiedAlbum>>,
    pub active: ActiveResult,
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

impl<T> ResultItem<Page<T>> {
    pub fn new(result: Page<T>) -> Self {
        let length = result.items.len();

        Self {
            data: Some(result),
            table_state: TableStateExt::new(length),
        }
    }
}

impl<T> Default for ResultItem<T> {
    fn default() -> Self {
        Self {
            data: None,
            table_state: TableStateExt::new(0),
        }
    }
}

impl TableStateExt {
    pub fn new(max_size: usize) -> Self {
        let state = TableState::default().with_selected(0);
        Self {
            state,
            max_size,
            active: false,
        }
    }
}

impl Playable for ResultItem<Page<FullTrack>> {
    fn get_selected_track(&self) -> (Option<String>, Option<usize>, Option<String>) {
        let page = self.data.as_ref();
        let idx = self.table_state.state.selected().unwrap_or(0);

        page.and_then(|page| {
            page.items
                .get(idx)
                .map(|song| (Some(song.uri.clone()), Some(idx), Some(song.uri.clone())))
        })
        .unwrap_or((None, None, None))
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

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl AsRef<TableStateExt> for TableStateExt {
    fn as_ref(&self) -> &TableStateExt {
        self
    }
}

impl SearchState {
    pub fn new() -> Self {
        let results = SearchResult {
            albums: ResultItem::default(),
            songs: ResultItem::default(),
            artists: ResultItem::default(),
            active: ActiveResult::default(),
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

    /// Sets the active result option based on the target.
    pub fn set_active(&mut self, target: ActiveResult) {
        self.disable_all();

        match target {
            ActiveResult::Songs => {
                self.results.songs.table_state.active = true;
            }
            ActiveResult::Albums => {
                self.results.albums.table_state.active = true;
            }
            ActiveResult::Artists => {
                self.results.artists.table_state.active = true;
            }
            ActiveResult::None => self.disable_all(),
        }

        self.results.active = target;
    }

    fn disable_all(&mut self) {
        self.results.songs.table_state.active = false;
        self.results.albums.table_state.active = false;
        self.results.artists.table_state.active = false;
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
