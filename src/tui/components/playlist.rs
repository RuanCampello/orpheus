use crate::tui::components::{pad, BlockExt, ListExt};
use crate::tui::State;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, List, ListItem};
use ratatui::{text, Frame};

pub fn draw_playlists_section<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let playlists_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(pad("Playlist", 2))
        .bordered_section(state.playlist_state.active);

    let playlists: Vec<ListItem> = state
        .playlist_state
        .playlists
        .iter()
        .take(10)
        .map(|i| ListItem::new(vec![text::Line::from(Span::raw(&i.name))]))
        .collect();

    let playlists = List::new(playlists)
        .block(playlists_block)
        .highlightable_section(state.playlist_state.active);

    frame.render_stateful_widget(playlists, area, &mut state.playlist_state.state);
}
